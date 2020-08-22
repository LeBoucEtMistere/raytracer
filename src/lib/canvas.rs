use super::export::{PPMWriter, RGBPixel};
use itertools::Itertools;
use nalgebra_glm::Vec3;
use ndarray::{s, Array3};
use std::ops::{Add, AddAssign};

#[derive(Debug)]
pub struct Canvas {
    height: usize,
    width: usize,
    data: Array3<f32>,
    layers: usize,
}

impl Canvas {
    pub fn new_initialized(height: usize, width: usize) -> Self {
        let data = Array3::zeros((height, width, 3));
        Canvas {
            height,
            width,
            data,
            layers: 0,
        }
    }

    pub fn set_pixel(&mut self, i: usize, j: usize, color: Vec3) {
        if self.layers == 0 {
            self.layers = 1;
        }
        let mut slice = self.data.slice_mut(s![j, i, ..]);
        slice[0] = color.x;
        slice[1] = color.y;
        slice[2] = color.z;
    }

    pub fn gamma_correction(&mut self) {
        self.data.par_mapv_inplace(f32::sqrt);
    }

    pub fn normalize(&mut self) {
        if self.layers > 1 {
            self.data /= self.layers as f32;
            self.layers = 1;
        }
    }
}

impl<'a> PPMWriter<'a> for Canvas {
    fn get_dim(self: &Self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get_pixels(self: &'a Self) -> Vec<RGBPixel> {
        self.data
            .iter()
            .chunks(3)
            .into_iter()
            .map(|chunk| {
                let clamped_values: Vec<u8> = chunk
                    .map(|x| (256f32 * f32::min(0.999, f32::max(0.0, *x))) as u8)
                    .take(3)
                    .collect();
                RGBPixel(
                    *clamped_values.get(0).unwrap(),
                    *clamped_values.get(1).unwrap(),
                    *clamped_values.get(2).unwrap(),
                )
            })
            .collect()
    }
}

impl Add for Canvas {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Canvas {
            width: self.width,
            height: self.height,
            data: self.data.add(other.data),
            layers: self.layers + other.layers,
        }
    }
}

impl AddAssign for Canvas {
    fn add_assign(&mut self, other: Self) {
        self.data += &other.data;
        self.layers += other.layers;
    }
}
