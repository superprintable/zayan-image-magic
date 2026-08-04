use image::imageops::{self, FilterType};
use image::{DynamicImage, GenericImageView, ImageEncoder, ImageFormat, Rgba, RgbaImage};
use imgref::Img;
use ravif::Encoder as AvifEncoder;
use rgb::RGBA8;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("{0}")]
    Message(String),
}

impl Serialize for ConvertError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type ConvertResult<T> = Result<T, ConvertError>;

fn err(msg: impl Into<String>) -> ConvertError {
    ConvertError::Message(msg.into())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Webp,
    Avif,
    Jpeg,
    Jpg,
    Png,
    Gif,
    Bmp,
    Tiff,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Webp => "webp",
            OutputFormat::Avif => "avif",
            OutputFormat::Jpeg | OutputFormat::Jpg => "jpg",
            OutputFormat::Png => "png",
            OutputFormat::Gif => "gif",
            OutputFormat::Bmp => "bmp",
            OutputFormat::Tiff => "tiff",
        }
    }

    pub fn supports_alpha(&self) -> bool {
        matches!(
            self,
            OutputFormat::Webp | OutputFormat::Avif | OutputFormat::Png | OutputFormat::Gif
        )
    }

    pub fn supports_lossless(&self) -> bool {
        matches!(
            self,
            OutputFormat::Webp
                | OutputFormat::Avif
                | OutputFormat::Png
                | OutputFormat::Bmp
                | OutputFormat::Tiff
        )
    }

    pub fn from_str_loose(s: &str) -> ConvertResult<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "webp" => Ok(OutputFormat::Webp),
            "avif" => Ok(OutputFormat::Avif),
            "jpeg" => Ok(OutputFormat::Jpeg),
            "jpg" => Ok(OutputFormat::Jpg),
            "png" => Ok(OutputFormat::Png),
            "gif" => Ok(OutputFormat::Gif),
            "bmp" => Ok(OutputFormat::Bmp),
            "tiff" | "tif" => Ok(OutputFormat::Tiff),
            other => Err(err(format!("Unsupported output format: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOptions {
    pub format: String,
    pub quality: u8,
    pub lossless: bool,
    pub background: Option<[u8; 3]>,
    pub suffix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub path: String,
    pub file_name: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub size_bytes: u64,
    pub has_alpha: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertSuccess {
    pub source_path: String,
    pub output_path: String,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertFailure {
    pub source_path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub successes: Vec<ConvertSuccess>,
    pub failures: Vec<ConvertFailure>,
}

fn read_exif_orientation(path: &Path) -> u32 {
    let Ok(file) = std::fs::File::open(path) else {
        return 1;
    };
    let mut bufreader = std::io::BufReader::new(&file);
    let Ok(exif) = exif::Reader::new().read_from_container(&mut bufreader) else {
        return 1;
    };
    match exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
        Some(field) => field.value.get_uint(0).unwrap_or(1).clamp(1, 8),
        None => 1,
    }
}

fn apply_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => DynamicImage::ImageRgba8(imageops::flip_horizontal(&img)),
        3 => DynamicImage::ImageRgba8(imageops::rotate180(&img)),
        4 => DynamicImage::ImageRgba8(imageops::flip_vertical(&img)),
        5 => {
            let rotated = imageops::rotate90(&img);
            DynamicImage::ImageRgba8(imageops::flip_horizontal(&rotated))
        }
        6 => DynamicImage::ImageRgba8(imageops::rotate90(&img)),
        7 => {
            let rotated = imageops::rotate270(&img);
            DynamicImage::ImageRgba8(imageops::flip_horizontal(&rotated))
        }
        8 => DynamicImage::ImageRgba8(imageops::rotate270(&img)),
        _ => img,
    }
}

fn load_image(path: &Path) -> ConvertResult<DynamicImage> {
    let bytes =
        fs::read(path).map_err(|e| err(format!("Failed to read {}: {e}", path.display())))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| err(format!("Failed to decode {}: {e}", path.display())))?;
    let orientation = read_exif_orientation(path);
    Ok(apply_orientation(img, orientation))
}

fn detect_format_label(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_uppercase())
        .unwrap_or_else(|| "UNKNOWN".into())
}

fn has_meaningful_alpha(img: &DynamicImage) -> bool {
    let rgba = img.to_rgba8();
    rgba.pixels().any(|p| p.0[3] < 255)
}

fn flatten_on_background(img: &DynamicImage, bg: [u8; 3]) -> RgbaImage {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut out = RgbaImage::new(w, h);
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let a = pixel.0[3] as f32 / 255.0;
        let r = (pixel.0[0] as f32 * a + bg[0] as f32 * (1.0 - a)).round() as u8;
        let g = (pixel.0[1] as f32 * a + bg[1] as f32 * (1.0 - a)).round() as u8;
        let b = (pixel.0[2] as f32 * a + bg[2] as f32 * (1.0 - a)).round() as u8;
        out.put_pixel(x, y, Rgba([r, g, b, 255]));
    }
    out
}

