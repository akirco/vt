pub use sixel_rs::optflags::{DiffusionMethod, Quality};

use crate::error::{Error, Result};
use sixel_rs::encoder::{Encoder, QuickFrameBuilder};
use sixel_rs::pixelformat::PixelFormat;

pub fn parse_diffusion(s: &str) -> std::result::Result<DiffusionMethod, String> {
    match s.to_lowercase().as_str() {
        "none" => Ok(DiffusionMethod::None),
        "atkinson" => Ok(DiffusionMethod::Atkinson),
        "fs" => Ok(DiffusionMethod::FS),
        "stucki" => Ok(DiffusionMethod::Stucki),
        "burkes" => Ok(DiffusionMethod::Burkes),
        "jajuni" => Ok(DiffusionMethod::Jajuni),
        "auto" => Ok(DiffusionMethod::Auto),
        _ => Err(format!("unknown diffusion: {s}")),
    }
}

pub fn parse_quality(s: &str) -> std::result::Result<Quality, String> {
    match s.to_lowercase().as_str() {
        "low" => Ok(Quality::Low),
        "high" => Ok(Quality::High),
        "full" => Ok(Quality::Full),
        "auto" => Ok(Quality::Auto),
        _ => Err(format!("unknown quality: {s}")),
    }
}

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
