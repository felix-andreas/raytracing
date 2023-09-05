use itertools::Itertools;

fn main() {
    let width = 2;
    let height = 2;
    let pixels = vec![
        Pixel(0.1, 0.2, 0.3),
        Pixel(0.4, 0.2, 1.0),
        Pixel(0.8, 0.2, 0.7),
        Pixel(1.0, 0.2, 0.3),
    ];
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
