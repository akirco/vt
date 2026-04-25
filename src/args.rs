use crate::error::{Error, Result};
use sixel_rs::optflags::{DiffusionMethod, Quality};

pub struct Config {
    pub path: String,
    pub scale: f32,
    pub colors: u8,
    pub diffusion: DiffusionMethod,
    pub quality: Quality,
    pub force_protocol: Option<String>,
    pub verbose: bool,
    pub audio: bool,
}

pub fn parse_args() -> Result<Config> {
    let args: Vec<String> = std::env::args().collect();

    let mut path = None;
    let mut scale = 1.0f32;
    let mut colors = 255u8;
    let mut diffusion = DiffusionMethod::Auto;
    let mut quality = Quality::Auto;
    let mut force_protocol = None;
    let mut verbose = false;
    let mut audio = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--scale" => {
                if i + 1 < args.len() {
                    scale = args[i + 1].parse().map_err(|e| Error::Args(format!("{}", e)))?;
                    i += 1;
                }
            }
            "-c" | "--colors" => {
                if i + 1 < args.len() {
                    let val: u8 = args[i + 1].parse().map_err(|e| Error::Args(format!("{}", e)))?;
                    if (2..=255).contains(&val) {
                        colors = val;
                    } else {
                        eprintln!("Colors must be between 2 and 255, using default 255");
                    }
                    i += 1;
                }
            }
            "-d" | "--diffusion" => {
                if i + 1 < args.len() {
                    diffusion = match args[i + 1].to_lowercase().as_str() {
                        "none" => DiffusionMethod::None,
                        "atkinson" => DiffusionMethod::Atkinson,
                        "fs" => DiffusionMethod::FS,
                        "stucki" => DiffusionMethod::Stucki,
                        "burkes" => DiffusionMethod::Burkes,
                        "jajuni" => DiffusionMethod::Jajuni,
                        _ => DiffusionMethod::Auto,
                    };
                    i += 1;
                }
            }
            "-q" | "--quality" => {
                if i + 1 < args.len() {
                    quality = match args[i + 1].to_lowercase().as_str() {
                        "low" => Quality::Low,
                        "high" => Quality::High,
                        "full" => Quality::Full,
                        _ => Quality::Auto,
                    };
                    i += 1;
                }
            }
            "-p" | "--protocol" => {
                if i + 1 < args.len() {
                    force_protocol = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-a" | "--audio" => {
                audio = true;
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            "-h" | "--help" => {
                println!("Usage: vt <video_path> [options]");
                println!("Options:");
                println!("  -s, --scale <n>       Scale factor (default: 1.0)");
                println!("  -c, --colors <n>      Number of colors 2-256 (Sixel only)");
                println!("  -d, --diffusion <m>   Dithering: none, atkinson, fs, stucki, burkes, jajuni (Sixel only)");
                println!("  -q, --quality <q>     Quality: low, high, full, auto (Sixel only)");
                println!("  -p, --protocol <p>    Protocol: sixel, kitty, auto");
                println!("  -a, --audio           Enable audio playback");
                println!("  -v, --verbose         Show status line");
                std::process::exit(0);
            }
            _ => {
                if path.is_none() {
                    path = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    let path = path.ok_or_else(|| Error::Args("Please provide a video path".to_string()))?;
    Ok(Config {
        path,
        scale,
        colors,
        diffusion,
        quality,
        force_protocol,
        verbose,
        audio,
    })
}
