use std::fmt::Write;
use std::io::{self, Write as IoWrite};

pub struct BrailleEncoder;

impl BrailleEncoder {
    pub fn new() -> Self {
        Self
    }

    pub fn encode_frame<W: IoWrite>(
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

        let cap = (width / 2) * 24 + 16;
        let mut buf = String::with_capacity(cap);

        for (i, by) in (0..height).step_by(4).enumerate() {
            buf.clear();
            let _ = write!(buf, "\x1b[{};{}H", y_off + i as u32 + 1, x_off + 1);
            for bx in (0..width).step_by(2) {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut count = 0u32;

                for dy in 0..4 {
                    let py = by + dy;
                    if py >= height {
                        continue;
                    }
                    for dx in 0..2 {
                        let px = bx + dx;
                        if px >= width {
                            continue;
                        }
                        let offset = (py * width + px) * 3;
                        r_sum += rgb_data[offset] as u32;
                        g_sum += rgb_data[offset + 1] as u32;
                        b_sum += rgb_data[offset + 2] as u32;
                        count += 1;
                    }
                }

                let r_avg = r_sum.checked_div(count);
                let g_avg = g_sum.checked_div(count);
                let b_avg = b_sum.checked_div(count);
                if let (Some(r), Some(g), Some(b)) = (r_avg, g_avg, b_avg) {
                    let _ = write!(
                        buf,
                        "\x1b[38;2;{};{};{}m\u{28ff}",
                        r as u8, g as u8, b as u8
                    );
                }
            }
            buf.push_str("\x1b[0m");
            writer.write_all(buf.as_bytes())?;
        }

        Ok(())
    }
}
