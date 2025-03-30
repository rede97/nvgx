use anyhow::Error;
use nvg::*;

mod demo;

struct DemoCutout {}

impl Default for DemoCutout {
    fn default() -> Self {
        Self {}
    }
}

impl<R: Renderer> demo::Demo<R> for DemoCutout {
    fn update(&mut self, _width: f32, _height: f32, ctx: &mut Context<R>) -> Result<(), Error> {
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
            ctx.stroke_width(1.0 / 1.5);
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

        Ok(())
    }
}

fn main() {
    demo::run(DemoCutout::default(), "demo-square");
}
