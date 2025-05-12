#[macro_use]
extern crate lazy_static;

use anyhow::Error;
use nvg::*;
use rand::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::Range;
use std::time::Instant;

lazy_static! {
    static ref COLORS: [Color; 4] = [
        Color::rgb_i(0x00, 0xBF, 0xA8),
        Color::rgb_i(0x99, 0x66, 0xFF),
        Color::rgb_i(0xFF, 0x64, 0x64),
        Color::rgb_i(0x00, 0xC8, 0xFF)
    ];
}

#[derive(Debug, PartialEq, Hash)]
enum ShapeKind {
    Polygon(u8),
    Squiggle(u8),
}

impl ShapeKind {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        match rng.gen_range(0, 2) {
            0 => ShapeKind::Polygon(rng.gen_range(3, 6)),
            1 => ShapeKind::Squiggle(rng.gen_range(3, 6)),
            _ => unreachable!(),
        }
    }
}

struct ShapeCache<R: RendererDevice> {
    shapes: HashMap<ShapeKind, Shape<R>>,
    instances: HashMap<u32, ShapeInstance>,
}

impl<R: RendererDevice> ShapeCache<R> {
    fn new() -> Self {
        ShapeCache {
            shapes: HashMap::new(),
            instances: HashMap::new(),
        }
    }

    // fn get<T: Rng>(&mut self, pair: (u16, u16), rng: &mut T) -> &mut Shape {
    //     let index = ShapeCache::elegent_pair(pair);
    //     self.0.entry(index).or_insert_with(|| Shape::new(rng))
    // }

    fn elegent_pair((x, y): (u16, u16)) -> u32 {
        let a = x as u32;
        let b = y as u32;

        if a >= b {
            a * a + a + b
        } else {
            a + b * b
        }
    }
}

struct ShapeInstance {
    kind: ShapeKind,
    pos: (f32, f32),
    rotation: f32,
    speed: f32,
}

impl ShapeInstance {
    fn new<T: Rng>(kind: ShapeKind, pos: (f32, f32), rng: &mut T) -> Self {
        let direction = [-1.0f32, 1.0f32].choose(rng).unwrap();
        return Self {
            kind,
            pos,
            rotation: rng.gen_range(0.0, 2.0 * PI),
            speed: rng.gen_range(1.0, 4.0) * direction,
        };
    }

    fn update(&mut self, dt: f32) -> Transform {
        self.rotation = self.rotation + dt * self.speed;
        return Transform::translate(self.pos.0, self.pos.1) * Transform::rotate(self.rotation);
    }
}

struct Shape<R: RendererDevice> {
    path: Path<R>,
}

impl<R: RendererDevice> Shape<R> {
    fn new(kind: ShapeKind, size: f32) -> Self {
        let margin = size * 0.2;
        let size = size - margin * 2.0;
        let path = match kind {
            ShapeKind::Polygon(sides) => Self::create_polygon(size, sides),
            ShapeKind::Squiggle(phi) => Self::create_squiggle((size, size / 3.0), phi),
        };
        let direction = [-1.0f32, 1.0f32].choose(rng).unwrap();
        return Self { path };
    }

    // fn draw(&self, ctx: &mut nvg::Context<R>, (x, y): (f32, f32), size: f32) {
    //     let margin = size * 0.2;
    //     let x = x + margin;
    //     let y = y + margin;
    //     let size = size - margin * 2.0;
    //     let half_size = size / 2.0;
    //     let pos = (x + half_size, y + half_size);
    //     match self.kind {
    //         ShapeKind::Polygon(sides) => {
    //             Shape::render_polygon(ctx, pos, size, self.rotation, self.color, sides)
    //         }
    //         ShapeKind::Squiggle(phi) => {
    //             Shape::render_squiggle(ctx, pos, (size, size / 3.0), self.rotation, self.color, phi)
    //         }
    //     };
    // }

    fn get_polygon_point(index: u32, num_sides: u32, radius: f32) -> (f32, f32) {
        let px = radius * (2.0 * PI * index as f32 / num_sides as f32).cos();
        let py = radius * (2.0 * PI * index as f32 / num_sides as f32).sin();
        (px, py)
    }

    fn create_polygon(diameter: f32, num_sides: u8) -> Path<R> {
        assert!(num_sides >= 3);
        let radius = diameter / 2.0;
        let num_sides = num_sides as u32;

        let mut path = Path::new();
        path.move_to(Self::get_polygon_point(0, num_sides, radius));
        for i in 1..num_sides {
            path.line_to(Self::get_polygon_point(i, num_sides, radius));
        }
        path.close_path();
        return path;
    }

