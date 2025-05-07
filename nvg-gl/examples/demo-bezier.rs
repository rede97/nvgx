use anyhow::Error;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use nvg::*;
mod demo;

struct ControlPoint {
    p: (f32, f32),
    color: Color,
    clicked: bool,
}

impl ControlPoint {
    pub fn new(x: f32, y: f32, color: Color) -> Self {
        return Self {
            p: (x, y),
            color,
            clicked: false,
        };
    }

    pub fn draw<R: RendererDevice>(&self, ctx: &mut Context<R>) -> Result<(), Error> {
        ctx.begin_path();
        if self.clicked {
            ctx.circle(self.p, 6.0);
        } else {
            ctx.circle(self.p, 4.0);
        }
        ctx.fill_paint(self.color);
        ctx.fill()?;
        Ok(())
    }

    pub fn mouse_event(&mut self, click: bool, x: f32, y: f32) -> bool {
        if click {
            let r2 = f32::powi(x - self.p.0, 2) + f32::powi(y - self.p.1, 2);
            self.clicked = r2 < f32::powi(4.0, 2);
        } else {
            self.clicked = false;
        }
        return self.clicked;
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        if self.clicked {
            self.p = (x, y)
        }
    }
}

struct ControlBezier {
    /// start, end, cp1, cp2
    control_points: [ControlPoint; 4],
}

impl ControlBezier {
    pub fn new() -> Self {
        let blue = Color::rgb(0.2, 0.4, 0.8);
        let orange = Color::rgb(0.8, 0.4, 0.2);
        return Self {
            control_points: [
                ControlPoint::new(100.0, 100.0, blue),
                ControlPoint::new(400.0, 400.0, blue),
                ControlPoint::new(100.0, 200.0, orange),
                ControlPoint::new(200.0, 100.0, orange),
            ],
        };
    }

    pub fn draw<R: RendererDevice>(&self, ctx: &mut Context<R>) -> Result<(), Error> {
        ctx.save();
        ctx.begin_path();
        ctx.move_to(self.control_points[0].p);
        ctx.line_to(self.control_points[1].p);
        ctx.stroke_paint(Color::rgba(0.9, 0.9, 0.9, 0.5));
        ctx.stroke()?;
        ctx.begin_path();
        ctx.move_to(self.control_points[0].p);
        ctx.line_to(self.control_points[2].p);
        ctx.move_to(self.control_points[1].p);
        ctx.line_to(self.control_points[3].p);
        ctx.stroke_paint(Color::rgba(0.2, 0.6, 0.8, 0.5));
        ctx.stroke()?;
        ctx.begin_path();
        ctx.move_to(self.control_points[0].p);
        ctx.bezier_to(
            self.control_points[2].p,
            self.control_points[3].p,
            self.control_points[1].p,
        );
        ctx.stroke_paint(Color::rgb(1.0, 1.0, 1.0));
        ctx.stroke_width(2.0);
        ctx.stroke()?;

        for cp in self.control_points.iter() {
            cp.draw(ctx)?;
        }

        ctx.restore();
        Ok(())
    }

    pub fn mouse_event(&mut self, click: bool, x: f32, y: f32) -> bool {
        for cp in self.control_points.iter_mut() {
            if cp.mouse_event(click, x, y) {
                return true;
            }
        }
        return false;
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        for cp in self.control_points.iter_mut() {
            cp.mouse_move(x, y);
        }
    }
}

struct Triangle {
    control_points: [ControlPoint; 3],
    paint: Paint,
}

impl Triangle {
    pub fn new() -> Self {
        let cyan = Color::rgb(0.2, 0.7, 0.8);
        let mut paint = Paint::new();
        paint.stroke = nvg::Color::rgb(0.9, 0.9, 0.9).into();
        paint.stroke_width = 2.0;
        paint.fill = nvg::Color::rgb(0.6, 0.4, 0.7).into();
        paint.style = PaintStyle::StrokeAndFill;
        return Self {
            control_points: [
                ControlPoint::new(200.0, 500.0, cyan),
                ControlPoint::new(400.0, 600.0, cyan),
                ControlPoint::new(600.0, 200.0, cyan),
            ],
            paint,
        };
    }

    pub fn draw<R: RendererDevice>(
        &mut self,
        ctx: &mut Context<R>,
        wirelines: bool,
    ) -> anyhow::Result<()> {
        let mut path = PathWithCache::new();
        path.move_to(self.control_points[0].p);
        path.line_to(self.control_points[1].p);
        path.line_to(self.control_points[2].p);
        path.close_path();
        if wirelines {
            self.paint.style = PaintStyle::Fill;
            ctx.draw_path(&path, &self.paint)?;
            ctx.draw_wirelines_path(&path, &self.paint.stroke)?;
        } else {
            self.paint.style = PaintStyle::StrokeAndFill;
            ctx.draw_path(&path, &self.paint)?;
        }

        for cp in self.control_points.iter() {
            cp.draw(ctx)?;
        }

        Ok(())
    }

    pub fn mouse_event(&mut self, click: bool, x: f32, y: f32) -> bool {
        for cp in self.control_points.iter_mut() {
            if cp.mouse_event(click, x, y) {
                return true;
            }
        }
        return false;
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        for cp in self.control_points.iter_mut() {
            cp.mouse_move(x, y);
        }
    }
}

