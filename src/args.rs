use clap::{Parser, ValueEnum};
use sixel_rs::optflags::{DiffusionMethod, Quality};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Diffusion {
    None,
    Atkinson,
    Fs,
    Stucki,
    Burkes,
    Jajuni,
    Auto,
}

impl From<Diffusion> for DiffusionMethod {
    fn from(d: Diffusion) -> Self {
        match d {
            Diffusion::None => DiffusionMethod::None,
            Diffusion::Atkinson => DiffusionMethod::Atkinson,
            Diffusion::Fs => DiffusionMethod::FS,
            Diffusion::Stucki => DiffusionMethod::Stucki,
            Diffusion::Burkes => DiffusionMethod::Burkes,
            Diffusion::Jajuni => DiffusionMethod::Jajuni,
            Diffusion::Auto => DiffusionMethod::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum QualityLevel {
    Low,
    High,
    Full,
    Auto,
}

impl From<QualityLevel> for Quality {
    fn from(q: QualityLevel) -> Self {
        match q {
            QualityLevel::Low => Quality::Low,
            QualityLevel::High => Quality::High,
            QualityLevel::Full => Quality::Full,
            QualityLevel::Auto => Quality::Auto,
        }
    }
}

#[derive(Debug, Parser)]
#[command(version, about = "A terminal media player", long_about = None)]
pub struct Cli {
    /// Video/ Image file path
    pub path: String,

    /// Scale factor (default: 1.0)
    #[arg(short, long, default_value = "1.0")]
    pub scale: f32,

    /// Number of colors 2-256 (Sixel only)
    #[arg(short, long, default_value = "255")]
    pub colors: u8,

    /// Dithering method (Sixel only)
    #[arg(short, long, value_enum, default_value = "auto")]
    pub diffusion: Diffusion,

    /// Quality level (Sixel only)
    #[arg(short, long, value_enum, default_value = "auto")]
    pub quality: QualityLevel,

    /// Force protocol: sixel, kitty, auto
    #[arg(short, long)]
    pub protocol: Option<String>,

    /// Enable audio playback
    #[arg(short, long)]
    pub audio: bool,

    /// Show status line
    #[arg(short, long)]
    pub verbose: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

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

impl From<Cli> for Config {
    fn from(cli: Cli) -> Self {
        let colors = if (2..=255).contains(&cli.colors) {
            cli.colors
        } else {
            println!("Colors must be between 2 and 255, using default 255");
            255
        };

        Config {
            path: cli.path,
            scale: cli.scale,
            colors,
            diffusion: cli.diffusion.into(),
            quality: cli.quality.into(),
            force_protocol: cli.protocol,
            verbose: cli.verbose,
            audio: cli.audio,
        }
    }
}
