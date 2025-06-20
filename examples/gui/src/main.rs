mod raytracing_worker;
mod render_controls;
mod render_progress;
mod ui_message;

use std::error::Error;
use std::fs;
use std::time::Instant;

use iced::advanced::graphics::image::image_rs::EncodableLayout;
use iced::advanced::image::Handle;
use iced::futures::channel::mpsc::Sender;
use iced::widget::{button, center, column, image, row};

use bytes::BytesMut;
use iced::Length::Fill;
use iced::{window, Element, Subscription, Task};
use nalgebra_glm::Vec3;
use raytracing_lib::export::MemWriter;
use raytracing_lib::{Camera, FocusData, MaterialAtlas, World};

use crate::raytracing_worker::{RenderRequest, WorkerMessage};
use crate::render_controls::RenderControls;
use crate::render_progress::{ProgressData, RenderProgress};
use crate::ui_message::Message;

struct Daemon {
    buf: BytesMut,
    raytracing_worker_tx: Option<Sender<WorkerMessage>>,
    render_progress: RenderProgress,
    render_controls: RenderControls,
}

impl Daemon {
    fn new() -> (Self, Task<Message>) {
        let (_id, open) = window::open(window::Settings::default());

        (
            Self {
                buf: BytesMut::from(
                    Vec::from_iter(
                        [0u8, 0u8, 0u8, 255u8]
                            .into_iter()
                            .cycle()
                            .take(500 * 500 * 4),
                    )
                    .as_bytes(),
                ),
                raytracing_worker_tx: None,
                render_progress: Default::default(),
                render_controls: Default::default(),
            },
            open.map(|_| Message::WindowOpened),
        )
    }

    fn title(&self, _window: window::Id) -> String {
        String::from("Realtime Raytracer")
    }

    fn view(&self, _window_id: window::Id) -> Element<Message> {
        let btn = if let RenderProgress::InProgress(_) = self.render_progress {
            button("Stop Render").on_press(Message::StopRender)
        } else {
            button("Start Render").on_press(Message::StartRender)
        };

        column![
            center(
                image(Handle::from_rgba(
                    self.render_controls.img_width,
                    self.render_controls.img_height,
                    BytesMut::clone(&self.buf).freeze(),
                ))
                .width(Fill)
                .height(Fill),
            ),
            self.render_controls.view(),
            row![self.render_progress.view(), btn]
                .padding(10)
                .spacing(5)
        ]
        .spacing(5)
        .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::WorkerEvent(we) => match we {
                raytracing_worker::Event::WorkerInitialized(tx) => {
                    self.raytracing_worker_tx = Some(tx);
                }
                raytracing_worker::Event::RenderStarted(total_passes) => {
                    self.render_progress = RenderProgress::InProgress(ProgressData {
                        current_pass: 0,
                        total_passes,
                        started_at: Instant::now(),
                    })
                }
                raytracing_worker::Event::RenderPassAvailable(render_pass) => {
                    render_pass.canvas.write_rgba_to_buffer(&mut self.buf);
                    if let RenderProgress::InProgress(rd) = &mut self.render_progress {
                        rd.current_pass = render_pass.current_pass
                    }
                }
                raytracing_worker::Event::RenderFinished => {
                    self.render_progress = match &self.render_progress {
                        RenderProgress::InProgress(rp) => RenderProgress::Finished {
                            time_taken: Instant::now().duration_since(rp.started_at),
                        },

                        RenderProgress::Aborted => RenderProgress::Aborted,
                        _ => unreachable!(),
                    }
                }
            },
            Message::StartRender => {
                // init the scene and send a render request

                // Camera
                let look_from = Vec3::new(13.0, 2.0, 3.0);
                let look_at = Vec3::new(0.0, 0.0, 0.0);
                let camera = Camera::builder()
                    .set_origin(look_from)
                    .set_look_at(look_at)
                    .set_v_up(Vec3::new(0.0, 1.0, 0.0))
                    .set_focus(FocusData {
                        aperture: 0.1f32,
                        focus_distance: 10.0,
                    })
                    .set_vertical_fov(20.0)
                    .build();

                let scene: raytracing_lib::scene::serialization::Scene = serde_yaml::from_str(
                    &fs::read_to_string("examples/from_scene/scene.yaml").unwrap(),
                )
                .unwrap();

                let (material_atlas, world) =
                    std::convert::TryInto::<(MaterialAtlas, World)>::try_into(scene).unwrap();

                if let Some(tx) = &mut self.raytracing_worker_tx {
                    tx.try_send(WorkerMessage::StartRenderRequest(RenderRequest {
                        camera,
                        world,
                        material_atlas,
                        image_width: self.render_controls.img_width as usize,
                        image_height: self.render_controls.img_height as usize,
                        bounces: self.render_controls.bounces,
                        samples: self.render_controls.passes,
                    }))
                    .unwrap();
                }
            }
            Message::CloseWindowEvent => {
                std::process::exit(0);
            }
            Message::WindowOpened => {}
            Message::StopRender => {
                if let Some(tx) = &mut self.raytracing_worker_tx {
                    tx.try_send(WorkerMessage::StopRender).unwrap();
                }
                self.render_progress = RenderProgress::Aborted;
            }
            Message::RenderBouncesChanged(_)
            | Message::RenderPassesChanged(_)
            | Message::RenderWidthChanged(_)
            | Message::RenderHeightChanged(_) => {
                self.render_controls.update(message);
            }
        }
    }

    fn subscriptions(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::window::close_events().map(|_| Message::CloseWindowEvent),
            Subscription::run(raytracing_worker::raytracer_worker).map(Message::WorkerEvent),
        ])
    }
}

pub fn main() -> Result<(), impl Error> {
    iced::daemon(Daemon::title, Daemon::update, Daemon::view)
        .subscription(Daemon::subscriptions)
        .run_with(Daemon::new)
}
