# Raytracing

![](https://github.com/user-attachments/assets/1b4dd4d7-9fc5-48f0-b509-daf02820c1f0)

## Usage

```
raytracing run -q medium
```

All options:

```
Usage: raytracing [OPTIONS]

Options:
  -q, --quality <QUALITY>
          Render resolution

          [default: low]

          Possible values:
          - high:   1920x1080
          - medium: 960x540
          - low:    480x270
          - debug:  320x180

  -s, --seed <SEED>
          Seed for rng

          [default: 41]

  -o, --output <OUTPUT>
          Output path

          [default: out.ppm]
```

## Build

Build executable:

```
nix build
```

Render image:

```
nix build .#render
```

## References

* [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html)
