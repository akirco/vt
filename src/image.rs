use crate::error::Result;

pub fn load_image(path: &str) -> Result<(image::DynamicImage, u32, u32)> {
    let img = image::open(path)?;
    let (w, h) = (img.width(), img.height());
    Ok((img, w, h))
}

pub fn resize_image(img: image::DynamicImage, target_width: u32, target_height: u32) -> Vec<u8> {
    img.resize_exact(
        target_width.max(1),
        target_height.max(1),
        image::imageops::FilterType::Triangle,
    )
    .to_rgb8()
    .to_vec()
}

pub fn is_image_extension(path: &str) -> bool {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    matches!(
        ext.as_str(),
        "jpg"
            | "jpeg"
            | "png"
            | "gif"
            | "webp"
            | "bmp"
            | "tiff"
            | "tif"
            | "ico"
            | "avif"
            | "pnm"
            | "ppm"
            | "pgm"
            | "pbm"
            | "hdr"
            | "qoi"
            | "exr"
    )
}
