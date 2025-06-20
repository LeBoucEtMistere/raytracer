use std::time::{Duration, Instant};

use iced::{
    widget::{progress_bar, row, text},
    Element,
};
use pretty_duration::pretty_duration;

use crate::ui_message::Message;

pub struct ProgressData {
    pub current_pass: usize,
    pub total_passes: usize,
    pub started_at: Instant,
}

#[derive(Default)]
pub enum RenderProgress {
    #[default]
    Idle,
    Aborted,
    Finished {
        time_taken: Duration,
    },
    InProgress(ProgressData),
}

impl RenderProgress {
    pub fn view(&self) -> Element<Message> {
        match &self {
            Self::Idle => text("Render not started").into(),
            Self::Aborted => text("Render stopped").into(),
            Self::InProgress(pd) => row![
                text("Rendering..."),
                progress_bar(0.0..=pd.total_passes as f32, pd.current_pass as f32),
                text(format!("{}/{}", pd.current_pass, pd.total_passes)),
            ]
            .spacing(10)
            .into(),
            Self::Finished { time_taken } => text(format!(
                "Render finished in {}",
                pretty_duration(time_taken, None)
            ))
            .into(),
        }
    }
}
