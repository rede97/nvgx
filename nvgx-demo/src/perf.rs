use std::time::Instant;

use nvgx::{Align, Color, Context, Extent, Point, Rect, RendererDevice};

#[derive(Default)]
pub struct PerfGraph<const N: usize> {
    values: Vec<f32>,
    idx: usize,
    name: String,
}

impl<const N: usize> PerfGraph<N> {
    pub fn new(name: String) -> Self {
        let mut values: Vec<f32> = Vec::with_capacity(N);
        values.resize_with(N, || 0.0);
        return Self {
            values,
            idx: 0,
            name,
        };
    }

    #[inline]
    pub fn update(&mut self, v: f32) {
        self.values[self.idx % N] = v;
        self.idx += 1;
    }

    pub fn render<R: RendererDevice, F, FTM, FTS>(
        &self,
        ctx: &mut Context<R>,
        rect: Rect,
        color: Color,
        mut val_norm: F,
        main_text: FTM,
        sub_text: FTS,
    ) -> anyhow::Result<f32>
    where
        F: FnMut(f32) -> f32,
        FTM: FnOnce(f32) -> Option<String>,
        FTS: FnOnce(f32) -> Option<String>,
    {
        let average_value = self.values.iter().fold(0.0, |acc, &x| acc + x) / (N as f32);

        ctx.begin_path();

        ctx.rect(rect);
        ctx.fill_paint(nvgx::Color::rgba(0.0, 0.0, 0.0, 0.5));
        ctx.fill()?;

        ctx.begin_path();

        let bottom = rect.xy.y + rect.size.height;
        ctx.move_to((rect.xy.x, bottom));
        for (idx, v) in (0..N)
            .into_iter()
            .map(|i| (i, self.values[(i + self.idx) % N]))
        {
            let x_off = idx as f32 / (N - 1) as f32 * rect.size.width;
            let y_off = f32::clamp(val_norm(v), 0.0, 1.0) * rect.size.height;
            ctx.line_to((rect.xy.x + x_off, bottom - y_off));
        }
        ctx.line_to((rect.xy.x + rect.size.width, bottom));
        ctx.fill_paint(nvgx::Color::rgba(color.r, color.g, color.b, 0.5));
        ctx.fill()?;
        {
            ctx.text_align(Align::TOP | Align::LEFT);
            ctx.font_size(20.0);
            ctx.fill_paint(nvgx::Color::rgba_i(240, 240, 240, 192));
            ctx.text(rect.xy.offset(3.0, 3.0), &self.name)?;
        }

        if let Some(main_text) = main_text(average_value) {
            ctx.text_align(Align::TOP | Align::RIGHT);
            ctx.font_size(20.0);
            ctx.fill_paint(nvgx::Color::rgba_i(240, 240, 240, 192));
            ctx.text(rect.xy.offset(rect.size.width - 3.0, 3.0), main_text)?;
        }

        if let Some(sub_text) = sub_text(average_value) {
            ctx.text_align(Align::BOTTOM | Align::RIGHT);
            ctx.font_size(18.0);
            ctx.fill_paint(nvgx::Color::rgba_i(240, 240, 240, 160));
            ctx.text(
                rect.xy
                    .offset(rect.size.width - 3.0, rect.size.height - 3.0),
                sub_text,
            )?;
        }

        Ok(average_value)
    }
}

pub struct Perf {
    prev_start_time: Instant,
    prev_end_time: Instant,
    frame_time_graph: PerfGraph<64>,
    cpu_time_graph: PerfGraph<64>,
    render_time_graph: PerfGraph<64>,
    #[cfg(feature = "save-fps")]
    save_fps: SaveFrameTime,
}

impl Perf {
    pub fn new(_name: String) -> Self {
        Self {
            prev_start_time: Instant::now(),
            prev_end_time: Instant::now(),
            frame_time_graph: PerfGraph::new("Frame".into()),
            cpu_time_graph: PerfGraph::new("CPU".into()),
            render_time_graph: PerfGraph::new("GPU Draw".into()),
            #[cfg(feature = "save-fps")]
            save_fps: SaveFrameTime::new(_name),
        }
    }

    pub fn frame_start(&mut self) -> f32 {
        let frame_start = Instant::now();
        let render_interval = frame_start - self.prev_end_time;
        let frame_duration =
            frame_start - std::mem::replace(&mut self.prev_start_time, frame_start);
        let frame_duration = frame_duration.as_secs_f32();
        self.frame_time_graph.update(frame_duration);
        self.render_time_graph.update(render_interval.as_secs_f32());
        return frame_duration;
    }

    pub fn render<R: RendererDevice>(
        &mut self,
        ctx: &mut Context<R>,
        pos: Point,
        size: Extent,
    ) -> anyhow::Result<()> {
        let cpu_time = Instant::now() - self.prev_start_time;
        self.cpu_time_graph.update(cpu_time.as_secs_f32());
        ctx.reset_transform();
        let x_offset = size.width + 10.0;
        let avg_frame_time = self.frame_time_graph.render(
            ctx,
            Rect { xy: pos, size },
            Color::rgb_i(0x00, 0xBF, 0xBF),
            |v| v * 1000.0 / 10.0,
            |v| Some(format!("{:.1} FPS", 1.0 / v)),
            |v| Some(format!("{:.1} ms", v * 1000.0)),
        )?;
        #[cfg(feature = "save-fps")]
        self.save_fps.push(avg_frame_time);

        self.cpu_time_graph.render(
            ctx,
            Rect {
                xy: pos.offset(x_offset * 1.0, 0.0),
                size,
            },
            Color::rgb_i(255, 192, 00),
            |v| v * 1000.0 / 10.0,
            |v| Some(format!("{:.1} ms", v * 1000.0)),
            |_| None,
        )?;
        self.render_time_graph.render(
            ctx,
            Rect {
                xy: pos.offset(x_offset * 2.0, 0.0),
                size,
            },
            Color::rgb_i(0xFF, 0x64, 0x64),
            |v| v * 1000.0 / 10.0,
            |v| Some(format!("{:.1} ms", v * 1000.0)),
            |_| None,
        )?;
        self.prev_end_time = Instant::now();
        Ok(())
    }
}

#[macro_export]
macro_rules! measure_time {
    ($block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        (result, duration)
    }};
}

#[cfg(feature = "save-fps")]
struct SaveFrameTime {
    pub name: String,
    pub data: Vec<f32>,
    pub idx: usize,
}

#[cfg(feature = "save-fps")]
impl SaveFrameTime {
    fn new(name: String) -> Self {
        return Self {
            name,
            data: Vec::new(),
            idx: 0,
        };
    }

    fn push(&mut self, fps: f32) {
        if self.idx == 1024 * 2 {
            println!("fps data done!");
        }
        if self.idx < 1024 {
            self.data.push(fps);
        } else {
            self.data[self.idx % 1024] = fps;
        }
        self.idx += 1;
    }
}

#[cfg(feature = "save-fps")]
impl Drop for SaveFrameTime {
    fn drop(&mut self) {
        use std::io::Write;
        if self.idx < 1024 {
            return;
        }
        let mut f = std::fs::File::create(format!("{}.csv", self.name)).unwrap();
        for ft in self.data.iter() {
            writeln!(f, "{}", ft).unwrap();
        }
    }
}
