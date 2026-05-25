use crate::sixel::{DiffusionMethod, Quality, parse_diffusion, parse_quality};
use clap::{
    Parser,
    builder::{Styles, styling::AnsiColor},
};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold())
    .usage(AnsiColor::Yellow.on_default().bold())
    .literal(AnsiColor::Cyan.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Debug, Parser)]
#[command(version, about = "A terminal media player", long_about = None, styles = STYLES)]
pub struct Cli {
    /// Video/ Image file path
    pub path: Option<String>,

    /// Scale factor
    #[arg(short, long, default_value = "1.0")]
    pub scale: f32,

    /// Number of colors 2-256 (Sixel only)
    #[arg(short, long, default_value = "255")]
    pub colors: u8,

    /// Dithering method: none, atkinson, fs, stucki, burkes, jajuni, auto (Sixel only)
    #[arg(short, long, default_value = "auto", value_parser = parse_diffusion)]
    pub diffusion: DiffusionMethod,

    /// Quality level: low, high, full, auto (Sixel only)
    #[arg(short, long, default_value = "auto", value_parser = parse_quality)]
    pub quality: Quality,

    /// Force protocol: sixel, kitty, halfblock, braille, ascii, auto
    #[arg(short, long)]
    pub protocol: Option<String>,

    /// Enable audio playback
    #[arg(short, long)]
    pub audio: bool,

    /// Show status line
    #[arg(short, long)]
    pub verbose: bool,

    /// Output size in characters (e.g., 80x40)
    #[arg(long)]
    pub size: Option<String>,

    /// Center the output on screen
    #[arg(short = 'C', long)]
    pub center: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

pub fn parse_size(s: &str) -> Option<(u32, u32)> {
    let (w, h) = s.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
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
    pub size: Option<(u32, u32)>,
    pub center: bool,
}

impl From<Cli> for Config {
    fn from(cli: Cli) -> Self {
        let colors = if (2..=255).contains(&cli.colors) {
            cli.colors
        } else {
            println!("Colors must be between 2 and 255, using default 255");
            255
        };

        let size = cli.size.as_deref().and_then(parse_size);

        Config {
            path: cli.path.unwrap_or_default(),
            scale: cli.scale,
            colors,
            diffusion: cli.diffusion,
            quality: cli.quality,
            force_protocol: cli.protocol,
            verbose: cli.verbose,
            audio: cli.audio,
            size,
            center: cli.center,
        }
    }
}
