use std::fs;

use crate::{
    common::Camera,
    gpu_buffer::{self, CameraUniform},
    materials::{EmitMaterial, Material, ScatterMaterial},
    scene::Scene,
    InitConfig, Vec3f,
};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Uniforms {
    pub vp_width: u32,
    pub vp_height: u32,
    pub frame_count: u32,
}
impl Uniforms {
    pub fn from_init_configs(init_configs: &InitConfig) -> Self {
        Self {
            vp_width: init_configs.vp_width,
            vp_height: init_configs.vp_height,
            frame_count: 0,
        }
    }
}

pub struct PathTracer {
    device: wgpu::Device,
    queue: wgpu::Queue,

    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,

    camera: Camera,
    camera_uniform_buffer: wgpu::Buffer,

    display_pipeline: wgpu::RenderPipeline,
    data_bind_group: wgpu::BindGroup,
    radiance_bind_group: [wgpu::BindGroup; 2],
}

impl PathTracer {
    pub fn move_camera(&mut self, key_code: winit::keyboard::KeyCode) {
        use winit::keyboard::KeyCode;
        match key_code {
            KeyCode::KeyW => {
                self.camera.move_forward();
            }
            KeyCode::KeyS => {
                self.camera.move_backward();
            }
            KeyCode::KeyA => {
                self.camera.move_left();
            }
            KeyCode::KeyD => {
                self.camera.move_right();
            }
            KeyCode::Space => {
                self.camera.move_up();
            }
            KeyCode::ShiftLeft => {
                self.camera.move_down();
            }
            _ => return,
        }

        self.uniforms.frame_count = 0;
    }
    fn generate_scene() -> Scene {
        let mut scene = Scene::new();
        let factor = 0.5;
        let lambertain_ground = ScatterMaterial::new_lambertain(Vec3f::new(0.8, 0.8, 0.0));
        let lambertain_red = ScatterMaterial::new_lambertain(Vec3f::new(factor, 0.0, 0.0));
        let lambertain_green = ScatterMaterial::new_lambertain(Vec3f::new(0.0, factor, 0.0));
        let lambertain_blue = ScatterMaterial::new_lambertain(Vec3f::new(0.0, 0.0, factor));

        let metal_shiny = ScatterMaterial::new_metal(Vec3f::new(0.8, 0.8, 0.8), 0.9);
        let metal_fuzzy = ScatterMaterial::new_metal(Vec3f::new(0.8, 0.8, 0.8), 0.2);

        let glass_outer = ScatterMaterial::new_dielectric(1.5);
        let glass_inner = ScatterMaterial::new_dielectric(1.0 / 1.5);

        // So the buffer is not empty
        let diffuse = EmitMaterial::new_diffuse(5f32);

        let mat_ground_id = scene.add_material(Material::ScatterMaterial(lambertain_ground));
        let mat_red_id = scene.add_material(Material::ScatterMaterial(lambertain_red));
        let mat_green_id = scene.add_material(Material::ScatterMaterial(lambertain_green));
        let mat_blue_id = scene.add_material(Material::ScatterMaterial(lambertain_blue));

        let mat_metal_shiny_id = scene.add_material(Material::ScatterMaterial(metal_shiny));
        let mat_metal_fuzzy_id = scene.add_material(Material::ScatterMaterial(metal_fuzzy));

        let mat_glass_outer_id = scene.add_material(Material::ScatterMaterial(glass_outer));
        let mat_glass_inner_id = scene.add_material(Material::ScatterMaterial(glass_inner));

        #[allow(unused_variables)]
        let diffuse_id = scene.add_material(Material::EmitMaterial(diffuse));

        let floor_sphere =
            gpu_buffer::Sphere::new(Vec3f::new(0.0, -100.5, -1.0), 100.0, mat_ground_id);
        let left_sphere = gpu_buffer::Sphere::new(Vec3f::new(-1.0, 0.0, -1.0), 0.5, mat_red_id);
        let middle_sphere = gpu_buffer::Sphere::new(Vec3f::new(0.0, 0.0, -1.0), 0.5, mat_green_id);
        let right_sphere = gpu_buffer::Sphere::new(Vec3f::new(1.0, 0.0, -1.0), 0.5, mat_blue_id);

        let right_metal_sphere =
            gpu_buffer::Sphere::new(Vec3f::new(2.0, 0.0, -2.2), 0.5, mat_metal_shiny_id);
        let righter_metal_sphere =
            gpu_buffer::Sphere::new(Vec3f::new(3.0, 0.0, -2.2), 0.5, mat_metal_fuzzy_id);
        let right_glass_ball_sphere_outer =
            gpu_buffer::Sphere::new(Vec3f::new(3.0, 0.0, -1.0), 0.5, mat_glass_outer_id);
        let right_glass_ball_sphere_inner =
            gpu_buffer::Sphere::new(Vec3f::new(3.0, 0.0, -1.0), 0.4, mat_glass_inner_id);
        let righter_glass_ball =
            gpu_buffer::Sphere::new(Vec3f::new(-2.0, 0.0, -1.0), 0.5, mat_glass_outer_id);

        scene.spheres = vec![
            floor_sphere,
            left_sphere,
            middle_sphere,
            right_sphere,
            right_metal_sphere,
            righter_metal_sphere,
            right_glass_ball_sphere_outer,
            right_glass_ball_sphere_inner,
            righter_glass_ball,
        ];
        scene
    }
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, init_configs: InitConfig) -> Self {
        device.on_uncaptured_error(Box::new(|error| panic!("Aborting due to error, {}", error)));

        // TODO: init GPU resources
        let shader_module = compile_shader_module(&device);
        let (display_pipeline, [data_bind_group_layout, radiance_bind_group_layout]) =
            create_display_pipeline(&device, &shader_module);

        let uniforms = Uniforms::from_init_configs(&init_configs);
        let camera = Camera::from_init_configs(&init_configs);
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera_uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let scene = Self::generate_scene();

        let radiance_samples =
            create_sample_textures(&device, uniforms.vp_width, uniforms.vp_height);
        let data_bind_group = Self::create_data_bind_group(
            &device,
            data_bind_group_layout,
            &uniform_buffer,
            &camera_uniform_buffer,
            &scene.get_spheres_buffer(&device),
            &scene.get_materials_buffer(&device),
        );
        let radiance_bind_group = Self::create_radiance_bind_group(
            &device,
            radiance_bind_group_layout,
            &radiance_samples,
        );

        Self {
            device,
            queue,

            uniforms,
            uniform_buffer,

            camera,
            camera_uniform_buffer,

            display_pipeline,
            data_bind_group,
            radiance_bind_group,
        }
    }
    pub fn render_frame(&mut self, target: &wgpu::TextureView) {
        // Update to uniforms
        self.uniforms.frame_count += 1;
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
        self.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::bytes_of(&self.camera.to_camera_uniform()),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render frame"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("display pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        render_pass.set_pipeline(&self.display_pipeline);
        render_pass.set_bind_group(0, &self.data_bind_group, &[]);
        render_pass.set_bind_group(
            1,
            &self.radiance_bind_group[(self.uniforms.frame_count % 2) as usize],
            &[],
        );

        // Draw verticies 0-2, 1 instance
        render_pass.draw(0..6, 0..1);

        // Ending the render by dropping the object
        drop(render_pass);

        let command_buffer = encoder.finish();
        self.queue.submit(Some(command_buffer));
    }

    fn create_radiance_bind_group(
        device: &wgpu::Device,
        layout: wgpu::BindGroupLayout,
        textures: &[wgpu::Texture; 2],
    ) -> [wgpu::BindGroup; 2] {
        let views = [
            textures[0].create_view(&wgpu::TextureViewDescriptor {
                ..Default::default()
            }),
            textures[1].create_view(&wgpu::TextureViewDescriptor {
                ..Default::default()
            }),
        ];
        [
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&views[0]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&views[1]),
                    },
                ],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&views[1]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&views[0]),
                    },
                ],
            }),
        ]
    }

    fn create_data_bind_group(
        device: &wgpu::Device,
        layout: wgpu::BindGroupLayout,
        uniform_buffer: &wgpu::Buffer,
        camera_uniform_buffer: &wgpu::Buffer,
        spheres_buffer: &wgpu::Buffer,
        materials_buffer: &[wgpu::Buffer; 3],
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: camera_uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: spheres_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &materials_buffer[0],
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &materials_buffer[1],
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &materials_buffer[2],
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        })
    }
}

