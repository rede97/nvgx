use crate::{measure_time, perf::Perf};

use super::Demo;

use nvgx::Color;
use nvgx_wgpu::RenderConfig;
use std::{sync::Arc, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey, PhysicalKey},
    window::{Window, WindowAttributes},
};

pub fn run<D: Demo<nvgx_wgpu::Renderer>>(demo: D, title: &str, fb_perf_en: bool) {
    let event_loop = EventLoop::new().unwrap();
    let attributes = Window::default_attributes()
        .with_inner_size(winit::dpi::LogicalSize::new(
            super::DEFAULT_SIZE.0,
            super::DEFAULT_SIZE.1,
        ))
        .with_title(format!("{} (WGPU)", title));
    let mut app = App::new(demo, attributes, fb_perf_en);
    event_loop.run_app(&mut app).expect("failed to run app");
    app.exit_state.unwrap();
}

struct App<D: Demo<nvgx_wgpu::Renderer>> {
    demo: D,
    start_time: Instant,
    perf: Perf,
    // NOTE: `AppState` carries the `Window`, thus it should be dropped after everything else.
    state: Option<AppState>,
    exit_state: anyhow::Result<()>,
    attributes: WindowAttributes,
}

impl<D: Demo<nvgx_wgpu::Renderer>> App<D> {
    fn new(demo: D, attributes: WindowAttributes, fb_perf_en: bool) -> Self {
        Self {
            demo,
            start_time: Instant::now(),
            perf: Perf::new(attributes.title.clone(), fb_perf_en),
            exit_state: Ok(()),
            state: None,
            attributes,
        }
    }
}

impl<D: Demo<nvgx_wgpu::Renderer>> ApplicationHandler for App<D> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(self.attributes.clone()).unwrap();

        let mut app_state = AppState::new(window).unwrap();
        let scale_factor = app_state.window.scale_factor() as f32;
        self.demo
            .init(&mut app_state.context, scale_factor)
            .unwrap();
        self.start_time = Instant::now();
        assert!(self.state.replace(app_state).is_none());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                if let Some(AppState { window: _, context }) = self.state.as_mut() {
                    context.resize(size.width, size.height).unwrap();
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                self.demo.key_event(keycode, state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.demo.cursor_moved(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                self.demo.mouse_event(button, state);
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                self.demo.mouse_wheel(delta);
            }

            WindowEvent::RedrawRequested => {
                let state = unsafe { self.state.as_mut().unwrap_unchecked() };
                {
                    let context = &mut state.context;
                    
                    let (_, fb_duration) = measure_time!({
                        self.demo.before_frame(context).unwrap();
                    });
                    self.perf.update_fb_time(fb_duration.as_secs_f32());
                    self.perf.frame_start();

                    let window_size = state.window.inner_size();
                    let scale_factor = state.window.scale_factor() as f32;
                    context
                        .begin_frame(
                            nvgx::Extent {
                                width: window_size.width as f32,
                                height: window_size.height as f32,
                            },
                            scale_factor,
                        )
                        .unwrap();
                    context.clear(Color::rgb(0.1, 0.1, 0.1)).unwrap();

                    context.save();
                    self.demo
                        .update(window_size.width as f32, window_size.height as f32, context)
                        .unwrap();
                    context.restore();

                    self.perf
                        .render(context, (10.0, 10.0).into(), (200.0, 50.0).into())
                        .unwrap();
                    context.end_frame().unwrap();
                    self.perf.frame_end();
                }

                {
                    state.window.request_redraw();
                }
            }
            _ => (),
        }
    }
}

struct AppState {
    window: Arc<Window>,
    context: nvgx::Context<nvgx_wgpu::Renderer>,
}

impl AppState {
    fn new(window: Window) -> anyhow::Result<Self> {
        let window = Arc::new(window);
        let size = window.inner_size();

        let backends = wgpu::Backends::all();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapters = instance.enumerate_adapters(backends);
        let adapter = adapters
            .iter()
            .filter(|adapter| adapter.is_surface_supported(&surface))
            .next()
            .expect("no avaliable adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::POLYGON_MODE_LINE,
            required_limits: wgpu::Limits::default(),
            label: None,
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .unwrap();

        let caps = surface.get_capabilities(adapter);
        let config = RenderConfig::default();

        let pos = caps
            .formats
            .iter()
            .position(|f| config.format_match(f))
            .expect(&format!(
                "Surface texture format: `{:?}` not support",
                &config.format
            ));
        let surface_config: wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>> =
            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: caps.formats[pos],
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoNoVsync,
                alpha_mode: caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
        surface.configure(&device, &surface_config);

        let renderer = nvgx_wgpu::Renderer::create(config, device, queue, surface, surface_config)?;
        let context = nvgx::Context::create(renderer)?;
        return Ok(Self { window, context });
    }
}
