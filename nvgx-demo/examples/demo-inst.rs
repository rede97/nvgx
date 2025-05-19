#[macro_use]
extern crate lazy_static;

use anyhow::Error;
use nvgx::*;
use rand::prelude::*;
use slab::Slab;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::Instant;

const BLOCK_SIZE: f32 = 75.0;

lazy_static! {
    static ref COLORS: [Color; 4] = [
        Color::rgb_i(0x00, 0xBF, 0xA8),
        Color::rgb_i(0x99, 0x66, 0xFF),
        Color::rgb_i(0xFF, 0x64, 0x64),
        Color::rgb_i(0x00, 0xC8, 0xFF)
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ShapeKind {
    Polygon(u8),
    Squiggle(u8),
}

impl ShapeKind {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        match rng.random_range(0..2) {
            0 => ShapeKind::Polygon(rng.random_range(3..6)),
            1 => ShapeKind::Squiggle(rng.random_range(3..6)),
            _ => unreachable!(),
        }
    }
}

trait ShapeDraw<R: RendererDevice> {
    fn get_or_append<'a, T: Rng>(
        &mut self,
        caches: &'a mut ShapeCache<R>,
        pair: (u16, u16),
        rng: &mut T,
    ) -> &'a ShapeInstance;
    fn draw(
        &mut self,
        caches: &mut ShapeCache<R>,
        ctx: &mut nvgx::Context<R>,
        dt: f32,
    ) -> anyhow::Result<()>;
}

#[allow(unused)]
struct ShapeDrawSingle<R: RendererDevice> {
    loc_map: HashMap<u32, usize>,
    instances: Instances<R>,
}

#[allow(unused)]
impl<R: RendererDevice> ShapeDrawSingle<R> {
    fn new() -> Self {
        return Self {
            loc_map: HashMap::new(),
            instances: Instances::new(Vec::new()),
        };
    }
}

impl<R: RendererDevice> ShapeDraw<R> for ShapeDrawSingle<R> {
    fn get_or_append<'a, T: Rng>(
        &mut self,
        caches: &'a mut ShapeCache<R>,
        pair: (u16, u16),
        rng: &mut T,
    ) -> &'a ShapeInstance {
        let offset = BLOCK_SIZE / 2.0;

        let index = ShapeCache::<R>::elegent_pair(pair);
        if let Some(inst_id) = self.loc_map.get(&index) {
            return caches.shape_insts.get(*inst_id).unwrap();
        }
        let new_kind = ShapeKind::rand(rng);
        if !caches.shapes.contains_key(&new_kind) {
            caches
                .shapes
                .insert(new_kind, ShapeCache::create_path(new_kind, BLOCK_SIZE));
        }

        let x = pair.0 as f32 * BLOCK_SIZE - offset;
        let y = pair.1 as f32 * BLOCK_SIZE - offset;
        let new_inst = ShapeInstance::new(new_kind, (x, y), rng);
        let new_inst_id = caches.shape_insts.insert(new_inst);
        self.loc_map.insert(index, new_inst_id);
        return caches.shape_insts.get(new_inst_id).unwrap();
    }

    fn draw(
        &mut self,
        caches: &mut ShapeCache<R>,
        ctx: &mut nvgx::Context<R>,
        dt: f32,
    ) -> anyhow::Result<()> {
        self.instances.clear();
        self.instances
            .extend(caches.shape_insts.iter_mut().map(|(_, i)| i.update(dt)));
        ctx.update_instances(&self.instances)?;
        for (idx, inst) in caches.shape_insts.iter() {
            let path = caches.shapes.get(&inst.kind).unwrap();
            let paint = Paint {
                fill: COLORS[inst.color].into(),
                stroke: COLORS[inst.color].into(),
                stroke_width: 3.0,
                ..Default::default()
            };
            match &inst.kind {
                ShapeKind::Polygon(_) => {
                    ctx.draw_path(
                        path,
                        &paint,
                        DrawPathStyle::FILL,
                        Some((&self.instances, idx as u32..(idx + 1) as u32)),
                    )?;
                }
                ShapeKind::Squiggle(_) => {
                    ctx.draw_path(
                        path,
                        &paint,
                        DrawPathStyle::STROKE,
                        Some((&self.instances, idx as u32..(idx + 1) as u32)),
                    )?;
                }
            };
        }
        return Ok(());
    }
}

