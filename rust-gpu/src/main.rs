// Originally written in 2023 by Arman Uguray <arman.uguray@gmail.com>
// SPDX-License-Identifier: CC-BY-4.0
// Edited by chuanhao01 2024

use std::sync::Arc;

use anyhow::{Context, Result};
use gpu_path_tracing::{InitConfig, InitParam, PathTracer, Vec3f};
use pollster::FutureExt as _;
use winit;

// Aasync ssign the appropriate window size in terms of physical pixels based on your display DPI.
const WIDTH: u32 = 1500;

struct State<'a> {
    surface: wgpu::Surface<'a>,
    renderer: PathTracer,
}
impl<'a> State<'a> {
    pub async fn new(
        window: Arc<winit::window::Window>,
        init_configs: InitConfig,
    ) -> Result<State<'a>> {
        use wgpu::TextureFormat::Rgba8Unorm;
        // Create an "instance" of wgpu. This is the entry-point to the API.
        let instance = wgpu::Instance::default();

        // Create a drawable "surface" that is associated with the window.
        let surface = instance.create_surface(Arc::clone(&window))?;

        // Request a GPU that is compatible with the surface. If the system has multiple GPUs then
        // pick the high performance one.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .context("failed to find a compatible adapter")?;

        // Connect to the GPU. "device" represents the connection to the GPU and allows us to create
        // resources like buffers, textures, and pipelines. "queue" represents the command queue that
        // we use to submit commands to the GPU.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .context("failed to connect to the GPU")?;

        // Configure the texture memory backs the surface. Our renderer will draw to a surface texture
        // every frame.
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .into_iter()
            .find(|it| matches!(it, Rgba8Unorm))
            .context("could not find preferred texture format (Rgba8Unorm or Bgra8Unorm)")?;
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok(Self {
            surface: surface,
            renderer: PathTracer::new(device, queue, init_configs),
        })
    }
}

struct App<'a> {
    init_configs: InitConfig,
    // Initalized with the window
    state: Option<State<'a>>,
    window: Option<Arc<winit::window::Window>>,
}
impl App<'_> {
    fn new(init_configs: InitConfig) -> Self {
        Self {
            init_configs,
            state: None,
            window: None,
        }
    }
}
impl winit::application::ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let None = self.window {
            let window_attributes = winit::window::Window::default_attributes()
                .with_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(
                    self.init_configs.vp_width,
                    self.init_configs.vp_height,
                )))
                .with_resizable(false)
                .with_title(String::from("GPU Path Tracer"));
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(window.clone());
            self.state = Some(
                State::new(window.clone(), self.init_configs.clone())
                    .block_on()
                    .unwrap(),
            );
        }
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::{ElementState, WindowEvent};
        // Only run if state is up (Should be)
        if let (Some(window), Some(state)) = (&mut self.window, &mut self.state) {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::KeyboardInput {
                    device_id: _,
                    is_synthetic,
                    event,
                } => {
                    if !is_synthetic && event.state == ElementState::Pressed {
                        // Ignore unidentified key inputs
                        if let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key {
                            state.renderer.move_camera(key_code);
                        }
                    }
                    // println!("{:?}", input);
                }
                WindowEvent::RedrawRequested => {
                    // Wait for the next available frame buffer.
                    let frame: wgpu::SurfaceTexture = state
                        .surface
                        .get_current_texture()
                        .expect("failed to get current texture");

                    // TODO: draw frame
                    let render_target = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    state.renderer.render_frame(&render_target);

                    frame.present();
                    window.request_redraw();
                }
                _ => (),
            }
        }
    }
}

#[pollster::main]
async fn main() -> Result<()> {
    let init_configs = InitConfig::new(InitParam {
        vp_width: WIDTH,
        camera_theta: 100f32,
        // look_from: Vec3f::new(-2.0, 2.0, 1.0),
        // look_at: Vec3f::new(3.0, 0.0, -1.0),
        look_from: Vec3f::new(0.0, 0.0, 1.0),
        look_at: Vec3f::new(0.0, 0.0, -1.0),
        v_up: Vec3f::new(0.0, 1.0, 0.0),
        ..Default::default()
    });
    let event_loop = winit::event_loop::EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(init_configs);
    event_loop.run_app(&mut app)?;
    Ok(())
}
