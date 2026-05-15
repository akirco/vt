use crate::error::{Error, Result};
use sixel_rs::encoder::{Encoder, QuickFrameBuilder};
use sixel_rs::optflags::{DiffusionMethod, Quality};
use sixel_rs::pixelformat::PixelFormat;

pub struct SixelEncoder {
    encoder: Encoder,
    diffusion_name: &'static str,
}

unsafe impl Send for SixelEncoder {}

impl SixelEncoder {
    pub fn new(num_colors: u8, diffusion: DiffusionMethod, quality: Quality) -> Result<Self> {
        let encoder = Encoder::new().map_err(|e| Error::Sixel(format!("{:?}", e)))?;
        encoder
            .set_num_colors(num_colors)
            .map_err(|e| Error::Sixel(format!("set_num_colors: {:?}", e)))?;
        encoder
            .set_diffusion(diffusion)
            .map_err(|e| Error::Sixel(format!("set_diffusion: {:?}", e)))?;
        encoder
            .set_quality(quality)
            .map_err(|e| Error::Sixel(format!("set_quality: {:?}", e)))?;
        Ok(Self {
            encoder,
            diffusion_name: match diffusion {
                DiffusionMethod::None => "none",
                DiffusionMethod::Atkinson => "atkinson",
                DiffusionMethod::FS => "fs",
                DiffusionMethod::Stucki => "stucki",
                DiffusionMethod::Burkes => "burkes",
                DiffusionMethod::Jajuni => "jajuni",
                DiffusionMethod::Auto => "auto",
            },
        })
    }

    pub fn encode_frame(&mut self, width: usize, height: usize, rgb_data: &[u8]) -> Result<()> {
        let frame = QuickFrameBuilder::new()
            .width(width)
            .height(height)
            .format(PixelFormat::RGB888)
            .pixels(rgb_data.to_vec());
        self.encoder
            .encode_bytes(frame)
            .map_err(|e| Error::Sixel(format!("encode: {:?}", e)))?;
        Ok(())
    }

    pub fn diffusion_name(&self) -> &'static str {
        self.diffusion_name
    }
}
