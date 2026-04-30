mod args;
mod audio;
mod display;
mod error;
mod kitty;
mod player;
mod sixel;
mod terminal;
mod video;

use crate::error::Result;
use ffmpeg_next as ffmpeg;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;

fn main() -> Result<()> {
    ffmpeg::init()?;

    let cli = args::Cli::parse_args();
    let config: args::Config = cli.into();
    let protocol = terminal::determine_protocol(config.force_protocol.as_deref());

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    })?;

    let ictx = ffmpeg::format::input(&config.path)?;
    let video_stream = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or(error::Error::NoVideoStream)?;
    let video_stream_index = video_stream.index();

    let decoder = video::VideoDecoder::new(&video_stream)?;
    let (orig_width, orig_height) = decoder.original_dimensions();

    let target_width = (orig_width as f32 * config.scale) as u32;
    let target_height = (orig_height as f32 * config.scale) as u32;

    let audio_stream_info = if config.audio {
        ictx.streams()
            .best(ffmpeg::media::Type::Audio)
            .map(|s| (s.index(), s.parameters()))
    } else {
        None
    };

    let mut audio_player = None;
    let audio_sender = if let Some((audio_stream_index, audio_params)) = audio_stream_info {
        let audio_decoder = ffmpeg::codec::context::Context::from_parameters(audio_params)
            .ok()
            .and_then(|ctx| ctx.decoder().audio().ok());

        if let Some(audio_decoder) = audio_decoder {
            let sample_rate = audio_decoder.rate();
            let channels = audio_decoder.channels();

            if sample_rate > 0 && channels > 0 {
                let (tx, rx) = mpsc::channel();

                let player = audio::AudioPlayer::new(
                    audio::AudioPlayerConfig {
                        sample_rate,
                        channels,
                    },
                    rx,
                    running.clone(),
                );

                if player.is_some() {
                    audio_player = player;
                    Some((
                        tx,
                        audio_stream_index,
                        audio_decoder,
                        channels as i32,
                        sample_rate,
                    ))
                } else {
                    eprintln!("Failed to create audio player, continuing without audio");
                    None
                }
            } else {
                eprintln!("Invalid audio parameters, continuing without audio");
                None
            }
        } else {
            eprintln!("Failed to initialize audio decoder, continuing without audio");
            None
        }
    } else {
        None
    };

    let mut player = player::Player::new(
        &video_stream,
        player::PlayerConfig {
            target_width,
            target_height,
            protocol,
            colors: config.colors,
            diffusion: config.diffusion,
            quality: config.quality,
            verbose: config.verbose,
        },
    )?;

    player.run(ictx, video_stream_index, audio_sender, running.clone())?;

    if let Some(mut audio) = audio_player {
        audio.stop();
    }

    Ok(())
}
