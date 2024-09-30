use {
    clap::{Parser, ValueEnum},
    indicatif::ParallelProgressIterator,
    itertools::Itertools,
    rand::{rngs::StdRng, Rng, SeedableRng},
    rayon::iter::{IntoParallelIterator, ParallelIterator},
    std::{
        fmt::Write,
        fs,
        path::{Path, PathBuf},
        time::Instant,
    },
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Render resolution
    #[arg(short, long, default_value = "low")]
    quality: Quality,

    /// Seed for rng
    #[arg(short, long, default_value_t = 41)]
    seed: u64,

    /// Output path
    #[arg(short, long, default_value = "out.ppm")]
    output: PathBuf,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Quality {
    /// 1920x1080
    High,
    /// 960x540
    Medium,
    /// 480x270
    Low,
    /// 320x180
    Debug,
}

fn main() {
    let start = Instant::now();

    // cli
    let Args {
        seed,
        quality,
        output,
    } = { Args::parse() };

    // render quality
    let (
        image_resolution_x,
        image_resolution_y,
        antialiasing_factor, // factor per axis (x and y)
    ) = match quality {
        Quality::High => (1920, 1080, 32),
        Quality::Medium => (960, 540, 16),
        Quality::Low => (480, 270, 8),
        Quality::Debug => (320, 180, 2),
    };

    // camera controls
    let look_from = (-7.5, 1.9, 14.0);
    let look_at = (0.0, 0.4, 0.0);
    let field_of_view = f64::to_radians(17.0); // vertical angle
    let vup = unit_vector((0.0, 1.0, 0.0));
    let defocus_angle = 0.55;

    let look_delta = (
        look_from.0 - look_at.0,
        look_from.1 - look_at.1,
        look_from.2 - look_at.2,
    );

    let focus_distance = {
        let focus_delta = (look_from.0 - 1.2, look_from.1 - 0.65, look_from.2 - 1.7);
        dot(focus_delta, focus_delta).sqrt() - 0.65
    };

    let w = unit_vector(look_delta);
    let u = unit_vector((
        vup.1 * w.2 - vup.2 * w.1,
        vup.2 * w.0 - vup.0 * w.2,
        vup.0 * w.1 - vup.1 * w.0,
    ));
    let v = (
        w.1 * u.2 - w.2 * u.1,
        w.2 * u.0 - w.0 * u.2,
        w.0 * u.1 - w.1 * u.0,
    );

    let viewport_height = 2.0 * f64::tan(field_of_view / 2.0) * focus_distance;
    let viewport_width = viewport_height * image_resolution_x as f64 / image_resolution_y as f64;
    let viewport_ul_position = (
        look_from.0 - (focus_distance * w.0) - 0.5 * viewport_width * u.0
            + 0.5 * viewport_height * v.0,
        look_from.1 - (focus_distance * w.1) - 0.5 * viewport_width * u.1
            + 0.5 * viewport_height * v.1,
        look_from.2 - (focus_distance * w.2) - 0.5 * viewport_width * u.2
            + 0.5 * viewport_height * v.2,
    );
    let pixel_width = viewport_width / image_resolution_x as f64;
    let pixel_height = viewport_height / image_resolution_y as f64;
    let defocus_radius = focus_distance * f64::tan(0.5 * f64::to_radians(defocus_angle));

    let scene = {
        let a = 16;
        // note: we use two rngs so positions don't change if we change material logic
        let mut rng = StdRng::seed_from_u64(seed); // used for position
        let mut rng2 = StdRng::seed_from_u64(seed); // used for other
        vec![
            Sphere {
                radius: 0.8,
                center: (-2.5, 0.8, 0.7),
                color: (1.0, 1.0, 1.0),
                material: Material::Dielectric(1.5),
            },
            Sphere {
                radius: 0.75,
                center: (-0.3, 0.75, 0.2),
                color: (0.3, 0.7, 0.8),
                material: Material::Diffuse,
            },
            Sphere {
                radius: 0.65,
                center: (1.2, 0.65, 1.7),
                color: (0.9, 0.9, 0.9),
                material: Material::Metal(0.0),
            },
            Sphere {
                radius: 10000.0,
                center: (0.0, -10000.0, 0.0),
                color: (0.95, 0.8, 0.35),
                material: Material::Diffuse,
            },
        ]
        .into_iter()
        .chain(
            (0..a)
                .cartesian_product(0..a)
                .map(|(j, i)| {
                    let max_size = 0.35;
                    let m = rng.gen_range(0..10);
                    let r = rng.gen_range(0.05..max_size);
                    let spread = 0.2 + max_size - r;
                    let x = 1.5 * ((i - a / 2) as f64) + rng.gen_range(-spread..spread);
                    let z = 1.5 * ((j - a / 2) as f64) + rng.gen_range(-spread..spread);
                    (x, z, r, m)
                })
                .map(|(x, z, r, m)| {
                    let (material, color) = {
                        let color = (
                            rng2.gen_range(0.0..1.0),
                            rng2.gen_range(0.0..1.0),
                            rng2.gen_range(0.0..1.0),
                        );
                        match m {
                            0..=3 => (Material::Diffuse, color),
                            4..=6 => (Material::Dielectric(1.5), color),
                            7..=9 => (Material::Metal(rng2.gen_range(0.0..1.0)), color),
                            _ => unreachable!(),
                        }
                    };
                    Sphere {
                        radius: r,
                        center: (x, r, z),
                        color,
                        material,
                    }
                }),
        )
        .collect::<Vec<_>>()
    };

    let image_pixels = (0..image_resolution_y)
        .into_par_iter()
        .flat_map(|y| (0..image_resolution_x).into_par_iter().map(move |x| (y, x)))
        .progress_count((image_resolution_x * image_resolution_y) as u64)
        .map(|(y, x)| {
            let mut rng = StdRng::seed_from_u64(seed);
            let color_sum = (0..antialiasing_factor)
                .cartesian_product(0..antialiasing_factor)
                .map(|(j, i)| {
                    let pixel_position = (
                        viewport_ul_position.0
                            + (x as f64 + (0.5 + i as f64) / antialiasing_factor as f64)
                                * pixel_width
                                * u.0
                            - (y as f64 + (0.5 + j as f64) / antialiasing_factor as f64)
                                * pixel_height
                                * v.0,
                        viewport_ul_position.1
                            + (x as f64 + (0.5 + i as f64) / antialiasing_factor as f64)
                                * pixel_width
                                * u.1
                            - (y as f64 + (0.5 + j as f64) / antialiasing_factor as f64)
                                * pixel_height
                                * v.1,
                        viewport_ul_position.2
                            + (x as f64 + (0.5 + i as f64) / antialiasing_factor as f64)
                                * pixel_width
                                * u.2
                            - (y as f64 + (0.5 + j as f64) / antialiasing_factor as f64)
                                * pixel_height
                                * v.2,
                    );
                    let (a, b) = defocus_disk_sample(&mut rng);
                    let ray_origin = (
                        look_from.0 + a * defocus_radius * u.0 + b * defocus_radius * v.0,
                        look_from.1 + a * defocus_radius * u.1 + b * defocus_radius * v.1,
                        look_from.2 + a * defocus_radius * u.2 + b * defocus_radius * v.2,
                    );
                    let ray_direction = unit_vector((
                        pixel_position.0 - ray_origin.0,
                        pixel_position.1 - ray_origin.1,
                        pixel_position.2 - ray_origin.2,
                    ));
                    compute_color(ray_direction, ray_origin, &scene, &mut rng, 0)
                })
                .fold((0.0, 0.0, 0.0), |accumulator, color| {
                    (
                        (accumulator.0 + color.0),
                        (accumulator.1 + color.1),
                        (accumulator.2 + color.2),
                    )
                });
            let averaging_factor = 1.0 / (antialiasing_factor * antialiasing_factor) as f64;
            (
                (255.999 * (averaging_factor * color_sum.0).sqrt()) as u8,
                (255.999 * (averaging_factor * color_sum.1).sqrt()) as u8,
                (255.999 * (averaging_factor * color_sum.2).sqrt()) as u8,
            )
        })
        .collect::<Vec<(u8, u8, u8)>>();

    println!(
        "resolution: {}x{}, antialiasing: {}, time elapsed: {:.3?}",
        image_resolution_x,
        image_resolution_y,
        antialiasing_factor,
        start.elapsed(),
    );
    write_p3(
        &output,
        image_resolution_x,
        image_resolution_y,
        image_pixels,
    );
}

struct Sphere {
    radius: f64,
    center: (f64, f64, f64),
    color: (f64, f64, f64),
    material: Material,
}

enum Material {
    Diffuse,
    Metal(f64),
    Dielectric(f64),
}

fn compute_color(
    ray_direction: (f64, f64, f64),
    ray_origin: (f64, f64, f64),
    scene: &Vec<Sphere>,
    rng: &mut StdRng,
    depth: i32,
) -> (f64, f64, f64) {
    if depth > 4 {
        return (0.0, 0.0, 0.0);
    }
    fn nearest_hit(
        ray_direction: (f64, f64, f64),
        ray_origin: (f64, f64, f64), // physically it is the destination but as we consider the ray backwards, here it is the origin
        sphere: &Sphere,
    ) -> Option<f64> {
        let distance = (
            ray_origin.0 - sphere.center.0,
            ray_origin.1 - sphere.center.1,
            ray_origin.2 - sphere.center.2,
        );
        let a = dot(ray_direction, ray_direction);
        let b = 2.0 * dot(ray_direction, distance);
        let c = dot(distance, distance) - sphere.radius * sphere.radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant >= 0.0 {
            let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
            let t2 = (-b - discriminant.sqrt()) / (2.0 * a);
            // To be checked whether object can be closer than viewport and if not exclude
            // by tx>=distance to viewport (supposing t is the distance from the viewport)
            match (t1 >= 0.001, t2 >= 0.001) {
                (true, true) => Some(f64::min(t1, t2)),
                (true, false) => Some(t1),
                (false, true) => Some(t2),
                (false, false) => None,
            }
        } else {
            None
        }
    }

    // optional_hit contains a the t of the nearest sphere hit and also this sphere
    let optional_hit = scene
        .iter()
        .filter_map(|sphere| {
            nearest_hit(ray_direction, ray_origin, sphere).map(|t: f64| (t, sphere))
        })
        .min_by(|(t_s1, _), (t_s2, _)| f64::total_cmp(t_s1, t_s2));

    if let Some((t, sphere)) = optional_hit {
        let hit_point = (
            ray_origin.0 + t * ray_direction.0,
            ray_origin.1 + t * ray_direction.1,
            ray_origin.2 + t * ray_direction.2,
        );
        let normal_vector = unit_vector((
            hit_point.0 - sphere.center.0,
            hit_point.1 - sphere.center.1,
            hit_point.2 - sphere.center.2,
        ));
        let previous_color = compute_color(
            match sphere.material {
                Material::Diffuse => lambertian_reflection(normal_vector, rng),
                Material::Metal(fuzziness) => {
                    metalic_reflection(ray_direction, normal_vector, fuzziness, rng)
                }
                Material::Dielectric(refractive_index) => {
                    let (refractive_ratio, normal_vector_adjusted) =
                        if dot(ray_direction, normal_vector) >= 0.0 {
                            (
                                refractive_index / 1.0,
                                (-normal_vector.0, -normal_vector.1, -normal_vector.2),
                            ) // 1.0 assumes that the surrounding is filled with air
                        } else {
                            (1.0 / refractive_index, normal_vector)
                        };
                    dielectric_scatter(ray_direction, normal_vector_adjusted, refractive_ratio, rng)
                }
            },
            hit_point,
            scene,
            rng,
            depth + 1,
        );

        return (
            0.9 * previous_color.0 * sphere.color.0,
            0.9 * previous_color.1 * sphere.color.1,
            0.9 * previous_color.2 * sphere.color.2,
        );
    }

    let a = (ray_direction.1 + 1.0) / 2.0; // factor for sky gradient
    let color_top = (0.4, 0.5, 1.0);
    let color_bottom = (1.0, 1.0, 1.0);
    (
        (1.0 - a) * color_bottom.0 + a * color_top.0,
        (1.0 - a) * color_bottom.1 + a * color_top.1,
        (1.0 - a) * color_bottom.2 + a * color_top.2,
    )
}

fn dot(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

fn unit_vector((x, y, z): (f64, f64, f64)) -> (f64, f64, f64) {
    let normalizer = 1.0 / (x * x + y * y + z * z).sqrt();
    (normalizer * x, normalizer * y, normalizer * z)
}

fn lambertian_reflection(normal_vector: (f64, f64, f64), rng: &mut StdRng) -> (f64, f64, f64) {
    loop {
        let random_vector = (
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        );
        if dot(random_vector, random_vector) <= 1.0 {
            break unit_vector((
                random_vector.0 + normal_vector.0,
                random_vector.1 + normal_vector.1,
                random_vector.2 + normal_vector.2,
            ));
        }
    }
}

fn defocus_disk_sample(rng: &mut StdRng) -> (f64, f64) {
    loop {
        let (a, b) = (rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
        if a * a + b * b <= 1.0 {
            break (a, b);
        }
    }
}

// Cave: Fuzziness could drive ray underneath the sphere-surface. This should be prevented but is currently not.
fn metalic_reflection(
    ray_direction: (f64, f64, f64),
    normal_vector: (f64, f64, f64),
    fuzziness: f64,
    rng: &mut StdRng,
) -> (f64, f64, f64) {
    let dot_ray_normal = dot(ray_direction, normal_vector);
    (
        ray_direction.0 - 2.0 * dot_ray_normal * normal_vector.0
            + fuzziness * rng.gen_range(-1.0..1.0),
        ray_direction.1 - 2.0 * dot_ray_normal * normal_vector.1
            + fuzziness * rng.gen_range(-1.0..1.0),
        ray_direction.2 - 2.0 * dot_ray_normal * normal_vector.2
            + fuzziness * rng.gen_range(-1.0..1.0),
    )
}

fn dielectric_scatter(
    ray_direction: (f64, f64, f64),
    normal_vector: (f64, f64, f64),
    refractive_ratio: f64,
    rng: &mut StdRng,
) -> (f64, f64, f64) {
    let ray_out_perp = {
        let cos_theta = f64::min(
            dot(
                (-ray_direction.0, -ray_direction.1, -ray_direction.2),
                normal_vector,
            ),
            1.0,
        );
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        if refractive_ratio * sin_theta > 1.0
            || reflectance(cos_theta, refractive_ratio) > rng.gen_range(0.0..1.0)
        {
            let dot_ray_normal = dot(ray_direction, normal_vector);
            return (
                ray_direction.0 - 2.0 * dot_ray_normal * normal_vector.0,
                ray_direction.1 - 2.0 * dot_ray_normal * normal_vector.1,
                ray_direction.2 - 2.0 * dot_ray_normal * normal_vector.2,
            );
        }

        (
            refractive_ratio * (ray_direction.0 + cos_theta * normal_vector.0),
            refractive_ratio * (ray_direction.1 + cos_theta * normal_vector.1),
            refractive_ratio * (ray_direction.2 + cos_theta * normal_vector.2),
        )
    };

    let ray_out_parallel = {
        let a = -(1.0 - dot(ray_out_perp, ray_out_perp)).abs().sqrt();
        (
            a * normal_vector.0,
            a * normal_vector.1,
            a * normal_vector.2,
        )
    };

    (
        ray_out_perp.0 + ray_out_parallel.0,
        ray_out_perp.1 + ray_out_parallel.1,
        ray_out_perp.2 + ray_out_parallel.2,
    )
}

fn reflectance(cos_theta: f64, refractive_ratio: f64) -> f64 {
    let r0 = (1.0 - refractive_ratio) / (1.0 + refractive_ratio);
    let r0_squared = r0 * r0;
    r0_squared + (1.0 - r0_squared) * (1.0 - cos_theta).powf(5.0)
}

fn write_p3(path: &Path, width: i32, height: i32, pixels: Vec<(u8, u8, u8)>) {
    let p3_output: String = pixels
        .iter()
        .try_fold(
            String::with_capacity(pixels.len()),
            |mut buffer, (r, g, b)| writeln!(&mut buffer, "{} {} {}", r, g, b).map(|_| buffer),
        )
        .unwrap();
    fs::write(
        path,
        format!("P3\n{} {}\n255\n{}", width, height, p3_output),
    )
    .unwrap();
}
