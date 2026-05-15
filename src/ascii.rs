use std::io::{self, Write};

const RAMP: &[u8] = b" .:-=+*#%@";

pub struct AsciiEncoder;

impl AsciiEncoder {
    pub fn new() -> Self {
        Self
    }

    pub fn encode_frame<W: Write>(
        &mut self,
        writer: &mut W,
        width: usize,
        height: usize,
        rgb_data: &[u8],
        x_off: u32,
        y_off: u32,
    ) -> io::Result<()> {
        if width == 0 || height == 0 || rgb_data.is_empty() {
            return Ok(());
        }

        for y in 0..height {
            let row = y_off + y as u32 + 1;
            write!(writer, "\x1b[{};{}H", row, x_off + 1)?;
            for x in 0..width {
                let offset = (y * width + x) * 3;
                let r = rgb_data[offset];
                let g = rgb_data[offset + 1];
                let b = rgb_data[offset + 2];

                let luminance = (r as u32 * 77 + g as u32 * 150 + b as u32 * 29) >> 8;
                let idx = luminance * (RAMP.len() - 1) as u32 / 255;
                let ch = RAMP[idx as usize] as char;

                let fg = if luminance > 140 { "30" } else { "37" };
                write!(writer, "\x1b[48;2;{};{};{}m\x1b[{}m{}", r, g, b, fg, ch)?;
            }
            write!(writer, "\x1b[0m")?;
        }

        Ok(())
    }
}
