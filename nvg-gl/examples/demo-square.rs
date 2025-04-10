use std::time::Instant;

use anyhow::Error;
use nvg::*;
use nvg_gl::{fb::FrameBuffer, Renderer};

mod demo;

struct DemoCutout {
    scale_factor: f32,
    fb: FrameBuffer,
    start_time: Instant,
    mouse: (f32, f32),
}

impl Default for DemoCutout {
    fn default() -> Self {
        Self {
            scale_factor: 0.0,
            fb: FrameBuffer::default(),
            start_time: Instant::now(),
            mouse: (0.0, 0.0),
        }
    }
}

impl DemoCutout {
    pub fn render_fb(&mut self, ctx: &mut Context<Renderer>) -> Result<(), Error> {
        let dt = Instant::now().duration_since(self.start_time).as_secs_f32();
        self.fb.bind();
        ctx.begin_frame((self.fb.width, self.fb.height), self.scale_factor)?;
        ctx.clear(Color::rgb(0.0, 0.0, 0.0))?;
        ctx.begin_path();
        ctx.circle((50, 50), 40.0 + 10.0 * f32::sin(dt));
        ctx.fill_paint(nvg::Color::rgb(0.6, 0.1, 0.8));
        ctx.fill()?;
        ctx.end_frame()?;
        self.fb.unbind();
        Ok(())
    }
}

impl demo::Demo<Renderer> for DemoCutout {
    fn init(&mut self, ctx: &mut Context<Renderer>, scale_factor: f32) -> Result<(), Error> {
        ctx.create_font_from_file("roboto", "nvg-gl/examples/Roboto-Bold.ttf")?;

        self.scale_factor = scale_factor;
        self.fb = FrameBuffer::new(
            ctx,
            (100.0 * scale_factor) as usize,
            (100.0 * scale_factor) as usize,
            ImageFlags::REPEATX | ImageFlags::REPEATY,
        )?;
        self.render_fb(ctx)?;

        Ok(())
    }

    fn before_frame(&mut self, ctx: &mut Context<Renderer>) -> anyhow::Result<()> {
        self.render_fb(ctx)?;
        Ok(())
    }

    fn update(
        &mut self,
        _width: f32,
        _height: f32,
        ctx: &mut Context<Renderer>,
    ) -> Result<(), Error> {
        {
            // draw background
            let pattern = ImagePattern {
                img: self.fb.image,
                angle: 0.0,
                alpha: 1.0,
                size: self.fb.size(),
                center: (0.0, 0.0).into(),
            };
            ctx.begin_path();
            ctx.fill_paint(pattern);
            ctx.rect(nvg::Rect::new((0.0, 0.0).into(), (_width, _height).into()));
            ctx.fill()?;
        }

        ctx.begin_path();
        if true {
            ctx.fill_paint(nvg::Color::rgb(1.0, 0.0, 0.0));
            ctx.rect(nvg::Rect::new(
                nvg::Point::new(250.0, 300.0),
                nvg::Extent::new(40.0, 40.0),
            ));
            ctx.fill()?;

            ctx.begin_path();
            ctx.shape_antialias(false);
            ctx.stroke_paint(nvg::Color::rgb(0.0, 1.0, 0.0));
            ctx.stroke_width(1.0 / self.scale_factor);
            ctx.move_to((100.0, 10.0));
            ctx.line_to((400.0, 500.0));
            ctx.line_to((500.0, 500.0));
            ctx.line_to((100.0, 200.0));
            ctx.stroke()?;
        }
        ctx.shape_antialias(true);
        ctx.begin_path();
        ctx.fill_type(FillType::EvenOdd);
        ctx.circle((250.0, 220.0), 150.0);
        ctx.circle((400.0, 220.0), 150.0);
        ctx.circle((300.0, 350.0), 100.0);
        ctx.path_winding(WindingSolidity::Hole);
        ctx.fill_paint(nvg::Color::rgb_i(255, 192, 0));
        ctx.fill()?;

        ctx.begin_path();
        ctx.move_to((_width / 2.0, _height / 2.0));
        ctx.line_to((self.mouse.0, self.mouse.1));
        let dt = Instant::now().duration_since(self.start_time).as_secs_f32();
        ctx.circle((self.mouse.0, self.mouse.1), 150.0 + f32::cos(dt) * 20.0);
        ctx.fill_paint(nvg::Color::rgba_i(90, 120, 250, 100));
        ctx.fill()?;
        ctx.stroke_paint(nvg::Color::rgb_i(90, 120, 250));
        ctx.wirelines()?;
        Ok(())
    }

    fn cursor_moved(&mut self, x: f32, y: f32) {
        self.mouse = (x, y);
    }
}

fn main() {
    demo::run(DemoCutout::default(), "demo-square");
}
