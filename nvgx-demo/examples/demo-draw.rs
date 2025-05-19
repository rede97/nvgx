use anyhow::Error;
use nvgx::*;
use std::f32::consts::PI;
use std::time::Instant;

enum LinesType {
    Stroke,
    WireLines,
}

struct DemoDraw {
    img: Option<ImageId>,
    start_time: Instant,
    prev_time: f32,
    close: bool,
    lines_type: LinesType,
    lines: bool,
    fill: bool,
    mouse: (f32, f32),
    smoothed_mouse: (f32, f32),
}

impl DemoDraw {
    fn demo_lines<R: RendererDevice>(
        &mut self,
        _width: f32,
        _height: f32,
        ctx: &mut Context<R>,
    ) -> Result<(), Error> {
        ctx.text_align(nvgx::Align::LEFT);
        ctx.text((10, 80), "key C: close path")?;
        ctx.text((10, 90), "key F: Fill path")?;
        ctx.text((10, 100), "key L: wirelines")?;
        ctx.text((10, 110), "key S: stroke")?;
        ctx.fill()?;

        ctx.global_composite_operation(CompositeOperation::Basic(BasicCompositeOperation::SrcOver));
        ctx.reset_transform();
        ctx.translate(_width / 2.0, 0.0);
        ctx.save();
        ctx.rotate(PI / 6.0);
        ctx.begin_path();
        ctx.move_to((200, 200));
        ctx.line_to((600, 200));
        ctx.line_to((400, 100));
        ctx.line_to((400, 600));
        if self.close {
            ctx.close_path();
        }
        ctx.restore();
        ctx.circle((700.0, 500.0), 500.0);

        ctx.reset_transform();
        ctx.stroke_paint(Color::rgb_i(0xFF, 0xFF, 0xFF));
        if self.lines {
            ctx.fill_paint(nvgx::Color::rgba_i(90, 120, 250, 100));
            ctx.fill()?;
            match self.lines_type {
                LinesType::Stroke => {
                    ctx.stroke_width(1.0);
                    ctx.stroke()?;
                }
                LinesType::WireLines => {
                    #[cfg(feature = "wirelines")]
                    ctx.wirelines()?;
                }
            }
        } else {
            if self.fill {
                ctx.fill()?;
            } else {
                ctx.stroke_width(3.0);
                ctx.stroke()?;
            }
        }

        Ok(())
    }

    fn demo_image<R: RendererDevice>(
        &mut self,
        _width: f32,
        _height: f32,
        ctx: &mut Context<R>,
    ) -> Result<(), Error> {
        ctx.begin_path();
        let radius = 100.0;
        ctx.fill_paint({
            ImagePattern {
                img: self.img.unwrap(),
                center: (0.0, 0.0).into(),
                size: (200.0, 200.0).into(),
                angle: 0.0,
                alpha: 0.8,
            }
        });
        ctx.circle(self.smoothed_mouse, radius);
        ctx.fill()?;
        Ok(())
    }
}

impl<R: RendererDevice> nvgx_demo::Demo<R> for DemoDraw {
    fn init(&mut self, ctx: &mut Context<R>, _scale_factor: f32) -> Result<(), Error> {
        ctx.create_font_from_file("roboto", nvgx_demo::FONT_PATH)?;
        self.img = Some(ctx.create_image_from_file(
            ImageFlags::REPEATX | ImageFlags::REPEATY,
            nvgx_demo::IMG_PATH,
        )?);
        Ok(())
    }