struct ArcTo {
    control_points: [ControlPoint; 3],
    radius: f32,
    paint: Paint,
}

impl ArcTo {
    pub fn new() -> Self {
        let cyan = Color::rgb(0.2, 0.7, 0.8);
        let mut paint = Paint::new();
        paint.stroke = nvg::Color::rgb(0.3, 0.8, 0.6).into();
        paint.stroke_width = 2.0;
        paint.style = PaintStyle::Stroke;
        return Self {
            control_points: [
                ControlPoint::new(400.0, 100.0, cyan),
                ControlPoint::new(200.0, 300.0, cyan),
                ControlPoint::new(500.0, 300.0, cyan),
            ],
            paint,
            radius: 50.0,
        };
    }

    pub fn draw<R: RendererDevice>(&mut self, ctx: &mut Context<R>) -> anyhow::Result<()> {
        let mut path = PathWithCache::new();
        path.move_to(self.control_points[0].p);
        path.line_to(self.control_points[1].p);
        path.line_to(self.control_points[2].p);
        ctx.draw_wirelines_path(&path, &nvg::Color::rgba(0.2, 0.4, 0.6, 0.7).into())?;
        let mut path = PathWithCache::new();
        path.move_to(self.control_points[0].p);
        path.arc_to(
            self.control_points[1].p,
            self.control_points[2].p,
            self.radius,
        );
        ctx.draw_path(&path, &self.paint)?;
        for cp in self.control_points.iter() {
            cp.draw(ctx)?;
        }

        Ok(())
    }

    pub fn mouse_event(&mut self, click: bool, x: f32, y: f32) -> bool {
        for cp in self.control_points.iter_mut() {
            if cp.mouse_event(click, x, y) {
                return true;
            }
        }
        return false;
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        for cp in self.control_points.iter_mut() {
            cp.mouse_move(x, y);
        }
    }

    pub fn mouse_wheel(&mut self, y: f32) {
        self.radius = f32::clamp(self.radius + y, 20.0, 500.0);
    }
}

struct DemoDraw {
    img: Option<ImageId>,
    cursor: (f32, f32),
    window_size: (f32, f32),
    bezier: ControlBezier,
    triangle: Triangle,
    arc_to: ArcTo,
    wirelines: bool,
}

impl<R: RendererDevice> demo::Demo<R> for DemoDraw {
    fn init(&mut self, ctx: &mut Context<R>, _scale_factor: f32) -> Result<(), Error> {
        ctx.create_font_from_file("roboto", "nvg-gl/examples/Roboto-Bold.ttf")?;
        self.img = Some(ctx.create_image_from_file(
            ImageFlags::REPEATX | ImageFlags::REPEATY,
            "nvg-gl/examples/lenna.png",
        )?);
        Ok(())
    }

    fn update(&mut self, _width: f32, _height: f32, ctx: &mut Context<R>) -> anyhow::Result<()> {
        self.window_size = (_width, _height);
        self.triangle.draw(ctx, self.wirelines)?;
        self.arc_to.draw(ctx)?;
        self.bezier.draw(ctx)?;

        Ok(())
    }

    fn cursor_moved(&mut self, _x: f32, _y: f32) {
        self.cursor = (
            _x.clamp(0.0, self.window_size.0),
            _y.clamp(0.0, self.window_size.1),
        );
        self.bezier.mouse_move(self.cursor.0, self.cursor.1);
        self.arc_to.mouse_move(self.cursor.0, self.cursor.1);
        self.triangle.mouse_move(self.cursor.0, self.cursor.1);
    }

    fn mouse_event(
        &mut self,
        _btn: winit::event::MouseButton,
        _state: winit::event::ElementState,
    ) {
        let click = _btn == MouseButton::Left && _state == ElementState::Pressed;
        if self.bezier.mouse_event(click, self.cursor.0, self.cursor.1) {
            return;
        }
        if self.arc_to.mouse_event(click, self.cursor.0, self.cursor.1) {
            return;
        }
        if self
            .triangle
            .mouse_event(click, self.cursor.0, self.cursor.1)
        {
            return;
        }
    }

    fn key_event(
        &mut self,
        _key: winit::keyboard::KeyCode,
        state: winit::event::ElementState,
    ) {
        match _key {
            winit::keyboard::KeyCode::KeyL => {
                if state == winit::event::ElementState::Pressed {
                    self.wirelines = !self.wirelines;
                }
            }
            _ => (),
        }
    }

    fn mouse_wheel(&mut self, _delta: MouseScrollDelta) {
        match _delta {
            MouseScrollDelta::LineDelta(_, y) => {
                self.arc_to.mouse_wheel(y);
            }
            _ => {}
        }
    }
}

fn main() {
    demo::run(
        DemoDraw {
            img: None,
            cursor: (0.0, 0.0),
            window_size: (0.0, 0.0),
            bezier: ControlBezier::new(),
            triangle: Triangle::new(),
            arc_to: ArcTo::new(),
            wirelines: false,
        },
        "demo-draw",
    );
}
