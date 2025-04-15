use std::borrow::Cow;

use instance::InstanceState;
use log::trace;

use texture::Texture;
use wgpu::TextureFormat;
use wgpu::{Adapter, Device, Instance, PipelineLayout, Queue, RenderPipeline, ShaderModule};

use winit::platform::run_return::EventLoopExtRunReturn;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget},
};

mod camera;
mod data;
mod instance;
mod texture;

struct RenderState {
    device: Device,
    queue: Queue,
    _shader: ShaderModule,
    target_format: TextureFormat,
    _pipeline_layout: PipelineLayout,
    render_pipeline: RenderPipeline,
    texture_state: texture::TextureData,
    camera_state: camera::CameraState,
}

struct SurfaceState {
    window: winit::window::Window,
    surface: wgpu::Surface,
}

struct App {
    instance: Instance,
    adapter: Option<Adapter>,
    surface_state: Option<SurfaceState>,
    render_state: Option<RenderState>,
    vertex_state: Option<data::VertexState>,
    instance_state: Option<InstanceState>,
}

impl App {
    fn new(instance: Instance) -> Self {
        Self {
            instance,
            adapter: None,
            surface_state: None,
            render_state: None,
            vertex_state: None,
            instance_state: None,
        }
    }
}

impl App {
    fn create_surface<T>(&mut self, event_loop: &EventLoopWindowTarget<T>) {
        let window = winit::window::Window::new(event_loop).unwrap();
        log::info!("WGPU: creating surface for native window");

        // # Panics
        // Currently create_surface is documented to only possibly fail with with WebGL2
        let surface = unsafe {
            self.instance
                .create_surface(&window)
                .expect("Failed to create surface")
        };
        self.surface_state = Some(SurfaceState { window, surface });
    }

    async fn init_render_state(adapter: &Adapter, target_format: TextureFormat) -> RenderState {
        log::info!("Initializing render state");

        log::info!("WGPU: requesting device");
        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        log::info!("WGPU: loading shader");
        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let texture_state = texture::TextureData::new(&device, &queue).unwrap();
        let camera_state = camera::CameraState::new(&device);

        log::info!("WGPU: creating pipeline layout");
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &texture_state.bind_group_layout,
                &camera_state.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        log::info!("WGPU: creating render pipeline");
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[data::VertexData::desc(), instance::InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(target_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        RenderState {
            device,
            queue,
            _shader: shader,
            target_format,
            _pipeline_layout: pipeline_layout,
            render_pipeline,
            texture_state,
            camera_state,
        }
    }

    // We want to defer the initialization of our render state until
    // we have a surface so we can take its format into account.
    //
    // After we've initialized our render state once though we
    // expect all future surfaces will have the same format and we
    // so this stat will remain valid.
    async fn ensure_render_state_for_surface(&mut self) {
        if let Some(surface_state) = &self.surface_state {
            if self.adapter.is_none() {
                log::info!("WGPU: requesting a suitable adapter (compatible with our surface)");
                let adapter = self
                    .instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::default(),
                        force_fallback_adapter: false,
                        // Request an adapter which can render to our surface
                        compatible_surface: Some(&surface_state.surface),
                    })
                    .await
                    .expect("Failed to find an appropriate adapter");

                self.adapter = Some(adapter);
            }
            let adapter: &Adapter = self.adapter.as_ref().unwrap();

            if self.render_state.is_none() {
                log::info!("WGPU: finding supported swapchain format");
                let surface_caps = surface_state.surface.get_capabilities(adapter);
                let swapchain_format = surface_caps.formats[0];
                let rs = Self::init_render_state(adapter, swapchain_format).await;
                self.render_state = Some(rs);
            }
        }
    }

