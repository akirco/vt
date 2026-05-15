use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageProtocol {
    Sixel,
    Kitty,
    HalfBlock,
    Braille,
    Ascii,
}

pub fn determine_protocol(force: Option<&str>) -> ImageProtocol {
    if let Some(forced) = force {
        match forced.to_lowercase().as_str() {
            "kitty" => return ImageProtocol::Kitty,
            "sixel" => return ImageProtocol::Sixel,
            "halfblock" => return ImageProtocol::HalfBlock,
            "braille" => return ImageProtocol::Braille,
            "ascii" => return ImageProtocol::Ascii,
            _ => {}
        }
    }
    if is_kitty_available() {
        ImageProtocol::Kitty
    } else {
        ImageProtocol::Sixel
    }
}

fn is_kitty_available() -> bool {
    env::var("KITTY_WINDOW_ID").is_ok()
}

pub fn is_fzf_preview() -> bool {
    std::env::var("FZF_PREVIEW").is_ok() || std::env::var("FZF_PREVIEW_LINES").is_ok()
}

pub fn cell_size() -> (u32, u32) {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::ioctl(std::io::stdout().as_raw_fd(), libc::TIOCGWINSZ, &mut ws) };
        if ret == 0 && ws.ws_xpixel > 0 && ws.ws_ypixel > 0 && ws.ws_col > 0 && ws.ws_row > 0 {
            let cw = ws.ws_xpixel as u32 / ws.ws_col as u32;
            let ch = ws.ws_ypixel as u32 / ws.ws_row as u32;
            if cw > 0 && ch > 0 {
                return (cw, ch);
            }
        }
    }
    (9, 18)
}

pub fn fit_dimensions(
    orig_w: u32,
    orig_h: u32,
    scale: f32,
    size: Option<(u32, u32)>,
    protocol: ImageProtocol,
) -> (u32, u32) {
    let (cell_w, cell_h) = cell_size();
    let (px_per_col, px_per_row) = match protocol {
        ImageProtocol::HalfBlock => (1, 2),
        ImageProtocol::Braille => (2, 4),
        ImageProtocol::Ascii => (1, 1),
        _ => (cell_w, cell_h),
    };

    let (phys_bounds_w, phys_bounds_h) = if let Some((cw, ch)) = size {
        (cw * cell_w, ch * cell_h)
    } else {
        let tw = (orig_w as f32 * scale) as u32;
        let th = (orig_h as f32 * scale) as u32;

        let (cols, rows) = terminal_size::terminal_size()
            .map(|(w, h)| (w.0 as u32, h.0 as u32))
            .unwrap_or((80, 24));

        let max_phys_w = cols * cell_w;
        let max_phys_h = rows.saturating_sub(2) * cell_h;

        if tw * cell_w / px_per_col <= max_phys_w && th * cell_h / px_per_row <= max_phys_h {
            return (tw, th);
        }
        (max_phys_w, max_phys_h)
    };

    let fit = (phys_bounds_w as f64 / orig_w as f64).min(phys_bounds_h as f64 / orig_h as f64);

    let target_w = (orig_w as f64 * fit * px_per_col as f64 / cell_w as f64) as u32;
    let target_h = (orig_h as f64 * fit * px_per_row as f64 / cell_h as f64) as u32;

    (target_w.max(1), target_h.max(1))
}
