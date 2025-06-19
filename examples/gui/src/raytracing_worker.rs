use std::sync::Arc;

use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::stream::{Stream, StreamExt};
use iced::stream;
use raytracing_lib::RenderPass;

pub fn raytracer_worker() -> impl Stream<Item = Event> {
    stream::channel(100, |mut output| async move {
        // create a channel we will send to the UI code so that it can communicate back with the worker
        let (sender, mut receiver) = mpsc::channel::<WorkerMessage>(20);

        let mut state = WorkerState::Idle;
        // let the driving code know that we are ready to accept render requests, passing it the tx end of the channel
        // created above so that it can send WorkerMessage back to this function.
        let _ = output.send(Event::ReadyToStart(sender)).await;

        // then we loop and take action depending on the worker state machine
        loop {
            match &mut state {
                WorkerState::Idle => {
                    loop {
                        // loop until we get an message asking us to start a render
                        let msg = receiver.select_next_some().await;
                        if let WorkerMessage::StartRenderRequest(rr) = msg {
                            state = WorkerState::Rendering(rr);
                            break;
                        }
                    }
                }
                WorkerState::Rendering(ref rr) => {
                    // create a renderer
                    let mut renderer =
                        raytracing_lib::Renderer::new(rr.world.clone(), Arc::clone(&rr.camera))
                            .with_cli_progress_tracker()
                            .width(rr.image_width)
                            .height(rr.image_height)
                            .bounces(rr.bounces)
                            .samples(rr.samples);
                    let render_pass_rx = renderer.get_render_pass_rx();

                    // spawn the renderer in another thread (not cancellable)
                    tokio::task::spawn_blocking(move || {
                        renderer.render();
                    });

                    // create another blocking task that bridges the sync channel used by the raytracer with an async-enabled tokio channel that the stream can consume from
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<RenderPass>(2);
                    tokio::task::spawn_blocking(move || {
                        while let Ok(rp) = render_pass_rx.recv() {
                            if tx.blocking_send(rp).is_err() {
                                // the receiving end was dropped by closing the window and killing the subscription
                                // we can stop this task as well
                                return;
                            };
                        }
                    });

                    // while we are getting render passes messages, loop and propagate events to the UI update logic
                    while let Some(rp) = rx.recv().await {
                        let _ = output.send(Event::RenderPassAvailable(rp)).await;
                    }
                }
            }
        }
    })
}

enum WorkerState {
    Idle,
    Rendering(RenderRequest),
}

#[allow(dead_code)]
pub struct RenderRequest {
    pub camera: Arc<raytracing_lib::Camera>,
    pub world: raytracing_lib::World,
    pub material_atlas: raytracing_lib::MaterialAtlas,
    pub image_width: usize,
    pub image_height: usize,
    pub bounces: usize,
    pub samples: usize,
}

#[allow(dead_code)]
pub enum WorkerMessage {
    StartRenderRequest(RenderRequest),
    Other,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Event {
    ReadyToStart(mpsc::Sender<WorkerMessage>),
    RenderStarted(mpsc::Sender<WorkerMessage>),
    RenderPassAvailable(raytracing_lib::RenderPass),
}
