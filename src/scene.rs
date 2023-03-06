use iced_wgpu::wgpu::{self, BindGroupLayout, ShaderStages};

use crate::{
    params::{ColorParams, Coordinates, Iterations, Viewport, Zoom},
    uniform::Uniform,
};

pub struct Scene {
    pipeline: wgpu::RenderPipeline,
    viewport: Uniform<Viewport>,
    coordinates: Uniform<Coordinates>,
    pub iterations: Uniform<Iterations>,
    color_params: Uniform<ColorParams>,
}

impl Scene {
    pub fn new(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
        window_size: [f32; 2],
    ) -> Scene {
        let viewport = Uniform::new(
            "Viewport",
            Viewport::new(window_size),
            ShaderStages::FRAGMENT,
            device,
        );
        let coordinates = Uniform::new(
            "Coordinates",
            Coordinates::default(),
            ShaderStages::FRAGMENT,
            device,
        );
        let iterations = Uniform::new(
            "Iterations",
            Iterations::new(1000),
            ShaderStages::FRAGMENT,
            device,
        );
        let color_params = Uniform::new(
            "ColorParams",
            ColorParams::new(0.5, 1.0, 1.0),
            ShaderStages::FRAGMENT,
            device,
        );
        let pipeline = build_pipeline(
            device,
            texture_format,
            viewport.get_bind_group_layout(),
            coordinates.get_bind_group_layout(),
            iterations.get_bind_group_layout(),
            color_params.get_bind_group_layout(),
        );

        Scene {
            pipeline,
            viewport,
            coordinates,
            iterations,
            color_params,
        }
    }

    pub fn clear<'a>(
        &self,
        target: &'a wgpu::TextureView,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(iced_wgpu::wgpu::Color::WHITE),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }

    pub fn resize(&mut self, window_size: [f32; 2]) {
        self.viewport.set(Viewport::new(window_size));
    }

    pub fn move_center(&mut self, motion: (f32, f32)) {
        let change_x = motion.0 / self.viewport.half_viewport_x * 2. * self.coordinates.get_zoom();
        let change_y = motion.1 / self.viewport.half_viewport_y * 2. * self.coordinates.get_zoom();

        let (real, imag) = self.coordinates.get_complex();

        let new_real = real + change_x;
        let new_imag = imag + change_y;

        self.coordinates
            .set(self.coordinates.set_complex((new_real, new_imag)));
    }

    pub fn zoom(&mut self, zoom: Zoom, cursor_pos: Option<(f32, f32)>) {
        self.coordinates.set(self.coordinates.set_zoom(
            zoom,
            cursor_pos.map(|c| {
                let ratio = self.viewport.ratio;

                let cur_rel_x =
                    (c.0 - self.viewport.half_viewport_x) / self.viewport.half_viewport_x * ratio;
                let cur_rel_y =
                    (c.1 - self.viewport.half_viewport_y) / self.viewport.half_viewport_y * -1.;

                (cur_rel_x, cur_rel_y)
            }),
        ));
    }

    pub fn set_iterations(&mut self, iterations: i32) {
        self.iterations.set(Iterations::new(iterations));
    }

    pub fn get_color_params(&self) -> &ColorParams {
        &self.color_params
    }

    pub fn set_hsl(&mut self, hsl: (f32, f32, f32)) {
        self.color_params.set(self.color_params.set_hsl(hsl));
    }

    pub fn set_hsl_link(&mut self, hsl_link: (bool, bool, bool)) {
        self.color_params.set(self.color_params.set_link(hsl_link));
    }

    pub fn draw<'a>(&'a mut self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass<'a>) {
        self.viewport.upload(queue);
        self.coordinates.upload(queue);
        self.iterations.upload(queue);
        self.color_params.upload(queue);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.viewport.get_bind_group(), &[]);
        render_pass.set_bind_group(1, self.coordinates.get_bind_group(), &[]);
        render_pass.set_bind_group(2, self.iterations.get_bind_group(), &[]);
        render_pass.set_bind_group(3, self.color_params.get_bind_group(), &[]);
        render_pass.draw(0..6, 0..1);
    }
}

fn build_pipeline(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
    viewport: &BindGroupLayout,
    coordinates: &BindGroupLayout,
    iterations: &BindGroupLayout,
    color_params: &BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        push_constant_ranges: &[],
        bind_group_layouts: &[viewport, coordinates, iterations, color_params],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: texture_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