#[allow(unused)]
struct ShapeDrawByProperity<R: RendererDevice> {
    loc_map: HashMap<u32, usize>,
    color_seqs: [HashMap<ShapeKind, Vec<usize>>; 4],
    instances: Instances<R>,
}

#[allow(unused)]
impl<R: RendererDevice> ShapeDrawByProperity<R> {
    fn new() -> Self {
        return Self {
            loc_map: HashMap::new(),
            color_seqs: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            instances: Instances::new(Vec::new()),
        };
    }
}

impl<R: RendererDevice> ShapeDraw<R> for ShapeDrawByProperity<R> {
    fn get_or_append<'a, T: Rng>(
        &mut self,
        caches: &'a mut ShapeCache<R>,
        pair: (u16, u16),
        rng: &mut T,
    ) -> &'a ShapeInstance {
        let offset = BLOCK_SIZE / 2.0;

        let index = ShapeCache::<R>::elegent_pair(pair);
        if let Some(inst_id) = self.loc_map.get(&index) {
            return caches.shape_insts.get(*inst_id).unwrap();
        }
        let new_kind = ShapeKind::rand(rng);
        if !caches.shapes.contains_key(&new_kind) {
            caches
                .shapes
                .insert(new_kind, ShapeCache::create_path(new_kind, BLOCK_SIZE));
        }

        let x = pair.0 as f32 * BLOCK_SIZE - offset;
        let y = pair.1 as f32 * BLOCK_SIZE - offset;
        let new_inst = ShapeInstance::new(new_kind, (x, y), rng);
        let color_idx = new_inst.color;
        let new_inst_id = caches.shape_insts.insert(new_inst);
        self.loc_map.insert(index, new_inst_id);
        {
            let color_shape = self.color_seqs.get_mut(color_idx).unwrap();
            let insts_id = color_shape.entry(new_kind).or_insert(Vec::new());
            insts_id.push(new_inst_id);
        }
        return caches.shape_insts.get(new_inst_id).unwrap();
    }

    fn draw(
        &mut self,
        caches: &mut ShapeCache<R>,
        ctx: &mut nvgx::Context<R>,
        dt: f32,
    ) -> anyhow::Result<()> {
        self.instances.clear();
        for (_, color_seq) in self.color_seqs.iter().enumerate() {
            for (_, insts_id) in color_seq.iter() {
                self.instances.extend(
                    insts_id
                        .iter()
                        .map(|idx| caches.shape_insts[*idx].update(dt)),
                );
            }
        }
        ctx.update_instances(&self.instances)?;

        let mut insts_offset: u32 = 0;
        for (color_idx, color_seq) in self.color_seqs.iter().enumerate() {
            let color = COLORS[color_idx];
            let paint = Paint {
                fill: color.into(),
                stroke: color.into(),
                stroke_width: 3.0,
                ..Default::default()
            };
            for (shape_kind, insts_id) in color_seq.iter() {
                let path = caches.shapes.get(shape_kind).unwrap();
                let insts_end = insts_offset + (insts_id.len() as u32);
                match shape_kind {
                    ShapeKind::Polygon(_) => {
                        ctx.draw_path(
                            path,
                            &paint,
                            DrawPathStyle::FILL,
                            Some((&self.instances, insts_offset..insts_end)),
                        )?;
                    }
                    ShapeKind::Squiggle(_) => {
                        ctx.draw_path(
                            path,
                            &paint,
                            DrawPathStyle::STROKE,
                            Some((&self.instances, insts_offset..insts_end)),
                        )?;
                    }
                };
                insts_offset = insts_end;
            }
        }
        return Ok(());
    }
}

