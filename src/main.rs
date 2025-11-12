// mod controls;
// mod params;
// mod scene;
// mod uniform;
//
// use std::ops::Add;
//
// use controls::{Controls, Message};
// use iced_wgpu::core::{Font, Pixels, Shell};
// use params::Zoom;
// use scene::Scene;
//
// use iced_wgpu::graphics::Viewport;
// use iced_wgpu::{Engine, Renderer, Settings, wgpu};
// use iced_winit::core::Size;
// use iced_winit::{
//     Clipboard, conversion, futures,
//     winit::{
//         self,
//         event::{ElementState, MouseScrollDelta},
//         keyboard::ModifiersState,
//     },
// };
//
// use winit::{
//     dpi::PhysicalPosition,
//     event::{Event, WindowEvent},
//     event_loop::{ControlFlow, EventLoop},
// };
//
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::JsCast;
// #[cfg(target_arch = "wasm32")]
// use web_sys::HtmlCanvasElement;
// #[cfg(target_arch = "wasm32")]
// use winit::platform::web::WindowBuilderExtWebSys;
//
// pub fn main() {
//     #[cfg(target_arch = "wasm32")]
//     let canvas_element = {
//         console_log::init_with_level(log::Level::Debug).expect("could not initialize logger");
//         std::panic::set_hook(Box::new(console_error_panic_hook::hook));
//
//         web_sys::window()
//             .and_then(|win| win.document())
//             .and_then(|doc| doc.get_element_by_id("iced_canvas"))
//             .and_then(|element| element.dyn_into::<HtmlCanvasElement>().ok())
//             .expect("Canvas with id `iced_canvas` is missing")
//     };
//     #[cfg(not(target_arch = "wasm32"))]
//     env_logger::init();
//
//     // Initialize winit
//     let event_loop = EventLoop::new();
//
//     #[cfg(target_arch = "wasm32")]
//     let window = winit::window::WindowBuilder::new()
//         .with_canvas(Some(canvas_element))
//         .build(&event_loop)
//         .expect("Failed to build winit window");
//
//     #[cfg(not(target_arch = "wasm32"))]
//     let window = winit::window::Window::new(&event_loop).unwrap();
//
//     let physical_size = window.inner_size();
//     let mut viewport = Viewport::with_physical_size(
//         Size::new(physical_size.width, physical_size.height),
//         window.scale_factor(),
//     );
//     let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
//     let mut modifiers = ModifiersState::default();
//     let mut clipboard = Clipboard::connect(&window);
//
//     // Initialize wgpu
//
//     #[cfg(target_arch = "wasm32")]
//     let default_backend = wgpu::Backends::GL;
//     #[cfg(not(target_arch = "wasm32"))]
//     let default_backend = wgpu::Backends::PRIMARY;
//
//     let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);
//
//     let instance = wgpu::Instance::new(backend);
//     let surface = unsafe { instance.create_surface(&window) };
//
//     let (format, adapter, device, queue) = futures::futures::executor::block_on(async {
//         let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
//             .await
//             .expect("Create adapter");
//
//         let adapter_features = adapter.features();
//
//         let capabilities = surface.get_capabilities(&adapter);
//
//         let (device, queue) = adapter
//             .request_device(&wgpu::DeviceDescriptor {
//                 label: None,
//                 required_features: adapter_features & wgpu::Features::default(),
//                 required_limits: wgpu::Limits::default(),
//                 memory_hints: wgpu::MemoryHints::MemoryUsage,
//                 trace: wgpu::Trace::Off,
//                 experimental_features: wgpu::ExperimentalFeatures::disabled(),
//             })
//             .await
//             .expect("Request device");
//
//         (
//             capabilities
//                 .formats
//                 .iter()
//                 .copied()
//                 .find(wgpu::TextureFormat::is_srgb)
//                 .or_else(|| capabilities.formats.first().copied())
//                 .expect("Get preferred format"),
//             adapter,
//             device,
//             queue,
//         )
//     });
//
//     surface.configure(
//         &device,
//         &wgpu::SurfaceConfiguration {
//             usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//             format,
//             width: physical_size.width,
//             height: physical_size.height,
//             present_mode: wgpu::PresentMode::AutoVsync,
//             alpha_mode: wgpu::CompositeAlphaMode::Auto,
//         },
//     );
//
//     let mut resized = false;
//
//     // Initialize staging belt
//     let mut staging_belt = wgpu::util::StagingBelt::new(5 * 1024);
//
//     // Initialize scene and GUI controls
//     let mut scene = Scene::new(
//         &device,
//         format,
//         [physical_size.width as f32, physical_size.height as f32],
//     );
//     let controls = Controls::new();
//
//     // Initialize iced
//     // let mut debug = Debug::new();
//     // let mut renderer = Renderer::new(Backend::new(&device, Settings::default(), format));
//
//     let renderer = {
//         let engine = Engine::new(
//             &adapter,
//             device.clone(),
//             queue.clone(),
//             format,
//             None,
//             Shell::headless(),
//         );
//
//         Renderer::new(engine, Font::default(), Pixels::from(16))
//     };
//
//     let mut state =
//         program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);
//
//     let mut drag = false;
//     let mut window_size = physical_size;
//
//     // Run event loop
//     event_loop.run(move |event, _, control_flow| {
//         // You should change this if you want to render continuosly
//         *control_flow = ControlFlow::Wait;
//
//         match event {
//             Event::WindowEvent { event, .. } => {
//                 match event {
//                     WindowEvent::CursorMoved { position, .. } => {
//                         if drag {
//                             scene.move_center((
//                                 cursor_position.x as f32 - position.x as f32,
//                                 cursor_position.y as f32 - position.y as f32,
//                             ));
//                         }
//
//                         cursor_position = position;
//                     }
//                     WindowEvent::ModifiersChanged(new_modifiers) => {
//                         modifiers = new_modifiers;
//                     }
//                     WindowEvent::Resized(size) => {
//                         println!("size {size:?}");
//                         window_size = size;
//                         scene.resize([size.width as f32, size.height as f32]);
//                         resized = true;
//                     }
//                     WindowEvent::CloseRequested => {
//                         *control_flow = ControlFlow::Exit;
//                     }
//                     WindowEvent::MouseInput { button, state, .. } => {
//                         if button == winit::event::MouseButton::Left {
//                             drag = state == winit::event::ElementState::Pressed
//                                 && (cursor_position.y as u32) < window_size.height - 180;
//                         }
//                     }
//                     WindowEvent::MouseWheel { delta, .. } => {
//                         let y = match delta {
//                             MouseScrollDelta::LineDelta(_, y) => y,
//                             MouseScrollDelta::PixelDelta(delta) => delta.y as f32,
//                         };
//                         scene.zoom(
//                             if y < 0. { Zoom::Out } else { Zoom::In },
//                             Some((cursor_position.x as f32, cursor_position.y as f32)),
//                         );
//                     }
//                     WindowEvent::KeyboardInput {
//                         input:
//                             KeyboardInput {
//                                 virtual_keycode: Some(code),
//                                 state: ElementState::Released,
//                                 ..
//                             },
//                         ..
//                     } => {
//                         let increment = if code == VirtualKeyCode::I {
//                             1000
//                         } else if code == VirtualKeyCode::O {
//                             -1000
//                         } else {
//                             0
//                         };
//
//                         if increment != 0 {
//                             scene.set_iterations(scene.iterations.add(increment));
//                             state.queue_message(Message::IterationsChange(Ok(**scene.iterations)));
//                         }
//                     }
//                     _ => {}
//                 }
//
//                 // Map window event to iced event
//                 if let Some(event) =
//                     iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers)
//                 {
//                     state.queue_event(event);
//                 }
//             }
//             Event::MainEventsCleared => {
//                 // If there are events pending
//                 if !state.is_queue_empty() {
//                     // We update iced
//                     let _ = state.update(
//                         viewport.logical_size(),
//                         conversion::cursor_position(cursor_position, viewport.scale_factor()),
//                         &mut renderer,
//                         &iced_wgpu::Theme::Dark,
//                         &renderer::Style {
//                             text_color: Color::WHITE,
//                         },
//                         &mut clipboard,
//                         &mut debug,
//                     );
//
//                     // and request a redraw
//                     window.request_redraw();
//                 }
//             }
//             Event::RedrawRequested(_) => {
//                 if resized {
//                     let size = window.inner_size();
//
//                     viewport = Viewport::with_physical_size(
//                         Size::new(size.width, size.height),
//                         window.scale_factor(),
//                     );
//
//                     surface.configure(
//                         &device,
//                         &wgpu::SurfaceConfiguration {
//                             format,
//                             usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//                             width: size.width,
//                             height: size.height,
//                             present_mode: wgpu::PresentMode::AutoVsync,
//                             alpha_mode: wgpu::CompositeAlphaMode::Auto,
//                         },
//                     );
//
//                     resized = false;
//                 }
//
//                 match surface.get_current_texture() {
//                     Ok(frame) => {
//                         let mut encoder =
//                             device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                                 label: None,
//                             });
//
//                         let program = state.program();
//
//                         let view = frame
//                             .texture
//                             .create_view(&wgpu::TextureViewDescriptor::default());
//
//                         {
//                             // We clear the frame
//                             let mut render_pass = scene.clear(&view, &mut encoder);
//
//                             let color_params = *scene.get_color_params();
//                             let (params_hue, params_saturation, param_lightness) =
//                                 color_params.get_hsl();
//                             let (hue, saturation, lightness) = program.color();
//                             if params_hue != hue
//                                 || params_saturation != saturation
//                                 || param_lightness != lightness
//                             {
//                                 scene.set_hsl((hue, saturation, lightness));
//                             }
//
//                             let (hue_link, saturation_link, lightness_link) = program.color_link();
//                             let (params_hue_link, params_saturation_link, param_lightness_link) =
//                                 color_params.get_link();
//                             if hue_link != params_hue_link
//                                 || saturation_link != params_saturation_link
//                                 || lightness_link != param_lightness_link
//                             {
//                                 scene.set_hsl_link((hue_link, saturation_link, lightness_link));
//                             }
//
//                             let iterations = program.iterations();
//                             if **scene.iterations != iterations {
//                                 scene.set_iterations(iterations);
//                             }
//
//                             // Draw the scene
//                             scene.draw(&queue, &mut render_pass);
//                         }
//
//                         // And then iced on top
//                         renderer.with_primitives(|backend, primitive| {
//                             backend.present(
//                                 &device,
//                                 &mut staging_belt,
//                                 &mut encoder,
//                                 &view,
//                                 primitive,
//                                 &viewport,
//                                 &debug.overlay(),
//                             );
//                         });
//
//                         // Then we submit the work
//                         staging_belt.finish();
//                         queue.submit(Some(encoder.finish()));
//                         frame.present();
//
//                         // Update the mouse cursor
//                         window.set_cursor_icon(iced_winit::conversion::mouse_interaction(
//                             state.mouse_interaction(),
//                         ));
//
//                         // And recall staging buffers
//                         staging_belt.recall();
//                     }
//                     Err(error) => match error {
//                         wgpu::SurfaceError::OutOfMemory => {
//                             panic!("Swapchain error: {error}. Rendering cannot continue.")
//                         }
//                         _ => {
//                             // Try rendering again next frame.
//                             window.request_redraw();
//                         }
//                     },
//                 }
//             }
//             _ => {}
//         }
//     })
// }

