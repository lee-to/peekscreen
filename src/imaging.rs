use std::io::Cursor;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::DynamicImage;
use image::imageops::FilterType;
use tracing::{debug, info, instrument};

/// Supported image output formats.
#[derive(Debug, Clone, Copy, Default)]
pub enum ImageFormat {
    #[default]
    Png,
    Jpeg,
}

impl ImageFormat {
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
        }
    }

    pub fn from_str_opt(s: Option<&str>) -> Self {
        match s.map(|s| s.to_lowercase()).as_deref() {
            Some("jpeg" | "jpg") => ImageFormat::Jpeg,
            _ => ImageFormat::Png,
        }
    }
}

/// Default maximum width for screenshots.
pub const DEFAULT_MAX_WIDTH: u32 = 1920;

/// Resize image proportionally if wider than `max_width`.
#[instrument(skip(img), fields(original_w = img.width(), original_h = img.height()))]
pub fn resize_image(img: &DynamicImage, max_width: u32) -> DynamicImage {
    if img.width() <= max_width {
        debug!("Image within max_width, no resize needed");
        return img.clone();
    }
    let ratio = max_width as f64 / img.width() as f64;
    let new_height = (img.height() as f64 * ratio) as u32;
    info!(new_w = max_width, new_h = new_height, "Resizing image");
    img.resize(max_width, new_height, FilterType::Lanczos3)
}

/// Encode image to bytes in the specified format.
#[instrument(skip(img), fields(w = img.width(), h = img.height(), format = ?format))]
pub fn encode_image(img: &DynamicImage, format: ImageFormat) -> anyhow::Result<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    match format {
        ImageFormat::Png => img.write_to(&mut buf, image::ImageFormat::Png)?,
        ImageFormat::Jpeg => img.write_to(&mut buf, image::ImageFormat::Jpeg)?,
    }
    let bytes = buf.into_inner();
    debug!(size_bytes = bytes.len(), "Image encoded");
    Ok(bytes)
}

/// Full pipeline: resize → encode → base64.
/// Returns `(base64_string, mime_type)`.
#[instrument(skip(img), fields(w = img.width(), h = img.height()))]
pub fn image_to_base64(
    img: &DynamicImage,
    max_width: Option<u32>,
    format: ImageFormat,
) -> anyhow::Result<(String, &'static str)> {
    let max_w = max_width.unwrap_or(DEFAULT_MAX_WIDTH);
    let resized = resize_image(img, max_w);
    let bytes = encode_image(&resized, format)?;
    let b64 = STANDARD.encode(&bytes);
    info!(
        base64_len = b64.len(),
        mime = format.mime_type(),
        "Image converted to base64"
    );
    Ok((b64, format.mime_type()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    fn make_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::new(width, height))
    }

    #[test]
    fn resize_no_op_when_within_max() {
        let img = make_test_image(800, 600);
        let result = resize_image(&img, 1920);
        assert_eq!(result.width(), 800);
        assert_eq!(result.height(), 600);
    }

    #[test]
    fn resize_scales_proportionally() {
        let img = make_test_image(3840, 2160);
        let result = resize_image(&img, 1920);
        assert_eq!(result.width(), 1920);
        assert_eq!(result.height(), 1080);
    }

    #[test]
    fn encode_png() {
        let img = make_test_image(100, 100);
        let bytes = encode_image(&img, ImageFormat::Png).unwrap();
        assert!(!bytes.is_empty());
        // PNG magic bytes
        assert_eq!(&bytes[..4], &[0x89, b'P', b'N', b'G']);
    }

    #[test]
    fn encode_jpeg() {
        let img = make_test_image(100, 100);
        let bytes = encode_image(&img, ImageFormat::Jpeg).unwrap();
        assert!(!bytes.is_empty());
        // JPEG magic bytes
        assert_eq!(&bytes[..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn image_to_base64_default() {
        let img = make_test_image(200, 100);
        let (b64, mime) = image_to_base64(&img, None, ImageFormat::Png).unwrap();
        assert!(!b64.is_empty());
        assert_eq!(mime, "image/png");
    }

    #[test]
    fn image_to_base64_with_resize() {
        let img = make_test_image(4000, 2000);
        let (b64, mime) = image_to_base64(&img, Some(800), ImageFormat::Jpeg).unwrap();
        assert!(!b64.is_empty());
        assert_eq!(mime, "image/jpeg");
    }

    #[test]
    fn image_format_from_str() {
        assert!(matches!(ImageFormat::from_str_opt(None), ImageFormat::Png));
        assert!(matches!(
            ImageFormat::from_str_opt(Some("png")),
            ImageFormat::Png
        ));
        assert!(matches!(
            ImageFormat::from_str_opt(Some("jpeg")),
            ImageFormat::Jpeg
        ));
        assert!(matches!(
            ImageFormat::from_str_opt(Some("jpg")),
            ImageFormat::Jpeg
        ));
        assert!(matches!(
            ImageFormat::from_str_opt(Some("JPEG")),
            ImageFormat::Jpeg
        ));
    }
}
