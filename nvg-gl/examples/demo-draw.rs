use anyhow::Error;
use nvg::*;
use std::f32::consts::PI;
use std::time::Instant;

mod demo;

struct DemoDraw {
    img: Option<ImageId>,
    start_time: Instant,
    close: bool,
    wirelines: bool,
    wireframe: bool,
}

impl<R: Renderer> demo::Demo<R> for DemoDraw {
    fn init(&mut self, ctx: &mut Context<R>, _scale_factor: f32) -> Result<(), Error> {
        ctx.create_font_from_file("roboto", "nvg-gl/examples/Roboto-Bold.ttf")?;
        self.img = Some(ctx.create_image_from_file(
            ImageFlags::REPEATX | ImageFlags::REPEATY,
            "nvg-gl/examples/lenna.png",
        )?);
        Ok(())
    }

    fn update(&mut self, _width: f32, _height: f32, ctx: &mut Context<R>) -> anyhow::Result<()> {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        {
            ctx.wireframe(self.wireframe)?;
            ctx.global_composite_operation(CompositeOperation::Basic(
                BasicCompositeOperation::SrcOver,
            ));
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
            if self.wirelines {
                ctx.fill_paint(nvg::Color::rgba_i(90, 120, 250, 100));
                ctx.fill()?;
                ctx.wirelines()?;
            } else {
                ctx.stroke_width(3.0);
                ctx.stroke()?;
            }

            ctx.wireframe(false)?;
        }

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

    fn key_event(
        &mut self,
        _key: glutin::event::VirtualKeyCode,
        state: glutin::event::ElementState,
    ) {
        match _key {
            glutin::event::VirtualKeyCode::C => {
                if state == glutin::event::ElementState::Pressed {
                    self.close = !self.close;
                }
            }
            glutin::event::VirtualKeyCode::W => {
                if state == glutin::event::ElementState::Pressed {
                    self.wireframe = !self.wireframe;
                }
            }
            glutin::event::VirtualKeyCode::L => {
                if state == glutin::event::ElementState::Pressed {
                    self.wirelines = !self.wirelines;
                }
            }
            _ => (),
        }
    }
}

fn main() {
    demo::run(
        DemoDraw {
            img: None,
            start_time: Instant::now(),
            close: false,
            wireframe: false,
            wirelines: false,
        },
        "demo-draw",
    );
}