mod controls;
mod params;
mod scene;
mod uniform;

use controls::Controls;
use iced_winit::winit::dpi::{PhysicalPosition, PhysicalSize};
use iced_winit::winit::event::{ElementState, KeyEvent, MouseScrollDelta};
use iced_winit::winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use scene::Scene;

use iced_wgpu::graphics::{Shell, Viewport};
use iced_wgpu::{Engine, Renderer, wgpu};
use iced_winit::Clipboard;
use iced_winit::conversion;
use iced_winit::core::mouse;
use iced_winit::core::renderer;
use iced_winit::core::time::Instant;
use iced_winit::core::window;
use iced_winit::core::{Event, Font, Pixels, Size, Theme};
use iced_winit::futures;
use iced_winit::runtime::user_interface::{self, UserInterface};
use iced_winit::winit;

use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::ModifiersState,
};

use std::sync::Arc;

use crate::controls::Message;
use crate::params::Zoom;

pub fn main() -> Result<(), winit::error::EventLoopError> {
    // tracing_subscriber::fmt::init();

    // Initialize winit
    let event_loop = EventLoop::new()?;

    #[allow(clippy::large_enum_variant)]
    enum Runner {
        Loading,
        Ready {
            window: Arc<winit::window::Window>,
            queue: wgpu::Queue,
            device: wgpu::Device,
            surface: wgpu::Surface<'static>,
            format: wgpu::TextureFormat,
            renderer: Renderer,
            scene: Scene,
            controls: Controls,
            events: Vec<Event>,
            cursor: mouse::Cursor,
            cache: user_interface::Cache,
            clipboard: Clipboard,
            viewport: Viewport,
            modifiers: ModifiersState,
            resized: bool,
            drag: bool,
            physical_size: PhysicalSize<u32>,
        },
    }

    impl winit::application::ApplicationHandler for Runner {
        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            if let Self::Loading = self {
                let window = Arc::new(
                    event_loop
                        .create_window(winit::window::WindowAttributes::default())
                        .expect("Create window"),
                );

                let physical_size = window.inner_size();
                let viewport = Viewport::with_physical_size(
                    Size::new(physical_size.width, physical_size.height),
                    window.scale_factor() as f32,
                );
                let clipboard = Clipboard::connect(window.clone());

                let backend = wgpu::Backends::from_env().unwrap_or_default();

                let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                    backends: backend,
                    ..Default::default()
                });
                let surface = instance
                    .create_surface(window.clone())
                    .expect("Create window surface");

                let (format, adapter, device, queue) =
                    futures::futures::executor::block_on(async {
                        let adapter = wgpu::util::initialize_adapter_from_env_or_default(
                            &instance,
                            Some(&surface),
                        )
                        .await
                        .expect("Create adapter");

                        let adapter_features = adapter.features();

                        let capabilities = surface.get_capabilities(&adapter);

                        let (device, queue) = adapter
                            .request_device(&wgpu::DeviceDescriptor {
                                label: None,
                                required_features: adapter_features & wgpu::Features::default(),
                                required_limits: wgpu::Limits::default(),
                                memory_hints: wgpu::MemoryHints::MemoryUsage,
                                trace: wgpu::Trace::Off,
                                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                            })
                            .await
                            .expect("Request device");

                        (
                            capabilities
                                .formats
                                .iter()
                                .copied()
                                .find(wgpu::TextureFormat::is_srgb)
                                .or_else(|| capabilities.formats.first().copied())
                                .expect("Get preferred format"),
                            adapter,
                            device,
                            queue,
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
                        view_formats: vec![],
                        desired_maximum_frame_latency: 2,
                    },
                );

                // Initialize scene and GUI controls
                let scene = Scene::new(
                    &device,
                    format,
                    [physical_size.width as f32, physical_size.height as f32],
                );

                let controls = Controls::new();

                // Initialize iced

                let renderer = {
                    let engine = Engine::new(
                        &adapter,
                        device.clone(),
                        queue.clone(),
                        format,
                        None,
                        Shell::headless(),
                    );

                    Renderer::new(engine, Font::default(), Pixels::from(16))
                };

                // You should change this if you want to render continuously
                event_loop.set_control_flow(ControlFlow::Wait);

                *self = Self::Ready {
                    window,
                    device,
                    queue,
                    renderer,
                    surface,
                    format,
                    scene,
                    controls,
                    events: Vec::new(),
                    cursor: mouse::Cursor::Unavailable,
                    modifiers: ModifiersState::default(),
                    cache: user_interface::Cache::new(),
                    clipboard,
                    viewport,
                    resized: false,
                    drag: false,
                    physical_size,
                };
            }
        }

        fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
            let Self::Ready {
                window,
                device,
                queue,
                surface,
                format,
                renderer,
                scene,
                controls,
                events,
                viewport,
                cursor,
                modifiers,
                clipboard,
                cache,
                resized,
                drag,
                physical_size,
            } = self
            else {
                return;
            };

            let mut window_size = physical_size;

            match event {
                WindowEvent::RedrawRequested => {
                    if *resized {
                        let size = window.inner_size();

                        *viewport = Viewport::with_physical_size(
                            Size::new(size.width, size.height),
                            window.scale_factor() as f32,
                        );

                        surface.configure(
                            device,
                            &wgpu::SurfaceConfiguration {
                                format: *format,
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                width: size.width,
                                height: size.height,
                                present_mode: wgpu::PresentMode::AutoVsync,
                                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                                view_formats: vec![],
                                desired_maximum_frame_latency: 2,
                            },
                        );

                        scene.resize([size.width as f32, size.height as f32]);

                        *resized = false;
                    }

                    match surface.get_current_texture() {
                        Ok(frame) => {
                            let view = frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());

                            let mut encoder =
                                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: None,
                                });

                            {
                                // Clear the frame
                                let mut render_pass =
                                    Scene::clear(&view, &mut encoder, controls.background_color());

                                let color_params = *scene.get_color_params();
                                let (params_hue, params_saturation, param_lightness) =
                                    color_params.get_hsl();
                                let (hue, saturation, lightness) = controls.color;
                                if params_hue != hue
                                    || params_saturation != saturation
                                    || param_lightness != lightness
                                {
                                    scene.set_hsl((hue, saturation, lightness));
                                }

                                let (hue_link, saturation_link, lightness_link) =
                                    controls.color_linked;
                                let (params_hue_link, params_saturation_link, param_lightness_link) =
                                    color_params.get_link();
                                if hue_link != params_hue_link
                                    || saturation_link != params_saturation_link
                                    || lightness_link != param_lightness_link
                                {
                                    scene.set_hsl_link((hue_link, saturation_link, lightness_link));
                                }

                                let iterations = controls.iterations;
                                if **scene.iterations != iterations {
                                    scene.set_iterations(iterations);
                                }

                                // Draw the scene
                                scene.draw(&mut render_pass, queue);
                            }

                            // Submit the scene
                            queue.submit([encoder.finish()]);

                            // Draw iced on top
                            let mut interface = UserInterface::build(
                                controls.view(),
                                viewport.logical_size(),
                                std::mem::take(cache),
                                renderer,
                            );

                            let (state, _) = interface.update(
                                &[Event::Window(
                                    window::Event::RedrawRequested(Instant::now()),
                                )],
                                *cursor,
                                renderer,
                                clipboard,
                                &mut Vec::new(),
                            );

                            // Update the mouse cursor
                            if let user_interface::State::Updated {
                                mouse_interaction, ..
                            } = state
                            {
                                window.set_cursor(conversion::mouse_interaction(mouse_interaction));
                            }

                            // Draw the interface
                            interface.draw(
                                renderer,
                                &Theme::Dark,
                                &renderer::Style::default(),
                                *cursor,
                            );
                            *cache = interface.into_cache();

                            renderer.present(None, frame.texture.format(), &view, viewport);

                            // Present the frame
                            frame.present();
                        }
                        Err(error) => match error {
                            wgpu::SurfaceError::OutOfMemory => {
                                panic!(
                                    "Swapchain error: {error}. \
                                        Rendering cannot continue."
                                )
                            }
                            _ => {
                                // Try rendering again next frame.
                                window.request_redraw();
                            }
                        },
                    }
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == winit::event::MouseButton::Left
                        && let Some(position) = cursor.position()
                    {
                        println!("position: {} - window_size: {:?}", position, window_size);
                        *drag = state == winit::event::ElementState::Pressed
                            && (position.y as u32) < window_size.height - 180;
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if let Some(position) = cursor.position() {
                        let y = match delta {
                            MouseScrollDelta::LineDelta(_, y) => y,
                            MouseScrollDelta::PixelDelta(delta) => delta.y as f32,
                        };
                        scene.zoom(
                            if y < 0. { Zoom::Out } else { Zoom::In },
                            Some((position.x as f32, position.y as f32)),
                        );
                    }
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: ref logical_key,
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => {
                    let increment = if *logical_key == Key::Character("i".into()) {
                        1000
                    } else if *logical_key == Key::Character("o".into()) {
                        -1000
                    } else {
                        0
                    };

                    if increment != 0 {
                        scene.set_iterations(scene.iterations.wrapping_add(increment));
                        controls.update(Message::IterationsChange(Ok(**scene.iterations)));
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if *drag && let Some(cur_position) = cursor.position() {
                        println!("cursor {:?}", cursor);
                        // println!("viewport {:?}", viewport);
                        println!("position {:?}", position);
                        scene.move_center((
                            (cur_position.x * viewport.scale_factor() - position.x as f32),
                            (cur_position.y * viewport.scale_factor() - position.y as f32),
                        ));
                    }

                    *cursor = mouse::Cursor::Available(conversion::cursor_position(
                        position,
                        viewport.scale_factor(),
                    ));
                }
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    *modifiers = new_modifiers.state();
                }
                WindowEvent::Resized(_) => {
                    *resized = true;
                }
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                _ => {}
            }

            // Map window event to iced event
            if let Some(event) =
                conversion::window_event(event, window.scale_factor() as f32, *modifiers)
            {
                events.push(event);
            }

            // If there are events pending
            if !events.is_empty() {
                // We process them
                let mut interface = UserInterface::build(
                    controls.view(),
                    viewport.logical_size(),
                    std::mem::take(cache),
                    renderer,
                );

                let mut messages = Vec::new();

                let _ = interface.update(events, *cursor, renderer, clipboard, &mut messages);

                events.clear();
                *cache = interface.into_cache();

                // update our UI with any messages
                for message in messages {
                    controls.update(message);
                }

                // and request a redraw
                window.request_redraw();
            }
        }
    }

    let mut runner = Runner::Loading;
    event_loop.run_app(&mut runner)
}