struct ShapeCache<R: RendererDevice> {
    shapes: HashMap<ShapeKind, Path<R>>,
    shape_insts: Slab<ShapeInstance>,
}

impl<R: RendererDevice> ShapeCache<R> {
    fn new() -> Self {
        ShapeCache {
            shapes: HashMap::new(),
            shape_insts: Slab::new(),
        }
    }

    fn elegent_pair((x, y): (u16, u16)) -> u32 {
        let a = x as u32;
        let b = y as u32;

        if a >= b { a * a + a + b } else { a + b * b }
    }

    fn create_path(kind: ShapeKind, size: f32) -> Path<R> {
        let margin = size * 0.2;
        let size = size - margin * 2.0;
        let path = match kind {
            ShapeKind::Polygon(sides) => Self::create_polygon(size, sides),
            ShapeKind::Squiggle(phi) => Self::create_squiggle((size, size / 3.0), phi),
        };
        return path;
    }

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

#[allow(unused)]
struct ShapeInstance {
    kind: ShapeKind,
    pos: (f32, f32),
    rotation: f32,
    speed: f32,
    color: usize,
}

impl ShapeInstance {
    fn new<T: Rng>(kind: ShapeKind, pos: (f32, f32), rng: &mut T) -> Self {
        let direction = [-1.0f32, 1.0f32].choose(rng).unwrap();
        let color_idx: [usize; 4] = [0, 1, 2, 3];
        return Self {
            kind,
            pos,
            rotation: rng.random_range(0.0..2.0 * PI),
            speed: rng.random_range(1.0..4.0) * direction,
            color: *color_idx.choose(rng).unwrap(),
        };
    }

    fn update(&mut self, dt: f32) -> Transform {
        self.rotation = self.rotation + dt * self.speed;
        return Transform::rotate(self.rotation) * Transform::translate(self.pos.0, self.pos.1);
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

struct DemoCutout<R: RendererDevice> {
    start_time: Instant,
    prev_time: f32,
    rng: ThreadRng,
    shapes: ShapeCache<R>,
    #[cfg(feature = "example-single-inst")]
    drawer: ShapeDrawSingle<R>,
    #[cfg(not(feature = "example-single-inst"))]
    drawer: ShapeDrawByProperity<R>,
    mouse: (f32, f32),
    smoothed_mouse: (f32, f32),
}

impl<R: RendererDevice> Default for DemoCutout<R> {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            prev_time: 0.0,
            shapes: ShapeCache::new(),
            #[cfg(feature = "example-single-inst")]
            drawer: ShapeDrawSingle::new(),
            #[cfg(not(feature = "example-single-inst"))]
            drawer: ShapeDrawByProperity::new(),
            rng: rand::rng(),
            mouse: (0.0, 0.0),
            smoothed_mouse: (0.0, 0.0),
        }
    }
}

impl<R: RendererDevice> nvgx_demo::Demo<R> for DemoCutout<R> {
    fn update(&mut self, width: f32, height: f32, ctx: &mut Context<R>) -> Result<(), Error> {
        let elapsed = get_elapsed(&self.start_time);
        let delta_time = elapsed - self.prev_time;
        self.prev_time = elapsed;

        self.smoothed_mouse = smooth_mouse(self.mouse, self.smoothed_mouse, delta_time, 7.0);

        render_rectangle(
            ctx,
            (0.0, 0.0),
            (width, height),
            Color::rgb_i(0xFF, 0xFF, 0xAF),
        );

        let max_cols = (width / BLOCK_SIZE) as u16 + 2;
        let max_rows = (height / BLOCK_SIZE) as u16 + 2;

        for x in 0..max_cols {
            for y in 0..max_rows {
                self.drawer
                    .get_or_append(&mut self.shapes, (x, y), &mut self.rng);
            }
        }
        self.drawer.draw(&mut self.shapes, ctx, delta_time)?;
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
    nvgx_demo::run(DemoCutout::default(), "demo-cutout-inst", false);
}
