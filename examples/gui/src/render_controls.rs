use iced::{
    alignment::Vertical,
    widget::{self, row, text, text_input},
    Background, Color, Element, Theme,
};

use crate::ui_message::Message;

pub struct RenderControls {
    pub img_height: u32,
    pub img_width: u32,
    pub bounces: usize,
    pub passes: usize,

    img_height_valid: bool,
    img_width_valid: bool,
    bounces_valid: bool,
    passes_valid: bool,
}

impl Default for RenderControls {
    fn default() -> Self {
        Self {
            img_height: 500,
            img_width: 500,
            bounces: 50,
            passes: 128,
            img_height_valid: true,
            img_width_valid: true,
            bounces_valid: true,
            passes_valid: true,
        }
    }
}

const RED_BG: Background = Background::Color(Color::from_rgb(1., 0., 0.));

impl RenderControls {
    pub fn view(&self) -> Element<Message> {
        let field_height = text_input("height", &format!("{}", self.img_height))
            .style(|theme: &Theme, status| {
                let mut base_s: widget::text_input::Style =
                    widget::text_input::default(theme, status);
                if !self.img_height_valid {
                    base_s.background = RED_BG;
                }
                base_s
            })
            .on_input(|content| Message::RenderHeightChanged(str::parse(&content).ok()));

        let field_width = text_input("width", &format!("{}", self.img_width))
            .style(|theme: &Theme, status| {
                let mut base_s: widget::text_input::Style =
                    widget::text_input::default(theme, status);
                if !self.img_width_valid {
                    base_s.background = RED_BG;
                }
                base_s
            })
            .on_input(|content| Message::RenderWidthChanged(str::parse(&content).ok()));

        let field_bounces = text_input("bounces", &format!("{}", self.bounces))
            .style(|theme: &Theme, status| {
                let mut base_s: widget::text_input::Style =
                    widget::text_input::default(theme, status);
                if !self.bounces_valid {
                    base_s.background = RED_BG;
                }
                base_s
            })
            .on_input(|content| {
                Message::RenderBouncesChanged(str::parse(&content).ok().filter(|&v| v > 0))
            });

        let field_passes = text_input("passes", &format!("{}", self.passes))
            .style(|theme: &Theme, status| {
                let mut base_s: widget::text_input::Style =
                    widget::text_input::default(theme, status);
                if !self.passes_valid {
                    base_s.background = RED_BG;
                }
                base_s
            })
            .on_input(|content| {
                Message::RenderPassesChanged(str::parse(&content).ok().filter(|&v| v > 0))
            });

        row![
            text("Image height:"),
            field_height,
            text("Image width:"),
            field_width,
            text("Bounces:"),
            field_bounces,
            text("Passes:"),
            field_passes,
        ]
        .align_y(Vertical::Center)
        .spacing(10)
        .padding(10)
        .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::RenderWidthChanged(content) => {
                if let Some(width) = content {
                    self.img_width = width;
                    self.img_width_valid = true;
                } else {
                    self.img_width_valid = false;
                }
            }
            Message::RenderHeightChanged(content) => {
                if let Some(height) = content {
                    self.img_height = height;
                    self.img_height_valid = true;
                } else {
                    self.img_height_valid = false;
                }
            }
            Message::RenderBouncesChanged(content) => {
                if let Some(bounces) = content {
                    self.bounces = bounces;
                    self.bounces_valid = true;
                } else {
                    self.bounces_valid = false;
                }
            }
            Message::RenderPassesChanged(content) => {
                if let Some(passes) = content {
                    self.passes = passes;
                    self.passes_valid = true;
                } else {
                    self.passes_valid = false;
                }
            }
            _ => {}
        }
    }
}
