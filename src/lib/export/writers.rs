use super::pixel::RGBPixel;
use std::fs::File;
use std::io::{BufWriter, Result, Write};
use std::path::Path;
pub trait PPMWriter<'a> {
    fn get_dim(self: &Self) -> (usize, usize);
    fn get_pixels(self: &'a Self) -> Vec<RGBPixel>;
    fn write_to_file<P: AsRef<Path>>(self: &'a Self, path: &P) -> Result<()> {
        let mut buffer = BufWriter::new(File::create(path)?);
        let dims = self.get_dim();
        buffer.write(format!("P3\n{} {}\n255\n", dims.0, dims.1).as_bytes())?;
        for p in self.get_pixels() {
            buffer.write(format!("{}\n", p).as_bytes())?;
        }
        buffer.flush()?;
        Ok(())
    }
}
