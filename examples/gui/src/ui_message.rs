use crate::raytracing_worker;

#[derive(Debug, Clone)]
pub enum Message {
    WorkerEvent(raytracing_worker::Event),
    CloseWindowEvent,
    WindowOpened,
    StartRender,
    StopRender,
    RenderWidthChanged(Option<u32>),
    RenderHeightChanged(Option<u32>),
    RenderBouncesChanged(Option<usize>),
    RenderPassesChanged(Option<usize>),
}