fn prepare_pixels(img: &DynamicImage, format: &OutputFormat, bg: [u8; 3]) -> RgbaImage {
    if format.supports_alpha() {
        img.to_rgba8()
    } else {
        flatten_on_background(img, bg)
    }
}

fn encode_image(
    pixels: &RgbaImage,
    format: &OutputFormat,
    quality: u8,
    lossless: bool,
) -> ConvertResult<Vec<u8>> {
    let (width, height) = pixels.dimensions();
    let quality = quality.clamp(1, 100);

    match format {
        OutputFormat::Png => {
            let mut buf = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut buf);
            encoder
                .write_image(
                    pixels.as_raw(),
                    width,
                    height,
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|e| err(format!("PNG encode failed: {e}")))?;
            Ok(buf)
        }
        OutputFormat::Jpeg | OutputFormat::Jpg => {
            let rgb = DynamicImage::ImageRgba8(pixels.clone()).to_rgb8();
            let mut buf = Vec::new();
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
            encoder
                .encode(rgb.as_raw(), width, height, image::ExtendedColorType::Rgb8)
                .map_err(|e| err(format!("JPEG encode failed: {e}")))?;
            Ok(buf)
        }
        OutputFormat::Bmp => {
            let rgb = DynamicImage::ImageRgba8(pixels.clone()).to_rgb8();
            let mut buf = Vec::new();
            let mut encoder = image::codecs::bmp::BmpEncoder::new(&mut buf);
            encoder
                .encode(rgb.as_raw(), width, height, image::ExtendedColorType::Rgb8)
                .map_err(|e| err(format!("BMP encode failed: {e}")))?;
            Ok(buf)
        }
        OutputFormat::Tiff => {
            let mut cursor = Cursor::new(Vec::new());
            let encoder = image::codecs::tiff::TiffEncoder::new(&mut cursor);
            encoder
                .write_image(
                    pixels.as_raw(),
                    width,
                    height,
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|e| err(format!("TIFF encode failed: {e}")))?;
            Ok(cursor.into_inner())
        }
        OutputFormat::Gif => {
            let mut buf = Vec::new();
            {
                let mut encoder = image::codecs::gif::GifEncoder::new(&mut buf);
                let frame = image::Frame::new(pixels.clone());
                encoder
                    .encode_frame(frame)
                    .map_err(|e| err(format!("GIF encode failed: {e}")))?;
            }
            Ok(buf)
        }
        OutputFormat::Webp => {
            let encoder = webp::Encoder::from_rgba(pixels.as_raw(), width, height);
            let memory = if lossless {
                encoder.encode_lossless()
            } else {
                encoder.encode(quality as f32)
            };
            Ok(memory.to_vec())
        }
        OutputFormat::Avif => {
            let rgba_pixels: Vec<RGBA8> = pixels
                .pixels()
                .map(|p| RGBA8 {
                    r: p.0[0],
                    g: p.0[1],
                    b: p.0[2],
                    a: p.0[3],
                })
                .collect();
            let img = Img::new(rgba_pixels.as_slice(), width as usize, height as usize);
            let q = if lossless { 100.0 } else { quality as f32 };
            let encoder = AvifEncoder::new()
                .with_quality(q)
                .with_speed(6)
                .with_alpha_quality(if lossless { 100.0 } else { q });
            let encoded = encoder
                .encode_rgba(img)
                .map_err(|e| err(format!("AVIF encode failed: {e}")))?;
            Ok(encoded.avif_file)
        }
    }
}

fn output_path_for(
    source: &Path,
    output_dir: &Path,
    format: &OutputFormat,
    suffix: &str,
) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let name = if suffix.is_empty() {
        format!("{stem}.{}", format.extension())
    } else {
        format!("{stem}{suffix}.{}", format.extension())
    };
    output_dir.join(name)
}

pub fn probe_image(path: String) -> ConvertResult<ImageInfo> {
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err(err(format!("File not found: {}", path.display())));
    }
    let meta = fs::metadata(&path).map_err(|e| err(e.to_string()))?;
    let img = load_image(&path)?;
    let (width, height) = img.dimensions();
    Ok(ImageInfo {
        path: path.to_string_lossy().into_owned(),
        file_name: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image")
            .to_string(),
        width,
        height,
        format: detect_format_label(&path),
        size_bytes: meta.len(),
        has_alpha: has_meaningful_alpha(&img),
    })
}

