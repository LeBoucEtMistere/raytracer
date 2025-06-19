mod raytracing_worker;

use std::error::Error;
use std::fs;

use iced::advanced::graphics::image::image_rs::EncodableLayout;
use iced::advanced::image::Handle;
use iced::futures::channel::mpsc::Sender;
use iced::widget::{button, center, column, image, progress_bar, row};

use bytes::BytesMut;
use iced::Length::Fill;
use iced::{window, Element, Subscription, Task};
use nalgebra_glm::Vec3;
use raytracing_lib::export::MemWriter;
use raytracing_lib::{Camera, FocusData, MaterialAtlas, World};

use crate::raytracing_worker::{RenderRequest, WorkerMessage};

struct ProgressData {
    current_pass: usize,
    total_passes: usize,
}

#[derive(Debug, Clone)]
enum Message {
    WorkerEvent(raytracing_worker::Event),
    CloseWindowEvent,
    WindowOpened,
    StartRender,
    StopRender,
}

struct Daemon {
    x_size: u32,
    y_size: u32,
    buf: BytesMut,
    raytracing_worker_tx: Option<Sender<WorkerMessage>>,
    progress: Option<ProgressData>,
}

impl Daemon {
    fn new(x_size: u32, y_size: u32) -> (Self, Task<Message>) {
        let (_id, open) = window::open(window::Settings::default());

        (
            Self {
                x_size,
                y_size,
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
                progress: None,
            },
            open.map(|_| Message::WindowOpened),
        )
    }

    fn title(&self, _window: window::Id) -> String {
        String::from("Realtime Raytracer")
    }

    fn view(&self, _window_id: window::Id) -> Element<Message> {
        let bottom_row = if let Some(progress) = &self.progress {
            row![
                progress_bar(
                    0.0..=progress.total_passes as f32,
                    progress.current_pass as f32
                ),
                button("Stop Render").on_press(Message::StopRender),
            ]
        } else {
            row![button("Start Render").on_press(Message::StartRender),]
        };
        column![
            center(
                image(Handle::from_rgba(
                    self.x_size,
                    self.y_size,
                    BytesMut::clone(&self.buf).freeze(),
                ))
                .width(Fill)
                .height(Fill),
            ),
            bottom_row.padding(10).spacing(5)
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
                    self.progress = Some(ProgressData {
                        current_pass: 0,
                        total_passes,
                    })
                }
                raytracing_worker::Event::RenderPassAvailable(render_pass) => {
                    render_pass.canvas.write_rgba_to_buffer(&mut self.buf);
                    self.progress.as_mut().unwrap().current_pass = render_pass.current_pass;
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
                        image_width: self.x_size as usize,
                        image_height: self.y_size as usize,
                        bounces: 10,
                        samples: 128,
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
                self.progress = None;
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
        .run_with(|| Daemon::new(500, 500))
}
