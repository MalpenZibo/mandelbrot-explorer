mod controls;
mod params;
mod scene;
mod uniform;

use std::ops::Add;

use controls::{Controls, Message};
use params::Zoom;
use scene::Scene;

use iced_wgpu::{wgpu, Backend, Renderer, Settings, Viewport};
use iced_winit::{
    conversion, futures, program, renderer,
    winit::{
        self,
        event::{ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode},
    },
    Clipboard, Color, Debug, Size,
};

use winit::{
    dpi::PhysicalPosition,
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowBuilderExtWebSys;

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    let canvas_element = {
        console_log::init_with_level(log::Level::Debug).expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.get_element_by_id("iced_canvas"))
            .and_then(|element| element.dyn_into::<HtmlCanvasElement>().ok())
            .expect("Canvas with id `iced_canvas` is missing")
    };
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    // Initialize winit
    let event_loop = EventLoop::new();

    #[cfg(target_arch = "wasm32")]
    let window = winit::window::WindowBuilder::new()
        .with_canvas(Some(canvas_element))
        .build(&event_loop)
        .expect("Failed to build winit window");

    #[cfg(not(target_arch = "wasm32"))]
    let window = winit::window::Window::new(&event_loop).unwrap();

    let physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
    let mut modifiers = ModifiersState::default();
    let mut clipboard = Clipboard::connect(&window);

    // Initialize wgpu

    #[cfg(target_arch = "wasm32")]
    let default_backend = wgpu::Backends::GL;
    #[cfg(not(target_arch = "wasm32"))]
    let default_backend = wgpu::Backends::PRIMARY;

    let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);

    let instance = wgpu::Instance::new(backend);
    let surface = unsafe { instance.create_surface(&window) };

    let (format, (device, queue)) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No suitable GPU adapters found on the system!");

        let adapter_features = adapter.features();

        #[cfg(target_arch = "wasm32")]
        let needed_limits =
            wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());

        #[cfg(not(target_arch = "wasm32"))]
        let needed_limits = wgpu::Limits::default();

        (
            surface
                .get_supported_formats(&adapter)
                .first()
                .copied()
                .expect("Get preferred format"),
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: adapter_features & wgpu::Features::default(),
                        limits: needed_limits,
                    },
                    None,
                )
                .await
                .expect("Request device"),
        )
    });

    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        },
    );

    let mut resized = false;

    // Initialize staging belt
    let mut staging_belt = wgpu::util::StagingBelt::new(5 * 1024);

    // Initialize scene and GUI controls
    let mut scene = Scene::new(
        &device,
        format,
        [physical_size.width as f32, physical_size.height as f32],
    );
    let controls = Controls::new();

    // Initialize iced
    let mut debug = Debug::new();
    let mut renderer = Renderer::new(Backend::new(&device, Settings::default(), format));

    let mut state =
        program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

    let mut drag = false;
    let mut window_size = physical_size;

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        // You should change this if you want to render continuosly
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        if drag {
                            scene.move_center((
                                cursor_position.x as f32 - position.x as f32,
                                cursor_position.y as f32 - position.y as f32,
                            ));
                        }

                        cursor_position = position;
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    WindowEvent::Resized(size) => {
                        println!("size {size:?}");
                        window_size = size;
                        scene.resize([size.width as f32, size.height as f32]);
                        resized = true;
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        if button == winit::event::MouseButton::Left {
                            drag = state == winit::event::ElementState::Pressed
                                && (cursor_position.y as u32) < window_size.height - 180;
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let y = match delta {
                            MouseScrollDelta::LineDelta(_, y) => y,
                            MouseScrollDelta::PixelDelta(delta) => delta.y as f32,
                        };
                        scene.zoom(
                            if y < 0. { Zoom::Out } else { Zoom::In },
                            Some((cursor_position.x as f32, cursor_position.y as f32)),
                        );
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(code),
                                state: ElementState::Released,
                                ..
                            },
                        ..
                    } => {
                        let increment = if code == VirtualKeyCode::I {
                            1000
                        } else if code == VirtualKeyCode::O {
                            -1000
                        } else {
                            0
                        };

                        if increment != 0 {
                            scene.set_iterations(scene.iterations.add(increment));
                            state.queue_message(Message::IterationsChange(Ok(**scene.iterations)));
                        }
                    }
                    _ => {}
                }

                // Map window event to iced event
                if let Some(event) =
                    iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers)
                {
                    state.queue_event(event);
                }
            }
            Event::MainEventsCleared => {
                // If there are events pending
                if !state.is_queue_empty() {
                    // We update iced
                    let _ = state.update(
                        viewport.logical_size(),
                        conversion::cursor_position(cursor_position, viewport.scale_factor()),
                        &mut renderer,
                        &iced_wgpu::Theme::Dark,
                        &renderer::Style {
                            text_color: Color::WHITE,
                        },
                        &mut clipboard,
                        &mut debug,
                    );

                    // and request a redraw
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if resized {
                    let size = window.inner_size();

                    viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        window.scale_factor(),
                    );

                    surface.configure(
                        &device,
                        &wgpu::SurfaceConfiguration {
                            format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        },
                    );

                    resized = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        let program = state.program();

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            // We clear the frame
                            let mut render_pass = scene.clear(&view, &mut encoder);

                            let color_params = *scene.get_color_params();
                            let (params_hue, params_saturation, param_lightness) =
                                color_params.get_hsl();
                            let (hue, saturation, lightness) = program.color();
                            if params_hue != hue
                                || params_saturation != saturation
                                || param_lightness != lightness
                            {
                                scene.set_hsl((hue, saturation, lightness));
                            }

                            let (hue_link, saturation_link, lightness_link) = program.color_link();
                            let (params_hue_link, params_saturation_link, param_lightness_link) =
                                color_params.get_link();
                            if hue_link != params_hue_link
                                || saturation_link != params_saturation_link
                                || lightness_link != param_lightness_link
                            {
                                scene.set_hsl_link((hue_link, saturation_link, lightness_link));
                            }

                            let iterations = program.iterations();
                            if **scene.iterations != iterations {
                                scene.set_iterations(iterations);
                            }

                            // Draw the scene
                            scene.draw(&queue, &mut render_pass);
                        }

                        // And then iced on top
                        renderer.with_primitives(|backend, primitive| {
                            backend.present(
                                &device,
                                &mut staging_belt,
                                &mut encoder,
                                &view,
                                primitive,
                                &viewport,
                                &debug.overlay(),
                            );
                        });

                        // Then we submit the work
                        staging_belt.finish();
                        queue.submit(Some(encoder.finish()));
                        frame.present();

                        // Update the mouse cursor
                        window.set_cursor_icon(iced_winit::conversion::mouse_interaction(
                            state.mouse_interaction(),
                        ));

                        // And recall staging buffers
                        staging_belt.recall();
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!("Swapchain error: {error}. Rendering cannot continue.")
                        }
                        _ => {
                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }
            }
            _ => {}
        }
    })
}
