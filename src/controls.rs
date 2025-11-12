use std::num::ParseIntError;

use iced_wgpu::{
    Renderer,
    core::{
        Color, Element, Length, Padding,
        alignment::{Horizontal, Vertical},
    },
};
use iced_widget::{Column, Row, Text, Theme, checkbox, container, slider, text_input};
use iced_winit::runtime::Task;

pub struct Controls {
    pub color: (f32, f32, f32),
    pub color_linked: (bool, bool, bool),
    pub iterations: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    ColorChanged(f32, f32, f32),
    ColorLinkChanged(bool, bool, bool),
    IterationsChange(Result<i32, ParseIntError>),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            color: (1., 1., 1.),
            color_linked: (false, false, false),
            iterations: 1000,
        }
    }

    pub fn background_color(&self) -> Color {
        Color::TRANSPARENT
    }
}

impl Controls {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ColorChanged(hue, saturation, lightness) => {
                self.color = (hue, saturation, lightness);
            }
            Message::ColorLinkChanged(hue_link, saturation_link, lightness_link) => {
                self.color_linked = (hue_link, saturation_link, lightness_link);
            }
            Message::IterationsChange(iterations) => {
                if let Ok(iterations) = iterations {
                    self.iterations = iterations
                }
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message, Theme, Renderer> {
        let color = self.color;
        let color_linked = self.color_linked;
        let iterations = self.iterations;

        let controls = Row::new()
            .width(600)
            .spacing(20)
            .push(
                Column::new()
                    .push(Text::new("Hue").color(Color::WHITE))
                    .push(
                        slider(0.01..=1.0, color.0, move |hue| {
                            Message::ColorChanged(hue, color.1, color.2)
                        })
                        .step(0.01),
                    )
                    .push(checkbox("Link?", color_linked.0).on_toggle(move |link| {
                        Message::ColorLinkChanged(link, color_linked.1, color_linked.2)
                    }))
                    .width(Length::Fill),
            )
            .push(
                Column::new()
                    .push(Text::new("Saturation").color(Color::WHITE))
                    .push(
                        slider(0.0..=1.0, color.1, move |saturation| {
                            Message::ColorChanged(color.0, saturation, color.2)
                        })
                        .step(0.01),
                    )
                    .push(checkbox("Link?", color_linked.1).on_toggle(move |link| {
                        Message::ColorLinkChanged(color_linked.0, link, color_linked.2)
                    }))
                    .width(Length::Fill),
            )
            .push(
                Column::new()
                    .push(Text::new("Lightness").color(Color::WHITE))
                    .push(
                        slider(0.0..=1.0, color.2, move |lightness| {
                            Message::ColorChanged(color.0, color.1, lightness)
                        })
                        .step(0.01),
                    )
                    .push(checkbox("Link?", color_linked.2).on_toggle(move |link| {
                        Message::ColorLinkChanged(color_linked.0, color_linked.1, link)
                    }))
                    .width(Length::Fill),
            )
            .push(
                Column::new()
                    .push(Text::new("Iterations").color(Color::WHITE))
                    .push(text_input("", &iterations.to_string()).on_input(|v| {
                        let parsed = if v.is_empty() {
                            Ok(0)
                        } else {
                            v.parse::<i32>()
                        };
                        Message::IterationsChange(parsed)
                    }))
                    .width(Length::Fill),
            );

        container(
            container(controls)
                .height(Length::Shrink)
                .width(Length::Fill)
                .align_y(Vertical::Center)
                .align_x(Horizontal::Center)
                .padding(Padding::new(12.)),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_y(Vertical::Bottom)
        .into()
    }
}
