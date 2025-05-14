# NVGX: Pure-rust NanoVG
`nvgx` is a pure Rust implementation and enhanced version of [NanoVG](https://github.com/memononen/nanovg), not merely a direct C API wrapper. Compared to [nvg](https://github.com/sunli829/nvg), it provides more comprehensive API functionality support, extensive performance optimizations, and improvements in certain visual effects.

* Support `Framebuffer` 
* Support `Path` and `Instanced API`
* Support `WGPU` backend

> [NanoVG](https://github.com/memononen/nanovg) is small antialiased vector graphics rendering library for OpenGL. It has lean API modeled after HTML5 canvas API. It is aimed to be a practical and fun toolset for building scalable user interfaces and visualizations.

### Note
The current OpenGL backend API is based on OpenGL 3.1, while WebGL 2.0 (GLES 3.0) compatibility has been considered but not yet tested. The fragmentation and problematic nature of GPU driver implementations across different vendors remain significant issues, as discussed in the [Glium post-mortem](https://users.rust-lang.org/t/glium-post-mortem/7063 ). With OpenGL 4.0+ APIs being gradually replaced by the more standardized Vulkan, the OpenGL backend should prioritize the relatively stable and unified OpenGL 3.1 standard. Although OpenGL 4.0 has been in existence for 15 years and is supported by the vast majority of modern GPUs, backward compatibility concerns for OpenGL 3.1 are largely obsolete for contemporary hardware. Earlier versions like OpenGL 2.0+ are no longer supported due to their lack of instanced rendering APIs and the excessive complexity of cross-version API and shader compatibility, which introduces unnecessary technical debt.

### Goal
This is a goal that the project hopes to achieve in the future, and everyone is welcome to participate actively. see: [TODO List](./TODO.md) 

## Usage

In the current graphics library, you can select different backend implementations according to your needs, such as WGPU and OpenGL.

* crates.io: [nvgx-ogl](https://crates.io/crates/nvgx-ogl)
* crates.io: [nvgx-wgpu](https://crates.io/crates/nvgx-wgpu)

```toml
[dependencies]
nvgx = "0.2.0"
# Use wgpu backend
nvgx-wgpu = "0.1.0"
# Use OpenGL 3.1 backend
nvgx-ogl = "0.1.0"
```
* Reference example project [nvgx-demo/Cargo.toml](https://github.com/rede97/nvgx/blob/master/nvgx-demo/Cargo.toml) 

### Example Code 

* draw a round rect
```rust
fn update(
        &mut self,
        width: f32,
        height: f32,
        ctx: &mut Context<Renderer>,
    ) -> Result<(), Error> {
    ctx.begin_path();
    ctx.fill_paint(nvgx::Color::rgb(0.9, 0.3, 0.4));
    ctx.rounded_rect(nvgx::Rect::new(
      Point::new(250.0, 300.0),
        Extent::new(80.0, 80.0),
    ), 5.0);
    ctx.fill()?;
}
```

* draw path instance
```rust
pub fn draw(&mut self, ctx: &mut Context<R>) -> anyhow::Result<()> {
    if self.update {
        let path = self.line_path.reset();
        path.move_to(self.control_points[0].p);
        path.line_to(self.control_points[1].p);
        path.line_to(self.control_points[2].p);
        let path = self.path.reset();
        path.move_to(self.control_points[0].p);
        path.arc_to(
            self.control_points[1].p,
            self.control_points[2].p,
            self.radius,
        );
        self.update = false;
    }
    ctx.draw_path(
        &self.line_path,
        &self.line_paint,
        DrawPathStyle::WIRELINES,
        None,
    )?;
    ctx.draw_path(&self.path, &self.paint, DrawPathStyle::STROKE, None)?;
    for cp in self.control_points.iter() {
        cp.draw(ctx)?;
    }

    Ok(())
}
```

## Bench OpenGL with WGPU backend
<img src="screenshots\fps.svg"/>

## Demos

The following command allows you to quickly run a demo, provided that you have cloned the entire project's code — fortunately, the total amount of code is not large.

```
git clone https://github.com/rede97/nvgx
cd nvgx
```

<table>

<tr><td><h3>Simple and Framebuffer</h3>
The tiniest way to use nvgx and framebuffer, can help beginner to start with nvgx.

```
cargo run -p nvgx-demo --example demo-square
```
Use WGPU backend by default
```
cargo run -p nvgx-demo --example demo-square --features "nvgx-demo/ogl"
```
Use OpenGL backend

</td><td>
<img src="screenshots/square.png" width="200" />
</td></tr>

<tr><td><h3>Clock</h3>

```
cargo run -p nvgx-demo --example demo-clock
```

</td><td>
<img src="screenshots/clock.png" width="200" />
</td></tr>

<tr><td><h3>Cutout</h3>

```
cargo run -p nvgx-demo --example demo-cutout
```
Use canvas api to draw cutout

```
cargo run -p nvgx-demo --example demo-inst
```
Use Path and instanced API to draw cutout

</td><td>
  <img src="screenshots/cutout.png" width="200" />
</td></tr>

<tr><td><h3>Draw</h3>

```
cargo run -p nvgx-demo --example demo-draw
```

</td><td>
  <img src="screenshots/draw.png" width="200" />
</td></tr>
<tr><td><h3>Bezier and ArcTo</h3>

```
cargo run -p nvgx-demo --example demo-bezier
```

</td><td>
  <img src="screenshots/bezier.png" width="200" />
</td></tr>
</table>
