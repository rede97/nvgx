use super::Demo;
use anyhow::anyhow;
use nvg::{Align, Color};
use nvg_gl;

use std::time::Instant;

use std::ffi::CString;
use std::num::NonZeroU32;

use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use glutin::config::{Config, ConfigTemplateBuilder, GetGlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentContext, PossiblyCurrentContext, Version,
};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};

use glutin_winit::{DisplayBuilder, GlWindow};

pub fn run<D: Demo<nvg_gl::Renderer>>(demo: D, title: &str) {
    let event_loop = EventLoop::new().unwrap();
    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(true);

    let display_builder =
        DisplayBuilder::new().with_window_attributes(Some(window_attributes().with_title(title)));

    let mut app = App::new(template, display_builder, demo);
    event_loop.run_app(&mut app).unwrap();

    app.exit_state.unwrap();
}

enum GlDisplayCreationState {
    /// The display was not build yet.
    Builder(DisplayBuilder),
    /// The display was already created for the application.
    Init,
}

struct AppState {
    gl_surface: Surface<WindowSurface>,
    // NOTE: Window should be dropped after all resources created using its
    // raw-window-handle.
    window: Window,
    context: nvg::Context<nvg_gl::Renderer>,
}

struct App<D: Demo<nvg_gl::Renderer>> {
    template: ConfigTemplateBuilder,
    demo: D,
    start_time: Instant,
    total_frames: u32,
    fps: String,
    // NOTE: `AppState` carries the `Window`, thus it should be dropped after everything else.
    state: Option<AppState>,
    gl_context: Option<PossiblyCurrentContext>,
    gl_display: GlDisplayCreationState,
    exit_state: anyhow::Result<()>,
}

impl<D: Demo<nvg_gl::Renderer>> App<D> {
    fn new(template: ConfigTemplateBuilder, display_builder: DisplayBuilder, demo: D) -> Self {
        Self {
            template,
            demo,
            start_time: Instant::now(),
            total_frames: 0,
            fps: String::new(),
            gl_display: GlDisplayCreationState::Builder(display_builder),
            exit_state: Ok(()),
            gl_context: None,
            state: None,
        }
    }

    #[inline]
    fn draw(&mut self) {
        let state = self.state.as_mut().unwrap();
        let context = &mut state.context;
        self.demo.before_frame(context).unwrap();

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
        context.clear(Color::rgb(0.1, 0.1, 0.1)).unwrap();

        context.save();
        self.demo
            .update(window_size.width as f32, window_size.height as f32, context)
            .unwrap();
        context.restore();

        context.save();
        let duration = Instant::now() - self.start_time;
        if duration.as_millis() > 20 {
            self.fps = format!(
                "FPS: {:.2}",
                (self.total_frames as f32) / duration.as_secs_f32()
            );
            self.start_time = Instant::now();
            self.total_frames = 0;
        } else {
            self.total_frames += 1;
        }
        context.begin_path();
        context.fill_paint(Color::rgb(1.0, 0.0, 0.0));
        context.font("roboto");
        context.font_size(20.0);
        context.text_align(Align::TOP | Align::LEFT);
        context.text((10, 10), &self.fps).unwrap();
        context.fill().unwrap();
        context.restore();
        context.end_frame().unwrap();
        state.window.request_redraw();
    }
}

impl<D: Demo<nvg_gl::Renderer>> ApplicationHandler for App<D> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, gl_config) = match &self.gl_display {
            // We just created the event loop, so initialize the display, pick the config, and
            // create the context.
            GlDisplayCreationState::Builder(display_builder) => {
                let (window, gl_config) = match display_builder.clone().build(
                    event_loop,
                    self.template.clone(),
                    gl_config_picker,
                ) {
                    Ok((window, gl_config)) => (window.unwrap(), gl_config),
                    Err(err) => {
                        self.exit_state = Err(anyhow!("{:?}", err));
                        event_loop.exit();
                        return;
                    }
                };

                println!("Picked a config with {} samples", gl_config.num_samples());

                // Mark the display as initialized to not recreate it on resume, since the
                // display is valid until we explicitly destroy it.
                self.gl_display = GlDisplayCreationState::Init;

                // Create gl context.
                self.gl_context =
                    Some(create_gl_context(&window, &gl_config).treat_as_possibly_current());

                (window, gl_config)
            }
            GlDisplayCreationState::Init => {
                println!("Recreating window in `resumed`");
                // Pick the config which we already use for the context.
                let gl_config = self.gl_context.as_ref().unwrap().config();
                match glutin_winit::finalize_window(event_loop, window_attributes(), &gl_config) {
                    Ok(window) => (window, gl_config),
                    Err(err) => {
                        self.exit_state = Err(err.into());
                        event_loop.exit();
                        return;
                    }
                }
            }
        };

        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        // The context needs to be current for the Renderer to set up shaders and
        // buffers. It also performs function loading, which needs a current context on
        // WGL.
        let gl_context = self.gl_context.as_ref().unwrap();
        gl_context.make_current(&gl_surface).unwrap();
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_config.display().get_proc_address(&symbol) as *const _
        });

        let context = {
            // Create the renderer and context.
            let renderer = nvg_gl::Renderer::create(nvg_gl::RenderConfig::default()).unwrap();
            let mut context = nvg::Context::create(renderer).unwrap();
            let scale_factor = window.scale_factor() as f32;
            self.demo.init(&mut context, scale_factor).unwrap();
            self.start_time = Instant::now();
            context
        };

        assert!(self
            .state
            .replace(AppState {
                gl_surface,
                window,
                context
            })
            .is_none());
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        println!("Android window removed");

        // Destroy the GL Surface and un-current the GL Context before ndk-glue releases
        // the window back to the system.
        self.state = None;

        // Make context not current.
        self.gl_context = Some(
            self.gl_context
                .take()
                .unwrap()
                .make_not_current()
                .unwrap()
                .treat_as_possibly_current(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                if let Some(AppState {
                    gl_surface,
                    window: _,
                    context: _,
                }) = self.state.as_ref()
                {
                    let gl_context = self.gl_context.as_ref().unwrap();
                    gl_surface.resize(
                        gl_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );
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
                self.draw();
            }
            _ => (),
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // NOTE: The handling below is only needed due to nvidia on Wayland to not crash
        // on exit due to nvidia driver touching the Wayland display from on
        // `exit` hook.
        let _gl_display = self.gl_context.take().unwrap().display();

        // Clear the window.
        self.state = None;
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(AppState {
            gl_surface,
            window,
            context: _,
        }) = self.state.as_ref()
        {
            let gl_context = self.gl_context.as_ref().unwrap();
            // let renderer = self.renderer.as_ref().unwrap();
            // renderer.draw();
            window.request_redraw();

            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }
}

fn window_attributes() -> WindowAttributes {
    Window::default_attributes()
        .with_transparent(true)
        .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
}

fn create_gl_context(window: &Window, gl_config: &Config) -> NotCurrentContext {
    let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

    // The context creation part.
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);

    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    // Reuse the uncurrented context from a suspended() call if it exists, otherwise
    // this is the first time resumed() is called, where the context still
    // has to be created.
    let gl_display = gl_config.display();

    unsafe {
        gl_display
            .create_context(gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(gl_config, &fallback_context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    })
            })
    }
}

// Find the config with the maximum number of samples, so our triangle will be
// smooth.
pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}
