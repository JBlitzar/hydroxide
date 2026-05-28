use std::sync::Arc;

use crate::geometry::Hittable;
pub mod upload;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectKey(usize);
impl ObjectKey {
    pub fn of(obj: &Arc<dyn Hittable>) -> Self {
        Self(Arc::as_ptr(obj) as *const () as usize)
    }
}

use crate::camera::Camera;
use crate::gpu::upload::{GpuLight, GpuPlane, GpuUniforms};
use bytemuck::Zeroable;
use crate::world::World;
use wgpu::util::DeviceExt;

pub struct GpuRenderer {
    // Cached copy of the uniform data so we can modify only the camera fields later
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pathtrace_pipeline: wgpu::ComputePipeline,
    tonemap_pipeline: wgpu::ComputePipeline,
    bind_groups: Option<Vec<wgpu::BindGroup>>,
    accum_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    // Keep a CPU‑side copy of the uniform block
    uniform_data: GpuUniforms,
    width: u32,
    height: u32,
    pathtrace_bind_group_layout_0: wgpu::BindGroupLayout,
    pathtrace_bind_group_layout_1: wgpu::BindGroupLayout,
    pathtrace_bind_group_layout_2: wgpu::BindGroupLayout,
}

impl GpuRenderer {
    pub async fn new(width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .expect("Failed to find wgpu adapter");

        // Query the adapter's limits and set the required limits accordingly
        let adapter_limits = adapter.limits();
        let mut required_limits = wgpu::Limits::default();
        required_limits.max_storage_buffers_per_shader_stage =
            adapter_limits.max_storage_buffers_per_shader_stage;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Hydroxide Device"),
                required_features: wgpu::Features::empty(),
                required_limits,
                ..Default::default()
            })
            .await
            .expect("Failed to create wgpu device");

        let pathtrace_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PathTrace Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("pathtrace.wgsl").into()),
        });

        let tonemap_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tonemap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/tonemap.wgsl").into()),
        });

        let pathtrace_bind_group_layout_0 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("pathtrace_bind_group_layout_0"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pathtrace_bind_group_layout_1 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("pathtrace_bind_group_layout_1"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pathtrace_bind_group_layout_2 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("pathtrace_bind_group_layout_2"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pathtrace_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("PathTrace Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&pathtrace_bind_group_layout_0),
                    Some(&pathtrace_bind_group_layout_1),
                    Some(&pathtrace_bind_group_layout_2),
                ],
                immediate_size: 0,
            });

        let pathtrace_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("PathTrace Pipeline"),
            layout: Some(&pathtrace_pipeline_layout),
            module: &pathtrace_shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let tonemap_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Tonemap Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&pathtrace_bind_group_layout_0),
                    Some(&pathtrace_bind_group_layout_2),
                ],
                immediate_size: 0,
            });

        let tonemap_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Tonemap Pipeline"),
            layout: Some(&tonemap_pipeline_layout),
            module: &tonemap_shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let pixel_count = (width * height) as u64;
        let accum_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Accumulation Buffer"),
            size: pixel_count * 16, // vec4f
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: pixel_count * 4, // u32
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: pixel_count * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<upload::GpuUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device,
            queue,
            pathtrace_pipeline,
            tonemap_pipeline,
            bind_groups: None,
            accum_buffer,
            output_buffer,
            staging_buffer,
            uniform_buffer,
            width,
            height,
            pathtrace_bind_group_layout_0,
            pathtrace_bind_group_layout_1,
            pathtrace_bind_group_layout_2,
            uniform_data: GpuUniforms::zeroed(),
        }
    }

    pub fn upload_scene(
        &mut self,
        world: &World,
        camera: &Camera,
        spp: u32,
        max_depth: u32,
        termination_prob: f64,
    ) {
        let packed = upload::pack_scene(world, camera, spp, max_depth, termination_prob);

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&packed.uniforms),
        );

        let create_storage = |label: &str, data: &[u8]| -> wgpu::Buffer {
            if data.is_empty() {
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size: 4,
                    usage: wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                })
            } else {
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(label),
                        contents: data,
                        usage: wgpu::BufferUsages::STORAGE,
                    })
            }
        };

        let b_spheres = create_storage("spheres", bytemuck::cast_slice(&packed.spheres));
        let b_top_bvh = create_storage("top_bvh", bytemuck::cast_slice(&packed.top_bvh));
        let b_mesh_bvh = create_storage("mesh_bvh", bytemuck::cast_slice(&packed.mesh_bvh));
        let b_tris = create_storage("tris", bytemuck::cast_slice(&packed.triangles));
        let b_mesh_info = create_storage("mesh_info", bytemuck::cast_slice(&packed.mesh_info));
        let b_planes = if packed.planes.is_empty() {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("planes"),
                size: std::mem::size_of::<GpuPlane>() as u64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        } else {
            create_storage("planes", bytemuck::cast_slice(&packed.planes))
        };
        let b_lights = if packed.lights.is_empty() {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("lights"),
                size: std::mem::size_of::<GpuLight>() as u64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        } else {
            create_storage("lights", bytemuck::cast_slice(&packed.lights))
        };

        let b_hdr_sky = if packed.hdr_sky.is_empty() {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("hdr_sky"),
                size: std::mem::size_of::<[f32; 4]>() as u64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        } else {
            create_storage("hdr_sky", bytemuck::cast_slice(&packed.hdr_sky))
        };

        let bg0 = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg0"),
            layout: &self.pathtrace_bind_group_layout_0,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.as_entire_binding(),
            }],
        });

        let bg1 = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg1"),
            layout: &self.pathtrace_bind_group_layout_1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: b_spheres.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: b_top_bvh.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: b_mesh_bvh.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: b_tris.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: b_mesh_info.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: b_planes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: b_lights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: b_hdr_sky.as_entire_binding(),
                },
            ],
        });

        let bg2 = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg2"),
            layout: &self.pathtrace_bind_group_layout_2,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.accum_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.output_buffer.as_entire_binding(),
                },
            ],
        });

        // Zero out accumulation buffer
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.clear_buffer(&self.accum_buffer, 0, None);
        self.queue.submit(Some(encoder.finish()));

        // Cache the uniform data so we can update only the camera later
        self.uniform_data = packed.uniforms;
        self.bind_groups = Some(vec![bg0, bg1, bg2]);
    }

    /// Update only the camera portion of the uniform buffer.
    pub fn update_camera(&mut self, camera: &Camera) {
        // Update cached copy of the uniform fields related to the camera.
        self.uniform_data.cam_pos = camera.position.to_f32_array();
        self.uniform_data.cam_forward = camera.forward.to_f32_array();
        self.uniform_data.cam_right = camera.right.to_f32_array();
        self.uniform_data.cam_up = camera.up.to_f32_array();
        self.uniform_data.half_tan_fov_x = camera.half_tan_fov_x as f32;
        self.uniform_data.half_tan_fov_y = camera.half_tan_fov_y as f32;
        self.uniform_data.focus_distance = camera.focus_distance as f32;
        self.uniform_data.aperture = camera.aperture as f32;
        // Write the whole uniform block (still cheap) – you could also write a sub‑range.
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&self.uniform_data),
        );
    }

    /// Convenience method: set a new camera and clear the accumulation buffer.
    pub fn set_camera(&mut self, camera: &Camera) {
        // Reset accumulation buffer so the new view starts clean.
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.clear_buffer(&self.accum_buffer, 0, None);
        self.queue.submit(Some(encoder.finish()));
        self.update_camera(camera);
    }

    pub fn render(&self, spp: u32) {
        let bgs = self.bind_groups.as_ref().expect("Scene not uploaded");

        let wg_x = (self.width + 7) / 8;
        let wg_y = (self.height + 7) / 8;

        #[cfg(feature = "native")]
        let pb = indicatif::ProgressBar::new(spp as u64);

        for sample in 0..spp {
            let offset = 3 * 4; // offsetof current_sample
            self.queue
                .write_buffer(&self.uniform_buffer, offset, bytemuck::bytes_of(&sample));

            let mut encoder = self.device.create_command_encoder(&Default::default());
            {
                let mut pass = encoder.begin_compute_pass(&Default::default());
                pass.set_pipeline(&self.pathtrace_pipeline);
                pass.set_bind_group(0, &bgs[0], &[]);
                pass.set_bind_group(1, &bgs[1], &[]);
                pass.set_bind_group(2, &bgs[2], &[]);
                pass.dispatch_workgroups(wg_x, wg_y, 1);
            }
            self.queue.submit(Some(encoder.finish()));

            #[cfg(feature = "native")]
            pb.inc(1);
        }

        #[cfg(feature = "native")]
        pb.finish_and_clear();

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.tonemap_pipeline);
            pass.set_bind_group(0, &bgs[0], &[]);
            pass.set_bind_group(1, &bgs[2], &[]);
            pass.dispatch_workgroups(wg_x, wg_y, 1);
        }

        let size = (self.width * self.height * 4) as u64;
        encoder.copy_buffer_to_buffer(&self.output_buffer, 0, &self.staging_buffer, 0, size);
        self.queue.submit(Some(encoder.finish()));
    }

    pub async fn read_pixels(&self) -> Vec<u8> {
        let slice = self.staging_buffer.slice(..);
        let (tx, rx) = futures_channel::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).ok();
        });
        self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        rx.await.unwrap().unwrap();

        let data = slice.get_mapped_range();
        let result = data.to_vec();
        drop(data);
        self.staging_buffer.unmap();
        result
    }
}
