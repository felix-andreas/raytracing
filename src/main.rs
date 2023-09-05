use itertools::Itertools;

fn main() {
    let width = 256;
    let height = 256;
    let pixels = (0..width)
        .cartesian_product(0..height)
        .map(|(i, j)| {
            Pixel(
                i as f64 / (width - 1) as f64,
                j as f64 / (height - 1) as f64,
                0.0,
            )
        })
        .collect::<Vec<_>>();
    write_ppm(width, height, &pixels);
}

struct Pixel(f64, f64, f64);

fn write_ppm(width: i32, height: i32, pixels: &[Pixel]) {
    std::fs::write(
        "out.ppm",
        format!(
            "P3\n{width} {height}\n255\n{}",
            pixels
                .iter()
                .map(|Pixel(r, g, b)| format!("{:.0} {:.0} {:.0}", 255.0 * r, 255.0 * g, 255.0 * b))
                .join("\n")
        ),
    )
    .unwrap()
}
