# Shard

Shard is a CLI program for offscreen rendering WGSL pixel (fragment) shaders into images (PNG) or animations (GIF).

Shard can be considered as alternative to [Shadertoy](https://www.shadertoy.com/) and similar projects, but for local image and animation generation.

Goals:

- Write **only shader code** - no C/C++/Rust/JS/... wrappers for rendering
- **Single-file specification** due to WGSL
- **Reproducible** - ability to integrate with reproducible research instruments like [Jupyter](https://jupyter.org/), [R Markdown](https://rmarkdown.rstudio.com/), or your notes/articles writing system
- Only **one**, **small** and working **fast** (~13 MB) binary file without external dependencies and footprint
- **Cross-platform**

## Installation

Shard can be installed using [Cargo](https://www.rust-lang.org/).

```sh
cargo install shard\
  --index sparse+https://gitea.zarux.ru/api/packages/deverte/cargo/
```

## First steps

Write WGSL shader `wave.wgsl`:

```rust
@group(0) @binding(0) var<uniform> in_res: vec2<u32>;
@group(0) @binding(1) var<uniform> in_frame: u32;

const FRAMES = 10.0;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> @builtin(position) vec4<f32> {
    var vertices = array<vec4<f32>, 6>(
        vec4<f32>(-1.0, 1.0, 0.0, 1.0),
        vec4<f32>(1.0, 1.0, 0.0, 1.0),
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(1.0, 1.0, 0.0, 1.0),
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(1.0, -1.0, 0.0, 1.0),
    );
    return vertices[in_vertex_index];
}

fn palette(
    t: f32, a: vec3<f32>,
    b: vec3<f32>,
    c: vec3<f32>,
    d: vec3<f32>,
) -> vec3<f32> {
    return a + b * abs(cos(6.28318 * (c * t + d)));
}

fn ocean(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.0, 0.5, 0.5);
    let b = vec3<f32>(0.1, 0.3, 0.5);
    let c = vec3<f32>(0.0, 0.5, 0.333);
    let d = vec3<f32>(0.0, 0.5, 0.667);
    return palette(t, a, b, c, d);
}

@fragment
fn fs_main(
    @builtin(position) in_pos: vec4<f32>,
) -> @location(0) vec4<f32> {
    let uv = in_pos.xy / vec2(f32(in_res.x), f32(in_res.y));
    let step = 3.14159 / FRAMES;
    return vec4<f32>(ocean(uv.x * 4 * sin(f32(in_frame) * step)), 1.0);
}
```

Compile:

```sh
shard wave.wgsl -f 10 -x 128 -y 128 -o wave.gif
```

Result:

![](assets/wave.gif)

## Documentation

### CLI

Usage: `shard [OPTIONS] <INPUT>`

Arguments:

- `<INPUT>` - Input WGSL (.wgsl) fragment (pixel) shader

Options:

- `-o, --output <OUTPUT>` - Output file (with .png or .gif extension) [default: image.png]
- `-x, --width <WIDTH>` - Image width (must be multiple of [256 / 4]) [default: 512]
- `-y, --height <HEIGHT>` - Image height [default: 512]
- `-f, --frames-count <FRAMES_COUNT>` - Number of frames (for GIF output) [default: 60]
- `-h, --help` - Print help
- `-v, --version` - Print version

### Shader

Shard provides two uniforms for fragment shader:

- `@group(0) @binding(0) var<uniform> in_res: vec2<u32>;` - screen resolution
- `@group(0) @binding(1) var<uniform> in_frame: u32;` - frame number (for GIF)

## License

- License: [GPL-3](./LICENSE)
- Author: [Artem Shepelin](mailto:4.shepelin@gmail.com)