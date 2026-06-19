use crate::error::{Error, Result};
use ffmpeg::{
    format::Pixel,
    software::scaling::{context::Context, flag::Flags},
};
use ffmpeg_next as ffmpeg;

pub struct VideoDecoder {
    decoder: ffmpeg::codec::decoder::Video,
    scaler: Option<Context>,
    target_width: u32,
    target_height: u32,
    orig_width: u32,
    orig_height: u32,
    decoded_frame: ffmpeg::util::frame::Video,
    rgb_frame: ffmpeg::util::frame::Video,
}

impl VideoDecoder {
    pub fn new(stream: &ffmpeg::format::stream::Stream) -> Result<Self> {
        let context_decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())
            .map_err(|e| Error::Ffmpeg(e.to_string()))?;
        let decoder = context_decoder
            .decoder()
            .video()
            .map_err(|e| Error::Ffmpeg(e.to_string()))?;
        let orig_width = decoder.width();
        let orig_height = decoder.height();

        Ok(Self {
            decoder,
            scaler: None,
            target_width: orig_width,
            target_height: orig_height,
            orig_width,
            orig_height,
            decoded_frame: ffmpeg::util::frame::Video::empty(),
            rgb_frame: ffmpeg::util::frame::Video::empty(),
        })
    }

    pub fn original_dimensions(&self) -> (u32, u32) {
        (self.orig_width, self.orig_height)
    }

    pub fn last_frame_pts(&self) -> Option<i64> {
        self.decoded_frame.pts()
    }

    pub fn set_scaling(&mut self, target_width: u32, target_height: u32) -> Result<()> {
        let scaler = Context::get(
            self.decoder.format(),
            self.orig_width,
            self.orig_height,
            Pixel::RGB24,
            target_width,
            target_height,
            Flags::BILINEAR,
        )
        .map_err(|e| Error::Ffmpeg(e.to_string()))?;
        self.scaler = Some(scaler);
        self.target_width = target_width;
        self.target_height = target_height;
        Ok(())
    }

    fn copy_frame(&self, output_buffer: &mut Vec<u8>) {
        let width = self.rgb_frame.width() as usize;
        let height = self.rgb_frame.height() as usize;
        let width_bytes = width * 3;
        let stride = self.rgb_frame.stride(0);
        let total = width_bytes * height;

        output_buffer.resize(total, 0);

        if stride == width_bytes {
            output_buffer.copy_from_slice(&self.rgb_frame.data(0)[..total]);
        } else {
            for y in 0..height {
                let src = y * stride;
                let dst = y * width_bytes;
                output_buffer[dst..dst + width_bytes]
                    .copy_from_slice(&self.rgb_frame.data(0)[src..src + width_bytes]);
            }
        }
    }

    pub fn process_packet(
        &mut self,
        packet: &ffmpeg::packet::Packet,
        output_buffer: &mut Vec<u8>,
    ) -> Result<bool> {
        if self.scaler.is_none() {
            return Err(Error::ScalingNotSet);
        }

        let mut drained = false;
        loop {
            match self.decoder.send_packet(packet) {
                Ok(()) => break,
                Err(ffmpeg::Error::Other { errno }) if errno == ffmpeg::error::EAGAIN => {
                    if self.decoder.receive_frame(&mut self.decoded_frame).is_ok() {
                        self.scaler.as_mut().unwrap()
                            .run(&self.decoded_frame, &mut self.rgb_frame)
                            .map_err(|e| Error::Ffmpeg(e.to_string()))?;
                        self.copy_frame(output_buffer);
                        drained = true;
                    }
                }
                Err(e) => return Err(Error::Ffmpeg(e.to_string())),
            }
        }

        if !drained {
            if let Ok(()) = self.decoder.receive_frame(&mut self.decoded_frame) {
                self.scaler.as_mut().unwrap()
                    .run(&self.decoded_frame, &mut self.rgb_frame)
                    .map_err(|e| Error::Ffmpeg(e.to_string()))?;
                self.copy_frame(output_buffer);
                return Ok(true);
            }
            Ok(false)
        } else {
            Ok(true)
        }
    }
}