    fn create_squiggle((w, h): (f32, f32), phi: u8) -> Path<R> {
        let phi = phi as f32;
        let mut points = [(0.0, 0.0); 64];
        for i in 0..points.len() {
            let pct = i as f32 / (points.len() as f32 - 1.0);
            let theta = pct * PI * 2.0 * phi + PI / 2.0;
            let sx = w * pct - w / 2.0;
            let sy = h / 2.0 * theta.sin();
            points[i as usize] = (sx, sy);
        }
        let mut path = Path::new();
        path.move_to(points[0]);
        for point in points.iter().skip(1) {
            path.line_to(*point);
        }
        return path;
    }
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

fn get_elapsed(instant: &Instant) -> f32 {
    let elapsed = instant.elapsed();
    let elapsed = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;
    elapsed as f32
}

fn render_cutout<R: RendererDevice>(
    ctx: &mut Context<R>,
    (x, y): (f32, f32),
    (w, h): (f32, f32),
    (mx, my): (f32, f32),
) {
    let base_circle_size = 200.0;
    let circle_thickness = 25.0;

    ctx.begin_path();
    ctx.rect((x, y, w, h));
    ctx.circle((mx, my), base_circle_size);
    ctx.path_winding(WindingSolidity::Hole);
    ctx.close_path();
    ctx.fill_paint(Color::rgb(1.0, 1.0, 1.0));
    ctx.fill().unwrap();

    ctx.begin_path();
    ctx.move_to((0, 0));
    ctx.circle((mx, my), base_circle_size + circle_thickness);
    ctx.circle((mx, my), base_circle_size);
    ctx.path_winding(WindingSolidity::Hole);
    ctx.close_path();
    ctx.fill_paint(Color::rgba_i(90, 94, 100, 25));
    ctx.fill().unwrap();

    ctx.begin_path();
    ctx.move_to((0, 0));
    ctx.circle((mx, my), base_circle_size);
    ctx.circle((mx, my), base_circle_size - circle_thickness);
    ctx.path_winding(WindingSolidity::Hole);
    ctx.close_path();
    ctx.fill_paint(Color::rgba_i(0, 0, 0, 25));
    ctx.fill().unwrap();
}

fn render_rectangle<R: RendererDevice>(
    ctx: &mut Context<R>,
    (x, y): (f32, f32),
    (w, h): (f32, f32),
    color: Color,
) {
    ctx.begin_path();
    ctx.rect((x, y, w, h));
    ctx.fill_paint(color);
    ctx.fill().unwrap();
}

mod demo;

struct DemoCutout<R: RendererDevice> {
    start_time: Instant,
    prev_time: f32,
    a: Path<R>,
    rng: ThreadRng,
    instances: Instances<R>,
    mouse: (f32, f32),
    smoothed_mouse: (f32, f32),
}

impl<R: RendererDevice> Default for DemoCutout<R> {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            prev_time: 0.0,
            a: Shape::create_polygon(50.0, 5),
            rng: thread_rng(),
            instances: Instances::new(vec![
                Transform::identity(),
                Transform::translate(200.0, 200.0),
            ]),
            mouse: (0.0, 0.0),
            smoothed_mouse: (0.0, 0.0),
        }
    }
}

impl<R: RendererDevice> demo::Demo<R> for DemoCutout<R> {
    fn update(&mut self, width: f32, height: f32, ctx: &mut Context<R>) -> Result<(), Error> {
        let elapsed = get_elapsed(&self.start_time);
        let delta_time = elapsed - self.prev_time;
        self.prev_time = elapsed;

        self.smoothed_mouse = smooth_mouse(self.mouse, self.smoothed_mouse, delta_time, 7.0);

        let block_size = 75.0;
        let offset = block_size / 2.0;

        render_rectangle(
            ctx,
            (0.0, 0.0),
            (width, height),
            Color::rgb_i(0xFF, 0xFF, 0xAF),
        );

        let paint = Paint {
            fill: COLORS[0].into(),
            stroke: COLORS[0].into(),
            stroke_width: 3.0,
            ..Default::default()
        };

        ctx.draw_path(
            &self.a,
            &paint,
            DrawPathStyle::FILL,
            Some((&self.instances, 0..2)),
        )?;

        // let max_cols = (width / block_size) as u16 + 2;
        // let max_rows = (height / block_size) as u16 + 2;

        // for x in 0..max_cols {
        //     for y in 0..max_rows {
        //         let shape = self.shapes.get((x, y), &mut self.rng);
        //         shape.update(delta_time);
        //         let x = x as f32 * block_size - offset;
        //         let y = y as f32 * block_size - offset;
        //         shape.draw(ctx, (x, y), block_size);
        //     }
        // }

        ctx.reset_transform();
        render_cutout(ctx, (0.0, 0.0), (width, height), self.smoothed_mouse);
        Ok(())
    }

    fn cursor_moved(&mut self, x: f32, y: f32) {
        self.mouse = (x, y);
    }
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
    demo::run(DemoCutout::default(), "demo-cutout");
}
