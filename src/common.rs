#![allow(dead_code)]
use bytemuck::{cast_slice, Pod, Zeroable};
use cgmath::{Matrix, Matrix4, SquareMatrix};
use std::{
    iter, mem,
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{
    camera::{self, flatten, Camera, CameraUniform},
    texture,
};
#[path = "transforms.rs"]
mod transforms;

const ANIMATION_SPEED: f32 = 1.0;
const IS_PERSPECTIVE: bool = true;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Light {
    specular_color: [f32; 4],
    ambient_intensity: f32,
    diffuse_intensity: f32,
    specular_intensity: f32,
    specular_shininess: f32,
    is_two_side: i32,
}

pub fn light(
    sc: [f32; 3],
    ambient: f32,
    diffuse: f32,
    specular: f32,
    shininess: f32,
    two_side: i32,
) -> Light {
    Light {
        specular_color: [sc[0], sc[1], sc[2], 1.0],
        ambient_intensity: ambient,
        diffuse_intensity: diffuse,
        specular_intensity: specular,
        specular_shininess: shininess,
        is_two_side: two_side,
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

// pub fn vertex(p: [f32; 3], n: [f32; 3], c: [f32; 3]) -> Vertex {
//     Vertex {
//         position: [p[0], p[1], p[2], 1.0],
//         normal: [n[0], n[1], n[2], 1.0],
//         color: [c[0], c[1], c[2], 1.0],
//     }
// }

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4, 2=>Float32x4];
    // fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    //     wgpu::VertexBufferLayout {
    //         array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
    //         step_mode: wgpu::VertexStepMode::Vertex,
    //         attributes: &Self::ATTRIBUTES,
    //     }
    // }
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4, // For position
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4, // For normal
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4, // For color
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2, // For texture coordinates
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>, // Change to u32 if you need more than 65,536 indices
}

pub struct State {
    pub init: transforms::InitWgpu,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer_vec: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_uniform_buffer: wgpu::Buffer,

    // camera
    camera_uniform: CameraUniform,
    camera: camera::Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    projection: camera::Projection,
    camera_controller: camera::CameraController,
    mouse_pressed: bool,

    // texture
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
    num_vertices_vec: u32,
    index_buffer_vec: wgpu::Buffer,
}

enum Direction {
    X,
    Y,
    Z,
    NegX,
    NegY,
    NegZ,
    None,
}
fn check_visibility(x: usize, y: usize, z: usize, cubes: &Vec<Vec<Vec<Mesh>>>) -> Vec<Direction> {
    let mut directions: Vec<Direction> = Vec::new();
    match cubes.get(x + 1) {
        Some(inner_vec) => match inner_vec.get(y) {
            Some(inner_inner_vec) => match inner_inner_vec.get(z) {
                Some(_) => {}
                None => directions.push(Direction::X),
            },
            None => directions.push(Direction::X),
        },
        None => directions.push(Direction::X),
    };
    // y = z
    match cubes.get(x) {
        Some(inner_vec) => match inner_vec.get(y + 1) {
            Some(inner_inner_vec) => match inner_inner_vec.get(z) {
                Some(_) => {}
                None => directions.push(Direction::Y),
            },
            None => directions.push(Direction::Y),
        },
        None => directions.push(Direction::Y),
    };
    match cubes.get(x) {
        Some(inner_vec) => match inner_vec.get(y) {
            Some(inner_inner_vec) => match inner_inner_vec.get(z + 1) {
                Some(_) => {}
                None => directions.push(Direction::Z),
            },
            None => {}
        },
        None => {}
    };
    if x == 0 {
        directions.push(Direction::NegX);
    } else {
        match cubes.get(x - 1) {
            Some(inner_vec) => match inner_vec.get(y) {
                Some(inner_inner_vec) => match inner_inner_vec.get(z) {
                    Some(_) => {}
                    None => directions.push(Direction::NegX),
                },
                None => {}
            },
            None => {}
        };
    }
    if y == 0 {
        directions.push(Direction::NegY);
    } else {
        match cubes.get(x) {
            Some(inner_vec) => match inner_vec.get(y - 1) {
                Some(inner_inner_vec) => match inner_inner_vec.get(z) {
                    Some(_) => {}
                    None => directions.push(Direction::NegY),
                },
                None => {}
            },
            None => {}
        };
    }
    if z == 0 {
        directions.push(Direction::NegZ);
    } else {
        match cubes.get(x) {
            Some(inner_vec) => match inner_vec.get(y) {
                Some(inner_inner_vec) => match inner_inner_vec.get(z - 1) {
                    Some(_) => {}
                    None => directions.push(Direction::NegZ),
                },
                None => {}
            },
            None => {}
        };
    }
    directions
}

fn get_visible_cubes(cubes: &Vec<Vec<Vec<Mesh>>>) -> Vec<Vertex> {
    let mut visible_cubes: Vec<Vertex> = Vec::new();
    for x in 0..cubes.len() {
        for y in 0..cubes[x].len() {
            for z in 0..cubes[x][y].len() {
                for direction in check_visibility(x, y, z, cubes) {
                    match direction {
                        Direction::X => {
                            visible_cubes.append(cubes[x][y][z].vertices[6..12].to_vec().as_mut());
                        }
                        Direction::Y => {
                            visible_cubes.append(cubes[x][y][z].vertices[24..30].to_vec().as_mut());
                        }
                        // x?
                        Direction::Z => {
                            visible_cubes.append(cubes[x][y][z].vertices[0..6].to_vec().as_mut());
                        }
                        Direction::NegX => {
                            visible_cubes.append(cubes[x][y][z].vertices[18..24].to_vec().as_mut());
                        }
                        // neg z
                        Direction::NegY => {
                            visible_cubes.append(cubes[x][y][z].vertices[30..36].to_vec().as_mut());
                        }
                        // neg y
                        Direction::NegZ => {
                            visible_cubes.append(cubes[x][y][z].vertices[12..18].to_vec().as_mut());
                        }
                        Direction::None => {}
                    }
                }
            }
        }
    }
    visible_cubes
}

impl State {
    pub async fn new(window: &Window, shape_data: &Vec<Vec<Vec<Mesh>>>, light_data: Light) -> Self {
        let mut vertex_data_vec: Vec<Vertex> = Vec::new();
        let visible_cubes: Vec<Vertex> = get_visible_cubes(shape_data);
        dbg!(&visible_cubes.len());
        // let mut vertex_data: Vec<Vertex> = Vec::with_capacity(visible_cubes.len());
        for x in 0..visible_cubes.len() {
            // for j in 0..visible_cubes[x].vertices.len() {
            vertex_data_vec.push(visible_cubes[x]);
            // }
            // vertex_data_vec.push(vertex_data);
        }
        // let mut vertex_data: Vec<Vertex> = Vec::with_capacity(visible_cubes[i].vertices.len());
        // for j in 0..visible_cubes[i].vertices.len() {
        //     vertex_data.push(visible_cubes[i].vertices[j]);
        // }
        // vertex_data_vec.push(vertex_data);

        // for shape_item in shape_data.iter() {
        //     if let Some(positions) = &shape_item.0 {
        //         let mut vertex_data: Vec<Vertex> = Vec::new();
        //         for (i, position) in positions.iter().enumerate() {
        //             // Attempt to retrieve corresponding color and other data, if they exist
        //             let color = shape_item
        //                 .1
        //                 .as_ref()
        //                 .and_then(|c| c.get(i))
        //                 .copied()
        //                 .unwrap_or_default(); // Provide default if not available
        //             let other_data = shape_item
        //                 .2
        //                 .as_ref()
        //                 .and_then(|o| o.get(i))
        //                 .copied()
        //                 .unwrap_or_default(); // Provide default if not available

        //             // Assuming `vertex` can handle optional inputs or defaults
        //             vertex_data.push(vertex(*position, color, other_data));
        //         }
        //         vertex_data_vec.push(vertex_data);
        //     }
        // }
        let init = transforms::InitWgpu::init_wgpu(window).await;

        let shader = init
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                //source: wgpu::ShaderSource::Wgsl(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/examples/ch06/line3d.wgsl")).into()),
            });
        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(
            init.config.width,
            init.config.height,
            cgmath::Deg(45.0),
            0.1,
            100.0,
        );
        let camera_controller = camera::CameraController::new(4.0, 0.4);
        // let camera = Camera {
        //     // position the camera 1 unit up and 2 units back
        //     // +z is out of the screen
        //     eye: (0.0, 1.0, 2.0).into(),
        //     // have it look at the origin
        //     target: (0.0, 0.0, 0.0).into(),
        //     // which way is "up"
        //     up: cgmath::Vector3::unit_y(),
        //     aspect: init.config.width as f32 / init.config.height as f32,
        //     fovy: 45.0,
        //     znear: 0.1,
        //     zfar: 100.0,
        // };
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);
        let camera_buffer = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let camera_bind_group_layout =
            init.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });
        let camera_bind_group = init.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let diffuse_bytes = include_bytes!("grass_block.jpg"); // CHANGED!
        let diffuse_texture = texture::Texture::from_bytes(
            &init.device,
            &init.queue,
            diffuse_bytes,
            "grass_block.png",
        )
        .unwrap(); // CHANGED!
        let texture_bind_group_layout =
            init.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });
        let diffuse_bind_group = init.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view), // CHANGED!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler), // CHANGED!
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        // uniform data
        // let camera_position = (-5.0, -5.0, 3.5).into();
        // let look_direction = (0.0, 0.0, 0.0).into();
        // let up_direction = cgmath::Vector3::unit_y();

        // let (view_mat, project_mat, _view_project_mat) = transforms::create_view_projection(
        //     camera_position,
        //     look_direction,
        //     up_direction,
        //     init.config.width as f32 / init.config.height as f32,
        //     IS_PERSPECTIVE,
        // );

        // create vertex uniform buffer
        // model_mat and view_projection_mat will be stored in vertex_uniform_buffer inside the update function
        let vertex_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Uniform Buffer"),
            size: 192,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // create fragment uniform buffer. here we set eye_position = camera_position and light_position = eye_position
        let fragment_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment Uniform Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // store light and eye positions
        let light_position: &[f32; 3] = camera.position.as_ref();
        let eye_position: &[f32; 3] = camera.position.as_ref();
        init.queue.write_buffer(
            &fragment_uniform_buffer,
            0,
            bytemuck::cast_slice(light_position),
        );
        init.queue.write_buffer(
            &fragment_uniform_buffer,
            16,
            bytemuck::cast_slice(eye_position),
        );

        // create light uniform buffer
        let light_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Uniform Buffer"),
            size: 48,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // store light parameters
        init.queue.write_buffer(
            &light_uniform_buffer,
            0,
            bytemuck::cast_slice(&[light_data]),
        );

        let uniform_bind_group_layout =
            init.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
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
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: Some("Uniform Bind Group Layout"),
                });

        let uniform_bind_group: wgpu::BindGroup =
            init.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: vertex_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: fragment_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: light_uniform_buffer.as_entire_binding(),
                    },
                ],
                label: Some("Uniform Bind Group"),
            });

        let pipeline_layout = init
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout,
                    &camera_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let pipeline = init
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: init.config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                //depth_stencil: None,
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
        let vertex_buffer_vec = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(&vertex_data_vec),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let num_vertices_vec = visible_cubes.len() as u32;
        let index_buffer_vec = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(&(0..visible_cubes.len() as u32).collect::<Vec<u32>>()),
                usage: wgpu::BufferUsages::INDEX,
            });
        // let vertex_buffer_vec = vertex_data_vec
        //     .iter()
        //     .map(|vertex_data| {
        //         init.device
        //             .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //                 label: Some("Vertex Buffer"),
        //                 contents: cast_slice(&vertex_data),
        //                 usage: wgpu::BufferUsages::VERTEX,
        //             })
        //     })
        //     .collect::<Vec<wgpu::Buffer>>();
        // let index_buffer_vec = shape_data
        //     .iter()
        //     .flatten()
        //     .flatten()
        //     .map(|shape_data| {
        //         init.device
        //             .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //                 label: Some("Index Buffer"),
        //                 contents: cast_slice(&shape_data.indices),
        //                 usage: wgpu::BufferUsages::INDEX,
        //             })
        //     })
        //     .collect::<Vec<wgpu::Buffer>>();

        // let num_vertices_vec = shape_data
        //     .iter()
        //     .flatten()
        //     .flatten()
        //     .map(|shape_data| shape_data.indices.len() as u32)
        //     .collect::<Vec<u32>>();

        // let index_buffer_vec = shape_data
        //     .iter()
        //     .map(|shape_data| {
        //         init.device
        //             .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //                 label: Some("Index Buffer"),
        //                 contents: cast_slice(&shape_data.3),
        //                 usage: wgpu::BufferUsages::INDEX,
        //             })
        //     })
        //     .collect::<Vec<wgpu::Buffer>>();
        // let num_vertices_vec = shape_data
        //     .iter()
        //     .map(|shape_data| shape_data.3.len() as u32)
        //     .collect::<Vec<u32>>();

        Self {
            init,
            pipeline,
            vertex_buffer_vec,
            uniform_bind_group,
            vertex_uniform_buffer,
            camera,
            camera_buffer,
            camera_bind_group,
            projection,
            camera_controller,
            camera_uniform,
            mouse_pressed: false,
            num_vertices_vec,
            index_buffer_vec,
            diffuse_bind_group,
            diffuse_texture,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.init.instance.poll_all(true);
            self.init.size = new_size;
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init
                .surface
                .configure(&self.init.device, &self.init.config);
            self.projection.resize(new_size.width, new_size.height);
            // self.project_mat = transforms::create_projection(
            //     new_size.width as f32 / new_size.height as f32,
            //     IS_PERSPECTIVE,
            // );
        }
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        // let dt = ANIMATION_SPEED * dt;
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
        self.init.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let model_mat =
            transforms::create_transforms([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        // let view_project_mat = self.project_mat * self.view_mat;
        let view_project_mat = flatten(self.camera_uniform.view_proj);
        let normal_mat = (model_mat.invert().unwrap()).transpose();

        let model_ref: &[f32; 16] = model_mat.as_ref();
        let view_projection_ref: &[f32; 16] = &view_project_mat;
        let normal_ref: &[f32; 16] = normal_mat.as_ref();

        self.init.queue.write_buffer(
            &self.vertex_uniform_buffer,
            0,
            bytemuck::cast_slice(model_ref),
        );
        self.init.queue.write_buffer(
            &self.vertex_uniform_buffer,
            64,
            bytemuck::cast_slice(view_projection_ref),
        );
        self.init.queue.write_buffer(
            &self.vertex_uniform_buffer,
            128,
            bytemuck::cast_slice(normal_ref),
        );
    }
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        //let output = self.init.surface.get_current_frame()?.output;
        let output = self.init.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = self.init.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.init.config.width,
                height: self.init.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.init
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.247,
                            b: 0.314,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                //depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer_vec.slice(..));
            render_pass
                .set_index_buffer(self.index_buffer_vec.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_vertices_vec, 0, 0..1);
        }

        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
pub fn run(shape_data: &Vec<Vec<Vec<Mesh>>>, light_data: Light) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    window.set_title(&*format!("Parametric 3D Surface"));

    let mut state = pollster::block_on(State::new(&window, &shape_data, light_data));
    let mut render_start_time = std::time::Instant::now();
    let mut frame_count = 0;
    let mut elapsed_time = Duration::new(0, 0);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            let now = std::time::Instant::now();
            let dt = now - render_start_time;
            render_start_time = now;
            elapsed_time += dt;
            frame_count += 1;

            if elapsed_time.as_secs_f32() >= 1.0 {
                let fps = frame_count as f32 / elapsed_time.as_secs_f32();
                println!("FPS: {}", fps);

                // Reset the counter and elapsed time
                frame_count = 0;
                elapsed_time = Duration::new(0, 0);
            }
            state.update(dt);

            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.init.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