fn compile_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
    use std::borrow::Cow;

    #[allow(non_snake_case)]
    let SHADERS: Vec<&str> = vec![
        "vs",
        "constants",
        "bindings",
        "helper",
        "materials",
        "objects",
        "fs",
    ];
    #[allow(non_snake_case)]
    let EXTENSION: &str = "wgsl";
    let code = SHADERS
        .iter()
        .map(|&shader| {
            fs::read_to_string(format!(
                "{}/shaders/{}.{}",
                env!("CARGO_MANIFEST_DIR"),
                shader,
                EXTENSION
            ))
            .unwrap()
        })
        .collect::<String>();

    // let code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/shaders.wgsl"));
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&code)),
    })
}

fn create_display_pipeline(
    device: &wgpu::Device,
    shader_module: &wgpu::ShaderModule,
) -> (wgpu::RenderPipeline, [wgpu::BindGroupLayout; 2]) {
    let radiance_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
    let data_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
    let display_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        cache: None,
        label: Some("display"),
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&data_bind_group_layout, &radiance_bind_group_layout],
                ..Default::default()
            }),
        ),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        },
        vertex: wgpu::VertexState {
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            module: shader_module,
            entry_point: Some("display_vs"),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            module: shader_module,
            entry_point: Some("display_fs"),
            targets: &[Some(wgpu::ColorTargetState {
                blend: None,
                format: wgpu::TextureFormat::Rgba8Unorm,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });
    (
        display_pipeline,
        [data_bind_group_layout, radiance_bind_group_layout],
    )
}

// Width and height are needed, to know how big the texture should be and stored data
fn create_sample_textures(device: &wgpu::Device, width: u32, height: u32) -> [wgpu::Texture; 2] {
    let desc = wgpu::TextureDescriptor {
        label: Some("randiance samples"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    };
    [device.create_texture(&desc), device.create_texture(&desc)]
}
