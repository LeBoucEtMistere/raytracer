use crossbeam_channel::{unbounded, Sender};
use indicatif::{ProgressBar, ProgressStyle};
use nalgebra_glm::Vec3;
use rand::prelude::*;
use std::{path::Path, sync::Arc, thread, thread::JoinHandle, time::Duration};
use threadpool::ThreadPool;

use crate::export::PPMWriter;
use crate::{Camera, Canvas, HittableList, Ray, World};

pub struct Renderer {
    world: World,
    camera: Arc<Camera>,
    width: usize,
    height: usize,
    samples: usize,
    bounces: usize,
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

    pub fn render(self) -> Render {
        // Progress tracking
        let (tx, handle) = Renderer::start_progress_tracker(self.samples as u64);

        let (data_tx, data_rx) = unbounded::<Canvas>();
        let tp = ThreadPool::default();

        for _ in 0..self.samples {
            let bounces = self.bounces;
            let height = self.height;
            let width = self.width;
            let camera_arc = Arc::clone(&self.camera);
            let hittables_arc = self.world.get_hittables();
            let tx_clone = tx.clone();
            let data_tx_clone = data_tx.clone();

            tp.execute(move || {
                let result =
                    Renderer::compute_render(height, width, camera_arc, hittables_arc, bounces);
                tx_clone.send(()).unwrap();
                data_tx_clone.send(result).unwrap();
            })
        }

        tp.join();
        let mut cv = Canvas::new_initialized(self.height, self.width);
        cv = data_rx.iter().take(self.samples).fold(cv, |acc, b| acc + b);

        cv.normalize();
        cv.gamma_correction();

        // Close progress tracking therad properly
        drop(tx); // close channel by dropping last alive Sender
        handle.join().unwrap();

        Render { canvas: cv }
    }

    fn compute_render(
        canvas_height: usize,
        canvas_width: usize,
        camera: Arc<Camera>,
        world: Arc<HittableList>,
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

    fn ray_color(world: &Arc<HittableList>, r: &Ray, depth: usize) -> Vec3 {
        if depth <= 0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        if let Some(record) = world.hit(r, 0.001f32, f32::INFINITY) {
            if let Some(scattered) = record.material_hit.scatter(r, &record) {
                // a ray is scattered by the material
                let albedo = record.material_hit.albedo();

                return albedo.component_mul(&Renderer::ray_color(world, &scattered, depth - 1));
            }
            // no scattered ray, return black
            return Vec3::new(0.0, 0.0, 0.0);
        } else {
            let unit_direction = nalgebra_glm::normalize(&r.direction);
            let t = 0.5 * (unit_direction.y + 1.0); // t is between 0.0 and 1.0
            nalgebra_glm::lerp(&Vec3::new(1.0, 1.0, 1.0), &Vec3::new(0.5, 0.7, 1.0), t)
        }
    }

    fn start_progress_tracker(length: u64) -> (Sender<()>, JoinHandle<()>) {
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
        (tx, progress_thread_handle)
    }
}

pub struct Render {
    canvas: Canvas,
}

impl Render {
    pub fn save<P: AsRef<Path>>(&self, path: &P) -> std::io::Result<()> {
        self.canvas.write_to_file(&path).map_err(|e| e.into())
    }
}
