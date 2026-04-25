use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("FFmpeg error: {0}")]
    Ffmpeg(String),

    #[error("No video stream found")]
    NoVideoStream,

    #[error("Sixel encoder error: {0}")]
    Sixel(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Scaling not set")]
    ScalingNotSet,

    #[error("Args error: {0}")]
    Args(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<ffmpeg_next::Error> for Error {
    fn from(e: ffmpeg_next::Error) -> Self {
        Error::Ffmpeg(e.to_string())
    }
}

impl From<ctrlc::Error> for Error {
    fn from(e: ctrlc::Error) -> Self {
        Error::Io(std::io::Error::other(e.to_string()))
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Args(s)
    }
}
