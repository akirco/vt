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
    pub fn new(
        stream: &ffmpeg::format::stream::Stream,
    ) -> Result<Self> {
        let context_decoder =
            ffmpeg::codec::context::Context::from_parameters(stream.parameters())
            .map_err(|e| Error::Ffmpeg(e.to_string()))?;
        let decoder = context_decoder.decoder().video()
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

    pub fn set_scaling(
        &mut self,
        target_width: u32,
        target_height: u32,
    ) -> Result<()> {
        let scaler = Context::get(
            self.decoder.format(),
            self.orig_width,
            self.orig_height,
            Pixel::RGB24,
            target_width,
            target_height,
            Flags::FAST_BILINEAR,
        ).map_err(|e| Error::Ffmpeg(e.to_string()))?;
        self.scaler = Some(scaler);
        self.target_width = target_width;
        self.target_height = target_height;
        Ok(())
    }

    pub fn process_packet(
        &mut self,
        packet: &ffmpeg::packet::Packet,
        output_buffer: &mut Vec<u8>,
    ) -> Result<bool> {
        let scaler = self.scaler.as_mut().ok_or(Error::ScalingNotSet)?;

        self.decoder.send_packet(packet)
            .map_err(|e| Error::Ffmpeg(e.to_string()))?;

        if let Ok(()) = self.decoder.receive_frame(&mut self.decoded_frame) {
            scaler.run(&self.decoded_frame, &mut self.rgb_frame)
                .map_err(|e| Error::Ffmpeg(e.to_string()))?;

            let width = self.rgb_frame.width() as usize;
            let height = self.rgb_frame.height() as usize;
            let data_len = width * height * 3;

            if output_buffer.len() != data_len {
                output_buffer.resize(data_len, 0);
            }
            output_buffer.copy_from_slice(&self.rgb_frame.data(0)[..data_len]);
            return Ok(true);
        }
        Ok(false)
    }
}