    fn update(&mut self, width: f32, height: f32, ctx: &mut Context<R>) -> anyhow::Result<()> {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let delta_time = elapsed - self.prev_time;
        self.prev_time = elapsed;
        self.smoothed_mouse = smooth_mouse(self.mouse, self.smoothed_mouse, delta_time, 7.0);
        self.demo_lines(width, height, ctx)?;
        self.demo_image(width, height, ctx)?;

        ctx.begin_path();
        ctx.rect((100.0, 100.0, 300.0, 300.0));
        ctx.fill_paint(Gradient::Linear {
            start: (100, 100).into(),
            end: (400, 400).into(),
            start_color: Color::rgb_i(0xAA, 0x6C, 0x39),
            end_color: Color::rgb_i(0x88, 0x2D, 0x60),
        });
        ctx.fill()?;

        ctx.save();
        ctx.global_composite_operation(CompositeOperation::Basic(BasicCompositeOperation::Lighter));
        let origin = (150.0, 140.0);
        ctx.begin_path();
        ctx.circle(origin, 64.0);
        ctx.move_to(origin);
        ctx.line_join(LineJoin::Round);
        ctx.line_to((origin.0 + 300.0, origin.1 - 50.0));
        ctx.quad_to((300.0, 100.0), (origin.0 + 500.0, origin.1 + 100.0));
        ctx.close_path();
        ctx.fill_paint(Color::rgba(0.2, 0.0, 0.8, 1.0));
        ctx.fill()?;
        ctx.stroke_paint(Color::rgba(1.0, 1.0, 0.0, 1.0));
        ctx.stroke_width(3.0);
        ctx.stroke()?;
        ctx.restore();

        ctx.begin_path();
        let radius = 100.0;
        let distance = 500.0; // Distance to roll
        let rolled = ((elapsed / 5.0).sin() * 0.5 + 0.5) * distance; // Distance currently rolled
        let origin = (rolled + 100.0, 600.0);
        ctx.fill_paint({
            ImagePattern {
                img: self.img.unwrap(),
                center: origin.into(),
                size: (100.0, 100.0).into(),
                angle: rolled / (2.0 * PI * radius) * 2.0 * PI,
                alpha: 1.0,
            }
        });
        ctx.scissor((150, 600, 1000, 200));
        ctx.circle(origin, radius);
        ctx.fill()?;

        ctx.reset_scissor();

        ctx.begin_path();
        ctx.rect((300.0, 310.0, 300.0, 300.0));
        let color = Color::lerp(
            Color::rgb_i(0x2e, 0x50, 0x77),
            Color::rgb_i(0xff, 0xca, 0x77),
            elapsed.sin() * 0.5 + 0.5,
        );
        ctx.fill_paint(Color::rgba(0.2, 0.2, 0.2, 0.7));
        ctx.fill()?;
        ctx.stroke_paint(color);
        ctx.stroke_width(5.0);
        ctx.stroke()?;

        Ok(())
    }

    fn key_event(&mut self, _key: winit::keyboard::KeyCode, state: winit::event::ElementState) {
        if state == winit::event::ElementState::Pressed {
            match _key {
                winit::keyboard::KeyCode::KeyC => {
                    self.close = !self.close;
                }
                winit::keyboard::KeyCode::KeyL => {
                    self.lines = !self.lines;
                }
                winit::keyboard::KeyCode::KeyS => {
                    self.lines_type = match self.lines_type {
                        LinesType::Stroke => LinesType::WireLines,
                        LinesType::WireLines => LinesType::Stroke,
                    }
                }
                winit::keyboard::KeyCode::KeyF => {
                    self.fill = !self.fill;
                }
                _ => (),
            }
        }
    }

    fn cursor_moved(&mut self, x: f32, y: f32) {
        self.mouse = (x, y);
    }
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

fn smooth_mouse(
    mouse: (f32, f32),
    prev_smoothed_mouse: (f32, f32),
    dt: f32,
    speed: f32,
) -> (f32, f32) {
    let smx = lerp(prev_smoothed_mouse.0, mouse.0, dt * speed);
    let smy = lerp(prev_smoothed_mouse.1, mouse.1, dt * speed);
    (smx, smy)
}

fn main() {
    nvgx_demo::run(
        DemoDraw {
            img: None,
            start_time: Instant::now(),
            close: false,
            lines: false,
            lines_type: LinesType::WireLines,
            fill: false,
            prev_time: 0.0,
            mouse: (0.0, 0.0),
            smoothed_mouse: (0.0, 0.0),
        },
        "demo-draw",
    );
}
