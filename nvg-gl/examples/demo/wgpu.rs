use super::Demo;
use nvg::{Align, Color};
use std::{sync::Arc, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey, PhysicalKey},
    window::{Window, WindowAttributes},
};

pub fn run<D: Demo<nvg_gl::Renderer>>(demo: D, title: &str) {
    let event_loop = EventLoop::new().unwrap();
    let attributes = Window::default_attributes()
        .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
        .with_title(title);
    let mut app = App::new(demo, attributes);
    event_loop.run_app(&mut app).expect("failed to run app");
    app.exit_state.unwrap();
}

struct App<D: Demo<nvg_gl::Renderer>> {
    demo: D,
    start_time: Instant,
    frame_count: u32,
    fps: String,
    // NOTE: `AppState` carries the `Window`, thus it should be dropped after everything else.
    state: Option<AppState>,
    exit_state: anyhow::Result<()>,
    attributes: WindowAttributes,
}

impl<D: Demo<nvg_gl::Renderer>> App<D> {
    fn new(demo: D, attributes: WindowAttributes) -> Self {
        Self {
            demo,
            start_time: Instant::now(),
            frame_count: 0,
            fps: String::new(),
            exit_state: Ok(()),
            state: None,
            attributes,
        }
    }
}

impl<D: Demo<nvg_gl::Renderer>> ApplicationHandler for App<D> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(self.attributes.clone()).unwrap();

        let mut app_state = AppState::new(window).unwrap();
        let scale_factor = app_state.window.scale_factor() as f32;
        // self.demo
        //     .init(&mut app_state.context, scale_factor)
        //     .unwrap();
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
                    // self.demo.before_frame(context).unwrap();

                    let window_size = state.window.inner_size();
                    let scale_factor = state.window.scale_factor() as f32;
                    context
                        .begin_frame(
                            nvg::Extent {
                                width: window_size.width as f32,
                                height: window_size.height as f32,
                            },
                            scale_factor,
                        )
                        .unwrap();
                    // context.clear(Color::rgb(0.1, 0.1, 0.1)).unwrap();

                    // context.save();
                    // self.demo
                    //     .update(window_size.width as f32, window_size.height as f32, context)
                    //     .unwrap();
                    // context.restore();

                    // context.save();
                    // let duration = Instant::now() - self.start_time;
                    // if duration.as_millis() > 20 {
                    //     self.fps = format!(
                    //         "FPS: {:.2}",
                    //         (self.frame_count as f32) / duration.as_secs_f32()
                    //     );
                    //     self.start_time = Instant::now();
                    //     self.frame_count = 0;
                    // } else {
                    //     self.frame_count += 1;
                    // }
                    // context.begin_path();
                    // context.fill_paint(Color::rgb(1.0, 0.0, 0.0));
                    // context.font("roboto");
                    // context.font_size(20.0);
                    // context.text_align(Align::TOP | Align::LEFT);
                    // context.text((10, 10), &self.fps).unwrap();
                    // context.fill().unwrap();
                    // context.restore();
                    // context.end_frame().unwrap();
                    state.context.begin_path();
                    state.context.rect((20, 20, 100, 100));
                    state.context.rect((50, 50, 100, 100));
                    state.context.fill().unwrap();
                    state.context.end_frame().unwrap();
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
    context: nvg::Context<nvg_gl::Renderer>,
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
        let config: wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>> =
            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: caps.formats[0],
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoNoVsync,
                alpha_mode: caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
        surface.configure(&device, &config);

        let renderer = nvg_gl::Renderer::create(device, queue, surface, config)?;
        let context = nvg::Context::create(renderer)?;
        return Ok(Self { window, context });
    }
}