    fn configure_surface_swapchain(&mut self) {
        if let (Some(render_state), Some(surface_state)) = (&self.render_state, &self.surface_state)
        {
            let swapchain_format = render_state.target_format;
            let size = surface_state.window.inner_size();

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: swapchain_format,
                width: size.width,
                height: size.height,
                //present_mode: wgpu::PresentMode::Mailbox,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![swapchain_format],
            };

            log::info!("WGPU: Configuring surface swapchain: format = {swapchain_format:?}, size = {size:?}");
            surface_state
                .surface
                .configure(&render_state.device, &config);
        }
    }

    fn queue_redraw(&self) {
        if let Some(surface_state) = &self.surface_state {
            trace!("Making Redraw Request");
            surface_state.window.request_redraw();
        }
    }

    fn resume<T>(&mut self, event_loop: &EventLoopWindowTarget<T>) {
        log::info!("Resumed, creating render state...");
        self.create_surface(event_loop);
        pollster::block_on(self.ensure_render_state_for_surface());
        self.configure_surface_swapchain();

        self.queue_redraw();
    }
}

fn run(mut event_loop: EventLoop<()>) {
    log::info!("Running mainloop...");

    // doesn't need to be re-considered later
    let instance = Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        //backends: wgpu::Backends::VULKAN,
        //backends: wgpu::Backends::GL,
        ..Default::default()
    });

    let mut app = App::new(instance);

    // It's not recommended to use `run` on Android because it will call
    // `std::process::exit` when finished which will short-circuit any
    // Java lifecycle handling
    event_loop.run_return(move |event, event_loop, control_flow| {
        log::info!("Received Winit event: {event:?}");

        *control_flow = ControlFlow::Wait;
        match event {
            Event::Resumed => {
                app.resume(event_loop);
            }
            Event::Suspended => {
                log::info!("Suspended, dropping render state...");
                app.render_state = None;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_size),
                ..
            } => {
                app.configure_surface_swapchain();
                // Winit: doesn't currently implicitly request a redraw
                // for a resize which may be required on some platforms...
                app.queue_redraw();
            }
            Event::RedrawRequested(_) => {
                log::info!("Handling Redraw Request");

                if let Some(ref surface_state) = app.surface_state {
                    if let Some(ref rs) = app.render_state {
                        let frame = surface_state
                            .surface
                            .get_current_texture()
                            .expect("Failed to acquire next swap chain texture");
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let vertex_state = data::VertexState::new(&rs.device);
                        let instance_state = InstanceState::new(&rs.device);

                        let size = surface_state.window.inner_size();
                        let depth_tex = Texture::create_depth_tex(&rs.device, size);

                        let mut encoder =
                            rs.device
                                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: None,
                                });
                        {
                            let mut rpass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: None,
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                                            store: true,
                                        },
                                    })],
                                    depth_stencil_attachment: Some(
                                        wgpu::RenderPassDepthStencilAttachment {
                                            view: &depth_tex.view,
                                            depth_ops: Some(wgpu::Operations {
                                                load: wgpu::LoadOp::Clear(1.0),
                                                store: true,
                                            }),
                                            stencil_ops: None,
                                        },
                                    ),
                                });

                            rpass.set_pipeline(&rs.render_pipeline);

                            rpass.set_bind_group(0, &rs.texture_state.bind_group, &[]);
                            rpass.set_bind_group(1, &rs.camera_state.bind_group, &[]);

                            rpass.set_vertex_buffer(0, vertex_state.vertex_buffer.slice(..));
                            rpass.set_vertex_buffer(1, instance_state.instance_buffer.slice(..));
                            rpass.set_index_buffer(
                                vertex_state.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );

                            rpass.draw_indexed(0..vertex_state.num_indices, 0, 0..1);
                        }

                        rs.queue.submit(Some(encoder.finish()));
                        frame.present();
                        surface_state.window.request_redraw();
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: _, .. } => {
                log::info!("Window event {:#?}", event);
            }
            _ => {}
        }
    });
}

fn _main(event_loop: EventLoop<()>) {
    run(event_loop);
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info) // Default Log Level
        .parse_default_env()
        .init();

    let event_loop = EventLoopBuilder::new().build();
    _main(event_loop);
}
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let event_loop = EventLoopBuilder::new().with_android_app(app).build();
    _main(event_loop);
}
