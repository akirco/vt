mod args;
mod ascii;
mod audio;
mod braille;
mod error;
mod halfblock;
mod image;
mod kitty;
mod player;
mod protocol;
mod sixel;
mod terminal;
mod video;

use crate::error::Result;
use crate::terminal::{CursorGuard, clear_screen, hide_cursor, is_fzf_preview};
use clap::CommandFactory;
use ffmpeg_next as ffmpeg;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;

fn main() -> Result<()> {
    let cli = args::Cli::parse_args();
    let config: args::Config = match cli.path.clone() {
        Some(_) => cli.into(),
        None => {
            let mut cmd = args::Cli::command();
            cmd.print_help()?;
            println!();
            return Ok(());
        }
    };
    let protocol = protocol::determine_protocol(config.force_protocol.as_deref());

    if image::is_image_extension(&config.path) {
        if let Err(e) = run_image(&config, protocol) {
            if config.verbose {
                eprintln!("image crate failed, falling back to FFmpeg: {e}");
            }
            run_video(&config, protocol)?;
        }
    } else {
        run_video(&config, protocol)?;
    }

    Ok(())
}

fn run_image(config: &args::Config, protocol: protocol::ImageProtocol) -> Result<()> {
    let (img, orig_w, orig_h) = image::load_image(&config.path)?;
    let (tw, th) = terminal::fit_dimensions(orig_w, orig_h, config.scale, config.size, protocol);

    let rgb_data = image::resize_image(img, tw, th);

    let (cx, cy) = terminal::compute_center_offset(tw, th, protocol, config.center);

    let _guard = CursorGuard;
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    clear_screen(&mut stdout_lock)?;
    hide_cursor(&mut stdout_lock)?;

    match protocol {
        protocol::ImageProtocol::Sixel => {
            let mut enc =
                sixel::SixelEncoder::new(config.colors, config.diffusion, config.quality)?;
            write!(stdout_lock, "\x1b[{};{}H", cy + 1, cx + 1)?;
            stdout_lock.flush()?;
            drop(stdout_lock);
            enc.encode_frame(tw as usize, th as usize, &rgb_data)?;
        }
        protocol::ImageProtocol::Kitty => {
            let mut enc = kitty::KittyEncoder::new();
            enc.encode_frame(
                &mut stdout_lock,
                tw as usize,
                th as usize,
                &rgb_data,
                cx,
                cy,
            )?;
        }
        protocol::ImageProtocol::HalfBlock => {
            let mut enc = halfblock::HalfBlockEncoder::new();
            enc.encode_frame(
                &mut stdout_lock,
                tw as usize,
                th as usize,
                &rgb_data,
                cx,
                cy,
            )?;
        }
        protocol::ImageProtocol::Braille => {
            let mut enc = braille::BrailleEncoder::new();
            enc.encode_frame(
                &mut stdout_lock,
                tw as usize,
                th as usize,
                &rgb_data,
                cx,
                cy,
            )?;
        }
        protocol::ImageProtocol::Ascii => {
            let mut enc = ascii::AsciiEncoder::new();
            enc.encode_frame(
                &mut stdout_lock,
                tw as usize,
                th as usize,
                &rgb_data,
                cx,
                cy,
            )?;
        }
    }

    Ok(())
}

fn run_video(config: &args::Config, protocol: protocol::ImageProtocol) -> Result<()> {
    ffmpeg::init()?;
    ffmpeg::log::set_level(ffmpeg::log::Level::Error);

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::Release);
    })?;

    let ictx = ffmpeg::format::input(&config.path)?;
    let video_stream = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or(error::Error::NoVideoStream)?;
    let video_stream_index = video_stream.index();

    let decoder = video::VideoDecoder::new(&video_stream)?;
    let (orig_width, orig_height) = decoder.original_dimensions();

    let (target_width, target_height) =
        terminal::fit_dimensions(orig_width, orig_height, config.scale, config.size, protocol);

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

    let preview_mode = is_fzf_preview();
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
            preview_mode,
            center: config.center,
        },
    )?;

    if preview_mode && config.verbose {
        eprintln!("fzf preview mode: limiting to first 5 seconds");
    }

    player.run(ictx, video_stream_index, audio_sender, running.clone())?;

    if let Some(mut audio) = audio_player {
        audio.stop();
    }

    Ok(())
}
