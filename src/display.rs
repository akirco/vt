use std::io::{self, Write};

pub struct CursorGuard;

impl Drop for CursorGuard {
    fn drop(&mut self) {
        let _ = std::io::stdout().write_all(b"\x1b[?25h\n");
        let _ = std::io::stdout().flush();
    }
}

pub fn clear_screen<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[2J")
}

pub fn hide_cursor<W: Write>(w: &mut W) -> io::Result<()> {
    write!(w, "\x1b[?25l")
}

pub fn move_cursor_home<W: Write>(w: &mut W) -> io::Result<()> {
    write!(w, "\x1b[H")
}

#[allow(clippy::too_many_arguments)]
pub fn write_status_line<W: Write>(
    w: &mut W,
    row: u32,
    fps: f64,
    frame_count: u32,
    elapsed: f64,
    width: u32,
    height: u32,
    colors: u8,
    protocol: &str,
    diffusion: &str,
) -> io::Result<()> {
    write!(
        w,
        "\x1b[{};1H\x1b[1mFPS:{:.1} F:{} T:{:.1}s {}x{} C:{} P:{} d:{}\x1b[0m\r",
        row, fps, frame_count, elapsed, width, height, colors, protocol, diffusion
    )
}