pub fn convert_image(
    source_path: String,
    output_dir: String,
    options: ConvertOptions,
) -> ConvertResult<ConvertSuccess> {
    let source = PathBuf::from(&source_path);
    let out_dir = PathBuf::from(&output_dir);
    if !source.exists() {
        return Err(err(format!("File not found: {}", source.display())));
    }
    fs::create_dir_all(&out_dir).map_err(|e| err(format!("Cannot create output folder: {e}")))?;

    let format = OutputFormat::from_str_loose(&options.format)?;
    let bg = options.background.unwrap_or([255, 255, 255]);
    let lossless = options.lossless && format.supports_lossless();
    let suffix = options.suffix.unwrap_or_default();

    let img = load_image(&source)?;
    let (width, height) = img.dimensions();
    let pixels = prepare_pixels(&img, &format, bg);
    let encoded = encode_image(&pixels, &format, options.quality, lossless)?;

    let dest = output_path_for(&source, &out_dir, &format, &suffix);
    fs::write(&dest, &encoded)
        .map_err(|e| err(format!("Failed to write {}: {e}", dest.display())))?;

    let input_bytes = fs::metadata(&source).map(|m| m.len()).unwrap_or(0);

    Ok(ConvertSuccess {
        source_path,
        output_path: dest.to_string_lossy().into_owned(),
        input_bytes,
        output_bytes: encoded.len() as u64,
        width,
        height,
        format: format.extension().to_ascii_uppercase(),
    })
}

pub fn convert_batch(
    source_paths: Vec<String>,
    output_dir: String,
    options: ConvertOptions,
) -> BatchResult {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for path in source_paths {
        match convert_image(path.clone(), output_dir.clone(), options.clone()) {
            Ok(ok) => successes.push(ok),
            Err(e) => failures.push(ConvertFailure {
                source_path: path,
                error: e.to_string(),
            }),
        }
    }

    BatchResult {
        successes,
        failures,
    }
}

pub fn preview_data_url(path: String, max_edge: u32) -> ConvertResult<String> {
    let path = PathBuf::from(&path);
    let img = load_image(&path)?;
    let (w, h) = img.dimensions();
    let max_edge = max_edge.max(64);
    let resized = if w > max_edge || h > max_edge {
        img.resize(max_edge, max_edge, FilterType::Triangle)
    } else {
        img
    };
    let mut buf = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| err(format!("Preview encode failed: {e}")))?;
    Ok(format!("data:image/png;base64,{}", base64_encode(&buf)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zayan-image-magic-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_sample_png(path: &Path) {
        let mut img = RgbaImage::new(32, 24);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = Rgba([
                (x * 8) as u8,
                (y * 10) as u8,
                180,
                if x < 8 { 128 } else { 255 },
            ]);
        }
        DynamicImage::ImageRgba8(img).save(path).unwrap();
    }

    #[test]
    fn converts_png_to_webp_jpeg_avif() {
        let dir = temp_dir();
        let src = dir.join("sample.png");
        write_sample_png(&src);

        let info = probe_image(src.to_string_lossy().into_owned()).unwrap();
        assert_eq!(info.width, 32);
        assert_eq!(info.height, 24);
        assert!(info.has_alpha);

        for (format, lossless) in [("webp", false), ("jpeg", false), ("avif", false), ("png", true)]
        {
            let out = dir.join(format);
            fs::create_dir_all(&out).unwrap();
            let result = convert_image(
                src.to_string_lossy().into_owned(),
                out.to_string_lossy().into_owned(),
                ConvertOptions {
                    format: format.into(),
                    quality: 90,
                    lossless,
                    background: Some([255, 255, 255]),
                    suffix: Some("-out".into()),
                },
            )
            .unwrap_or_else(|e| panic!("{format} convert failed: {e}"));
            assert!(PathBuf::from(&result.output_path).exists());
            assert!(result.output_bytes > 0);
        }
    }

    #[test]
    fn batch_continues_after_failure() {
        let dir = temp_dir();
        let src = dir.join("ok.png");
        write_sample_png(&src);
        let missing = dir.join("missing.png");
        let out = dir.join("batch-out");

        let result = convert_batch(
            vec![
                src.to_string_lossy().into_owned(),
                missing.to_string_lossy().into_owned(),
            ],
            out.to_string_lossy().into_owned(),
            ConvertOptions {
                format: "webp".into(),
                quality: 85,
                lossless: false,
                background: None,
                suffix: None,
            },
        );
        assert_eq!(result.successes.len(), 1);
        assert_eq!(result.failures.len(), 1);
    }
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, b) in chunk.iter().enumerate() {
            buf[i] = *b;
        }
        let n = chunk.len();
        let b0 = buf[0] as u32;
        let b1 = buf[1] as u32;
        let b2 = buf[2] as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 63) as usize] as char);
        out.push(TABLE[((triple >> 12) & 63) as usize] as char);
        if n > 1 {
            out.push(TABLE[((triple >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if n > 2 {
            out.push(TABLE[(triple & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
