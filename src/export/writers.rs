use super::pixel::RGBPixel;
use std::fs::File;
use std::io::{BufWriter, Result, Write};
use std::path::Path;
pub trait PPMWriter<'a> {
    fn get_dim(&self) -> (usize, usize);
    fn get_pixels(&'a self) -> Vec<RGBPixel>;
    fn write_to_file<P: AsRef<Path>>(&'a self, path: &P) -> Result<()> {
        let mut buffer = BufWriter::new(File::create(path)?);
        let dims = self.get_dim();
        buffer.write_all(format!("P3\n{} {}\n255\n", dims.0, dims.1).as_bytes())?;
        for p in self.get_pixels() {
            buffer.write_all(format!("{}\n", p).as_bytes())?;
        }
        buffer.flush()?;
        Ok(())
    }
}

#[cfg(feature = "bytes")]
pub trait MemWriter {
    fn write_rgba_to_buffer(&self, buf: &mut bytes::BytesMut);
}
