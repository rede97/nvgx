use nvgx::*;
use std::f32::consts::PI;

struct DemoText;

impl<R: RendererDevice> nvgx_demo::Demo<R> for DemoText {
    fn update(&mut self, _width: f32, _height: f32, ctx: &mut Context<R>) -> anyhow::Result<()> {
        ctx.begin_path();
        ctx.move_to((150, 20));
        ctx.line_to((150, 170));
        ctx.stroke_paint((1.0, 0.0, 0.0));
        ctx.stroke()?;

        ctx.save();
        {
            ctx.font("roboto");
            ctx.font_size(16.0);
            ctx.fill_paint((1.0, 1.0, 0.0));

            // horz align
            ctx.text_align(nvgx::Align::LEFT);
            ctx.text((150, 60), "left")?;

            ctx.text_align(nvgx::Align::CENTER);
            ctx.text((150, 80), "center")?;

            ctx.text_align(nvgx::Align::RIGHT);
            ctx.text((150, 100), "right")?;

            // vert align
            ctx.begin_path();
            ctx.move_to((5, 270));
            ctx.line_to((300, 270));
            ctx.stroke_paint((1.0, 0.0, 0.0));
            ctx.stroke()?;

            ctx.text_align(nvgx::Align::TOP);
            ctx.text((5, 270), "top")?;

            ctx.text_align(nvgx::Align::MIDDLE);
            ctx.text((50, 270), "middle")?;

            ctx.text_align(nvgx::Align::BOTTOM);
            ctx.text((120, 270), "bottom")?;

            ctx.text_align(nvgx::Align::BASELINE);
            ctx.text((200, 270), "baseline")?;

            // spaces
            ctx.text((200, 300), "a b  c   d")?;
        }
        ctx.restore();

        ctx.font("notocjk");
        ctx.font_size(40.0);
        ctx.rotate(PI / 4.0);
        ctx.fill_paint(Color::rgb_i(0x00, 0xC8, 0xFF));
        ctx.text((_width / 4.0, 0.0), "Hello 世界，ハローワールド")?;
        Ok(())
    }
}

fn main() {
    nvgx_demo::run(DemoText, "text");
}
