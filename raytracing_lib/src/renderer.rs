#[cfg(feature = "bytes")]
use bytes::BytesMut;
use crossbeam_channel::{unbounded, Receiver, Sender};
use indicatif::{ProgressBar, ProgressStyle};
use nalgebra_glm::Vec3;
use rand::prelude::*;
use std::{path::Path, sync::Arc, thread, thread::JoinHandle, time::Duration};
use threadpool::ThreadPool;

use crate::{bvh::BVHNode, collision::Hittable, export::PPMWriter};
use crate::{Camera, Canvas, Ray, World};

#[derive(Debug, Clone)]
pub struct RenderPass {
    pub canvas: Canvas,
    pub current_pass: usize,
    pub total_passes: usize,
}

pub struct Renderer {
    world: World,
    camera: Arc<Camera>,
    width: usize,
    height: usize,
    samples: usize,
    bounces: usize,
    render_pass_tx: Option<Sender<RenderPass>>,
    with_cli_progress_tracker: bool,
}

impl Renderer {
    pub fn new(world: World, camera: Arc<Camera>) -> Self {
        Self {
            world,
            camera,
            width: 960,
            height: 540,
            samples: 100,
            bounces: 2,
            render_pass_tx: None,
            with_cli_progress_tracker: false,
        }
    }

    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn samples(mut self, samples: usize) -> Self {
        self.samples = samples;
        self
    }

    pub fn bounces(mut self, bounces: usize) -> Self {
        self.bounces = bounces;
        self
    }

    pub fn get_render_pass_rx(&mut self) -> Receiver<RenderPass> {
        let (tx, rx) = unbounded::<RenderPass>();
        self.render_pass_tx = Some(tx);
        rx
    }

    pub fn with_cli_progress_tracker(mut self) -> Self {
        self.with_cli_progress_tracker = true;
        self
    }

    pub fn render(self) -> Render {
        // Progress tracking
        let mut progress_tracker = self.start_progress_tracker(self.samples as u64);

        // each independently rendered frame is sent using this channel to a thread that will aggregate them on the go
        // possibly sending regular updates to any registered render_pass channel.
        let (data_tx, data_rx) = unbounded::<Canvas>();
        let mut cv = Canvas::new_initialized(self.height, self.width);
        let total_passes = self.samples;
        let mut render_pass_tx = self.render_pass_tx;

        let render_pass_aggregator = thread::spawn(move || {
            let mut render_pass: usize = 1;
            while let Ok(new_cv) = data_rx.recv() {
                cv += new_cv;
                if let Some(render_pass_tx) = render_pass_tx.as_mut() {
                    let mut output = cv.clone();
                    output.normalize();
                    output.gamma_correction();
                    render_pass_tx
                        .send(RenderPass {
                            canvas: output,
                            current_pass: render_pass,
                            total_passes,
                        })
                        .unwrap();
                    render_pass += 1;
                }
            }
            cv.normalize();
            cv.gamma_correction();

            cv
        });

        // create rendering threadpool and spawn it
        let tp = ThreadPool::default();
        for _ in 0..self.samples {
            let bounces = self.bounces;
            let height = self.height;
            let width = self.width;
            let camera_arc = Arc::clone(&self.camera);
            let hittables_arc = self.world.get_hittables();
            let tx_clone = progress_tracker.as_mut().map(|x| x.0.clone());
            let data_tx_clone = data_tx.clone();

            tp.execute(move || {
                let result =
                    Renderer::compute_render(height, width, camera_arc, hittables_arc, bounces);
                if let Some(tx) = tx_clone {
                    tx.send(()).unwrap();
                }
                data_tx_clone.send(result).unwrap();
            })
        }

        tp.join();
        if let Some((tx, handle)) = progress_tracker {
            drop(tx); // close channel by dropping last alive Sender
                      // Close progress tracking thread properly
            handle.join().unwrap();
        }
        let cv = render_pass_aggregator.join().unwrap();

        Render { canvas: cv }
    }

    fn compute_render(
        canvas_height: usize,
        canvas_width: usize,
        camera: Arc<Camera>,
        world: Arc<BVHNode>,
        max_depth: usize,
    ) -> Canvas {
        let mut temp_cv = Canvas::new_initialized(canvas_height, canvas_width);
        for j in 0..canvas_height {
            for i in 0..canvas_width {
                let u = (i as f32 + random::<f32>()) / (canvas_width - 1) as f32;
                let v = (j as f32 + random::<f32>()) / (canvas_height - 1) as f32;
                let r = camera.get_ray_from_coords(u, v);

                // need to flip horizontally since Canvas has its y axis going down and camera going up
                temp_cv.set_pixel(
                    i,
                    canvas_height - 1 - j,
                    Renderer::ray_color(&world, &r, max_depth),
                )
            }
        }
        temp_cv
    }

    fn ray_color(world: &Arc<BVHNode>, r: &Ray, depth: usize) -> Vec3 {
        if depth == 0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        if let Some(record) = world.hit(r, 0.001f32, f32::INFINITY) {
            if let Some(scattered) = record.material_hit.scatter(r, &record) {
                // a ray is scattered by the material
                let albedo = record.material_hit.albedo();

                return albedo.component_mul(&Renderer::ray_color(world, &scattered, depth - 1));
            }
            // no scattered ray, return black
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            let unit_direction = nalgebra_glm::normalize(&r.direction);
            let t = 0.5 * (unit_direction.y + 1.0); // t is between 0.0 and 1.0
            nalgebra_glm::lerp(&Vec3::new(1.0, 1.0, 1.0), &Vec3::new(0.5, 0.7, 1.0), t)
        }
    }

    fn start_progress_tracker(&self, length: u64) -> Option<(Sender<()>, JoinHandle<()>)> {
        if !self.with_cli_progress_tracker {
            return None;
        }
        // Progress tracking
        let (tx, rx) = unbounded::<()>();
        let progress_thread_handle = std::thread::spawn(move || {
            println!("Starting render ...");
            let pb = ProgressBar::new(length);
            pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} pass(es) ({eta})")
        .progress_chars("#>-"));
            pb.set_position(0);
            let mut done = 0usize;
            let mut finished = false;
            while !finished {
                for _msg in rx.try_iter() {
                    done += 1;
                    pb.set_position(done as u64);
                }
                match rx.try_recv() {
                    Err(crossbeam_channel::TryRecvError::Empty) => {}
                    Err(crossbeam_channel::TryRecvError::Disconnected) => {
                        finished = true;
                    }
                    Ok(_msg) => {
                        done += 1;
                        pb.set_position(done as u64);
                    }
                }
                pb.set_position(done as u64);
                thread::sleep(Duration::from_millis(500));
            }
            pb.finish();
        });
        Some((tx, progress_thread_handle))
    }
}

pub struct Render {
    canvas: Canvas,
}

impl Render {
    pub fn save<P: AsRef<Path>>(&self, path: &P) -> std::io::Result<()> {
        self.canvas.write_to_file(&path)
    }

    #[cfg(feature = "bytes")]
    pub fn write_rgba_to_buffer(&self, buf: &mut BytesMut) {
        use crate::export::MemWriter;

        self.canvas.write_rgba_to_buffer(buf);
    }
}
