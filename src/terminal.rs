use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageProtocol {
    Sixel,
    Kitty,
}

pub fn determine_protocol(force: Option<&str>) -> ImageProtocol {
    if let Some(forced) = force {
        match forced.to_lowercase().as_str() {
            "kitty" => return ImageProtocol::Kitty,
            "sixel" => return ImageProtocol::Sixel,
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
