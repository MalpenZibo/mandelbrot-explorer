use std::ops::Deref;

use bytemuck::Pod;
use iced_wgpu::wgpu::{self, util::DeviceExt};

pub struct Uniform<T> {
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    value: T,
    should_update: bool,
}

impl<T: Pod> Uniform<T> {
    pub fn new(
        name: &str,
        value: T,
        visibility: wgpu::ShaderStages,
        device: &wgpu::Device,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{name} Buffer")),
            contents: bytemuck::cast_slice(&[value]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some(&format!("{name}_bind_group_layout")),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(&format!("{name}_bind_group")),
        });

        Self {
            bind_group,
            bind_group_layout,
            buffer,

            value,
            should_update: false,
        }
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.should_update = true;
    }

    pub fn upload(&mut self, queue: &wgpu::Queue) {
        if self.should_update {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.value]));
            self.should_update = false;
        }
    }
}

impl<T> Deref for Uniform<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
