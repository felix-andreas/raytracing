use itertools::Itertools;

fn main() {
    let width = 1280;
    let height = 720;
    let pixels = (0..width)
        .cartesian_product(0..height)
        .map(|(i, j)| {
            DVec3(
                i as f64 / (width - 1) as f64,
                j as f64 / (height - 1) as f64,
                0.0,
            )
        })
        .collect::<Vec<_>>();
    write_ppm(width, height, &pixels);
}

struct DVec3(f64, f64, f64);
struct Ray {
    origin: DVec3,
    direction: DVec3,
}

impl Ray {
    fn at(&self, t: f64) -> DVec3 {
        DVec3(
            self.origin.0 + t * self.direction.0,
            self.origin.1 + t * self.direction.1,
            self.origin.2 + t * self.direction.2,
        )
    }
}

fn write_ppm(width: i32, height: i32, pixels: &[DVec3]) {
    std::fs::write(
        "out.ppm",
        format!(
            "P3\n{width} {height}\n255\n{}",
            pixels
                .iter()
                .map(|DVec3(r, g, b)| format!("{:.0} {:.0} {:.0}", 255.0 * r, 255.0 * g, 255.0 * b))
                .join("\n")
        ),
    )
    .unwrap()
}
