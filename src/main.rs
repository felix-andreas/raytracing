use itertools::Itertools;

fn main() {
    let width = 1280;
    let height = 720;
    let pixels = (0..width)
        .cartesian_product(0..height)
        .map(|(i, j)| DVec3 {
            x: i as f64 / (width - 1) as f64,
            y: j as f64 / (height - 1) as f64,
            z: 0.0,
        })
        .collect::<Vec<_>>();
    write_ppm(width, height, &pixels);
}

struct DVec3 {
    x: f64,
    y: f64,
    z: f64,
}

struct Ray {
    origin: DVec3,
    direction: DVec3,
}

impl Ray {
    fn at(&self, t: f64) -> DVec3 {
        DVec3 {
            x: self.origin.x + t * self.direction.x,
            y: self.origin.y + t * self.direction.y,
            z: self.origin.z + t * self.direction.z,
        }
    }
}

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
