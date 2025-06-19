mod raytracing_worker;

use std::error::Error;
use std::fs;

use iced::advanced::graphics::image::image_rs::EncodableLayout;
use iced::advanced::image::Handle;
use iced::widget::image;

use bytes::BytesMut;
use iced::Length::Fill;
use iced::{Element, Subscription, Task};
use nalgebra_glm::Vec3;
use raytracing_lib::export::MemWriter;
use raytracing_lib::{Camera, FocusData, MaterialAtlas, World};

use crate::raytracing_worker::{RenderRequest, WorkerMessage};

#[derive(Debug)]
enum Message {
    WorkerEvent(raytracing_worker::Event),
    CloseWindowEvent,
}

struct State {
    x_size: u32,
    y_size: u32,
    buf: BytesMut,
}

struct MainWindow {}

impl MainWindow {
    fn view(state: &State) -> Element<Message> {
        image(Handle::from_rgba(
            state.x_size,
            state.y_size,
            BytesMut::clone(&state.buf).freeze(),
        ))
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn update(state: &mut State, message: Message) {
        match message {
            Message::WorkerEvent(we) => {
                match we {
                    raytracing_worker::Event::ReadyToStart(mut sender) => {
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

                        let scene: raytracing_lib::scene::serialization::Scene =
                            serde_yaml::from_str(
                                &fs::read_to_string("examples/from_scene/scene.yaml").unwrap(),
                            )
                            .unwrap();

                        let (material_atlas, world) =
                            std::convert::TryInto::<(MaterialAtlas, World)>::try_into(scene)
                                .unwrap();

                        sender
                            .try_send(WorkerMessage::StartRenderRequest(RenderRequest {
                                camera,
                                world,
                                material_atlas,
                                image_width: state.x_size as usize,
                                image_height: state.y_size as usize,
                                bounces: 10,
                                samples: 128,
                            }))
                            .unwrap();
                    }
                    raytracing_worker::Event::RenderStarted(_sender) => {}
                    raytracing_worker::Event::RenderPassAvailable(render_pass) => {
                        render_pass.canvas.write_rgba_to_buffer(&mut state.buf);
                    }
                }
            }
            Message::CloseWindowEvent => {
                println!("closing...");
                std::process::exit(0);
            }
        }
    }

    fn subscriptions(_state: &State) -> Subscription<Message> {
        Subscription::batch([
            iced::window::close_events().map(|_| Message::CloseWindowEvent),
            Subscription::run(raytracing_worker::raytracer_worker).map(Message::WorkerEvent),
        ])
    }
}

pub fn main() -> Result<(), impl Error> {
    iced::application("Realtime Raytracer", MainWindow::update, MainWindow::view)
        .subscription(MainWindow::subscriptions)
        .window_size([600., 600.])
        .run_with(|| {
            (
                State {
                    x_size: 500,
                    y_size: 500,
                    buf: BytesMut::from(
                        Vec::from_iter(
                            [0u8, 0u8, 0u8, 255u8]
                                .into_iter()
                                .cycle()
                                .take(500 * 500 * 4),
                        )
                        .as_bytes(),
                    ),
                },
                Task::none(),
            )
        })
}
