use crate::display::{CursorGuard, clear_screen, hide_cursor, move_cursor_home, write_status_line};
use crate::error::Result;
use crate::kitty::KittyEncoder;
use crate::sixel::SixelEncoder;
use crate::terminal::ImageProtocol;
use crate::video::VideoDecoder;
use sixel_rs::optflags::{DiffusionMethod, Quality};

use ffmpeg_next as ffmpeg;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Instant;

type AudioInfo = (
    mpsc::Sender<Vec<f32>>,
    usize,
    ffmpeg::codec::decoder::Audio,
    i32,
    u32,
);

pub struct PlayerConfig {
    pub target_width: u32,
    pub target_height: u32,
    pub protocol: ImageProtocol,
    pub colors: u8,
    pub diffusion: DiffusionMethod,
    pub quality: Quality,
    pub verbose: bool,
}

pub struct Player {
    decoder: VideoDecoder,
    sixel_enc: Option<SixelEncoder>,
    kitty_enc: Option<KittyEncoder>,
    target_width: u32,
    target_height: u32,
    frame_duration: f64,
    protocol: ImageProtocol,
    colors: u8,
    rgb_buffer: Vec<u8>,
    verbose: bool,
}

impl Player {
    pub fn new(stream: &ffmpeg::Stream, config: PlayerConfig) -> Result<Self> {
        let mut decoder = VideoDecoder::new(stream)?;
        decoder.set_scaling(config.target_width, config.target_height)?;

        let sixel_enc = if config.protocol == ImageProtocol::Sixel {
            Some(SixelEncoder::new(
                config.colors,
                config.diffusion,
                config.quality,
            )?)
        } else {
            None
        };

        let kitty_enc = if config.protocol == ImageProtocol::Kitty {
            Some(KittyEncoder::new())
        } else {
            None
        };

        let fps = stream.avg_frame_rate().0 as f64 / stream.avg_frame_rate().1 as f64;
        let frame_duration = if fps > 0.0 { 1.0 / fps } else { 0.0 };

        let buffer_capacity = (config.target_width * config.target_height * 3) as usize;
        let rgb_buffer = Vec::with_capacity(buffer_capacity);

        Ok(Self {
            decoder,
            sixel_enc,
            kitty_enc,
            target_width: config.target_width,
            target_height: config.target_height,
            frame_duration,
            protocol: config.protocol,
            colors: config.colors,
            rgb_buffer,
            verbose: config.verbose,
        })
    }

    pub fn run(
        &mut self,
        mut ictx: ffmpeg::format::context::Input,
        video_stream_index: usize,
        audio_info: Option<AudioInfo>,
        running: Arc<AtomicBool>,
    ) -> Result<()> {
        let _cursor_guard = CursorGuard;
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();

        clear_screen(&mut stdout_lock)?;
        hide_cursor(&mut stdout_lock)?;

        let start = Instant::now();
        let mut frame_count = 0u32;
        let mut last_frame_time = Instant::now();

        let mut audio_decoder = None;
        let mut audio_resampler = None;
        let mut audio_sender = None;

        if let Some((sender, audio_stream_index, decoder, channels, sample_rate)) = audio_info {
            audio_sender = Some((sender, audio_stream_index));

            let channel_layout = ffmpeg::ChannelLayout::default(channels);
            let resampler = ffmpeg::software::resampling::context::Context::get(
                decoder.format(),
                channel_layout,
                decoder.rate(),
                ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
                channel_layout,
                sample_rate,
            )
            .ok();

            audio_decoder = Some(decoder);
            audio_resampler = resampler;
        }

        for (stream, packet) in ictx.packets() {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            if let Some((ref sender, audio_stream_index)) = audio_sender
                && stream.index() == audio_stream_index
            {
                if let Some(ref mut decoder) = audio_decoder {
                    if decoder.send_packet(&packet).is_err() {
                        continue;
                    }

                    let mut decoded = ffmpeg::util::frame::Audio::empty();
                    while decoder.receive_frame(&mut decoded).is_ok() {
                        if !running.load(Ordering::SeqCst) {
                            break;
                        }

                        let samples = if let Some(ref mut resampler) = audio_resampler {
                            let mut resampled = ffmpeg::util::frame::Audio::empty();
                            if resampler.run(&decoded, &mut resampled).is_err() {
                                continue;
                            }
                            let data = resampled.data(0);
                            unsafe {
                                std::slice::from_raw_parts(
                                    data.as_ptr() as *const f32,
                                    data.len() / std::mem::size_of::<f32>(),
                                )
                                .to_vec()
                            }
                        } else {
                            let data = decoded.data(0);
                            unsafe {
                                std::slice::from_raw_parts(
                                    data.as_ptr() as *const f32,
                                    data.len() / std::mem::size_of::<f32>(),
                                )
                                .to_vec()
                            }
                        };

                        if !samples.is_empty() {
                            let _ = sender.send(samples);
                        }
                    }
                }
                continue;
            }

            if stream.index() == video_stream_index
                && self.decoder.process_packet(&packet, &mut self.rgb_buffer)?
            {
                self.sync_frame(&mut last_frame_time);

                match self.protocol {
                    ImageProtocol::Sixel => {
                        if let Some(enc) = self.sixel_enc.as_mut() {
                            move_cursor_home(&mut stdout_lock)?;
                            stdout_lock.flush()?;
                            drop(stdout_lock);
                            enc.encode_frame(
                                self.target_width as usize,
                                self.target_height as usize,
                                &self.rgb_buffer,
                            )?;
                            stdout_lock = std::io::stdout().lock();
                        }
                    }
                    ImageProtocol::Kitty => {
                        if let Some(enc) = self.kitty_enc.as_mut() {
                            move_cursor_home(&mut stdout_lock)?;
                            enc.encode_frame(
                                &mut stdout_lock,
                                self.target_width as usize,
                                self.target_height as usize,
                                &self.rgb_buffer,
                            )?;
                        }
                    }
                }

                frame_count += 1;
                let elapsed = start.elapsed().as_secs_f64();
                let fps = if elapsed > 0.0 {
                    frame_count as f64 / elapsed
                } else {
                    0.0
                };

                if self.verbose {
                    let (protocol_name, diffusion_name) = match self.protocol {
                        ImageProtocol::Sixel => {
                            ("sixel", self.sixel_enc.as_ref().unwrap().diffusion_name())
                        }
                        ImageProtocol::Kitty => ("kitty", "none"),
                    };

                    write_status_line(
                        &mut stdout_lock,
                        self.target_height + 2,
                        fps,
                        frame_count,
                        elapsed,
                        self.target_width,
                        self.target_height,
                        self.colors,
                        protocol_name,
                        diffusion_name,
                    )?;
                }
                stdout_lock.flush()?;
            }
        }

        Ok(())
    }

    fn sync_frame(&mut self, last_frame_time: &mut Instant) {
        if self.frame_duration > 0.0 {
            let elapsed_since_last = last_frame_time.elapsed().as_secs_f64();
            if elapsed_since_last < self.frame_duration {
                let remaining = self.frame_duration - elapsed_since_last;
                if remaining > 0.001 {
                    std::thread::sleep(std::time::Duration::from_secs_f64(remaining));
                }
            }
        }
        *last_frame_time = Instant::now();
    }
}
