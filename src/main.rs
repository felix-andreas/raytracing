use itertools::Itertools;

/*
 * MAIN
 */

fn main() {
    let width = 720;
    let height = 405;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (width as f64) / (height as f64);
    let focal_length = 1.0;
    let camera_center = DVec3::new(0.0, 0.0, 0.0);

    let viewport_u = DVec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = DVec3::new(0.0, -viewport_height, 0.0);
    let pixel_delta_u = (1.0 / width as f64) * viewport_u;
    let pixel_delta_v = (1.0 / height as f64) * viewport_v;

    let viewport_upper_left =
        camera_center + DVec3::new(0.0, 0.0, -focal_length) - 0.5 * (viewport_u + viewport_v);

    let pixels = (0..height)
        .cartesian_product(0..width)
        .map(|(j, i)| {
            let subpixel = 8;
            (1.0 / (subpixel as f64 * subpixel as f64))
                * (0..subpixel)
                    .cartesian_product(0..subpixel)
                    .map(|(k, l)| {
                        let pixel_center = viewport_upper_left
                            + ((i as f64) + (0.5 + k as f64) * (1.0 / subpixel as f64))
                                * pixel_delta_u
                            + ((j as f64) + (0.5 + l as f64) * (1.0 / subpixel as f64))
                                * pixel_delta_v;
                        let ray = Ray {
                            origin: camera_center,
                            direction: pixel_center - camera_center,
                        };
                        ray.color()
                    })
                    .reduce(|acc, e| acc + e)
                    .unwrap()
        })
        .collect::<Vec<_>>();

    write_ppm(width, height, &pixels);
}

/*
 * Object
 */

#[derive(Debug)]
enum Object {
    Sphere { center: DVec3, radius: f64 },
}

/*
 * Ray
 */

#[derive(Debug)]
struct Ray {
    #[allow(unused)]
    origin: DVec3,
    direction: DVec3,
}

impl Ray {
    #[allow(unused)]
    fn at(&self, t: f64) -> DVec3 {
        self.origin + t * self.direction
    }
    fn color(&self) -> DVec3 {
        fn hit_object(ray: &Ray, object: &Object) -> Option<f64> {
            match object {
                Object::Sphere { center, radius } => {
                    let oc = ray.origin - *center;
                    let a = ray.direction * ray.direction;
                    let b = ray.direction * oc;
                    let c = oc * oc - radius * radius;
                    let discriminant = b * b - a * c;
                    if discriminant < 0.0 {
                        return None;
                    }

                    (discriminant >= 0.0)
                        .then(|| discriminant.sqrt())
                        .and_then(|d_sqrt| match ((-b - d_sqrt) / a, (-b + d_sqrt) / a) {
                            (x, _) if (0.0..f64::INFINITY).contains(&x) => Some(x),
                            (_, y) if (0.0..f64::INFINITY).contains(&y) => Some(y),
                            _ => None,
                        })
                }
            }
        }

        let objects = vec![
            Object::Sphere {
                center: DVec3::new(0.0, -1000.5, -1.0),
                radius: 1000.0,
            },
            Object::Sphere {
                center: DVec3::new(0.0, 0.0, -1.0),
                radius: 0.5,
            },
            Object::Sphere {
                center: DVec3::new(-2.0, -0.25, -2.0),
                radius: 0.25,
            },
            Object::Sphere {
                center: DVec3::new(-3.0, -0.25, -4.0),
                radius: 0.25,
            },
        ];

        if let Some((t, object)) = objects
            .into_iter()
            .filter_map(|object| hit_object(self, &object).map(|t| (t, object)))
            .min_by(|(ta, _), (tb, _)| ta.total_cmp(tb))
        {
            match object {
                Object::Sphere { center, .. } => {
                    let normal = (self.at(t) - center).unit();
                    return 0.5 * (normal + DVec3::new(1.0, 1.0, 1.0));
                }
            }
        }

        let a = 0.5 * (self.direction.unit().y + 1.0);
        (1.0 - a) * DVec3::new(1.0, 1.0, 1.0) + a * DVec3::new(0.5, 0.7, 0.9)
    }
}

/*
 * DVec3
 */

#[derive(Debug, Clone, Copy)]
struct DVec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl DVec3 {
    fn new(x: f64, y: f64, z: f64) -> DVec3 {
        DVec3 { x, y, z }
    }
    fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    fn unit(self) -> DVec3 {
        (1.0 / self.length()) * self
    }
}

impl std::ops::Add for DVec3 {
    type Output = DVec3;

    fn add(self, other: DVec3) -> DVec3 {
        DVec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Sub for DVec3 {
    type Output = DVec3;

    fn sub(self, other: DVec3) -> DVec3 {
        DVec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Mul<DVec3> for f64 {
    type Output = DVec3;

    fn mul(self, vec: DVec3) -> Self::Output {
        DVec3::new(self * vec.x, self * vec.y, self * vec.z)
    }
}

impl std::ops::Mul<DVec3> for DVec3 {
    type Output = f64;

    fn mul(self, other: DVec3) -> Self::Output {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

/*
 * PPM
 */

fn write_ppm(width: i32, height: i32, pixels: &[DVec3]) {
    std::fs::write(
        "out.ppm",
        format!(
            "P3\n{width} {height}\n255\n{}",
            pixels
                .iter()
                .map(|DVec3 { x: r, y: g, z: b }| format!(
                    "{:.0} {:.0} {:.0}",
                    255.0 * r,
                    255.0 * g,
                    255.0 * b
                ))
                .join("\n")
        ),
    )
    .unwrap()
}
