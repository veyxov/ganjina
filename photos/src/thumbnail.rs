const MAX_EDGE: u32 = 400;

pub struct Thumbnail {
    pub bytes: Vec<u8>,
    /// Dimensions of the original (not the thumbnail) — the gallery layout
    /// needs the real aspect ratio, and we've already decoded the full image
    /// here, so returning it costs nothing extra.
    pub width: u32,
    pub height: u32,
}

/// Generates a JPEG thumbnail for a photo or video, plus the original's pixel
/// dimensions. Runs blocking work (subprocess calls, image decode/resize) on a
/// blocking thread pool — this is always called from a background job, never
/// the request path.
pub async fn generate(bytes: Vec<u8>, content_type: String) -> anyhow::Result<Thumbnail> {
    tokio::task::spawn_blocking(move || generate_blocking(&bytes, &content_type)).await?
}

fn generate_blocking(bytes: &[u8], content_type: &str) -> anyhow::Result<Thumbnail> {
    let img = if content_type.starts_with("video/") {
        decode_video_frame(bytes, content_type)?
    } else if matches!(content_type, "image/heic" | "image/heif") {
        decode_heic(bytes)?
    } else {
        image::load_from_memory(bytes)?
    };

    let (width, height) = (img.width(), img.height());
    let thumb = img.thumbnail(MAX_EDGE, MAX_EDGE);
    let mut out = Vec::new();
    thumb.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Jpeg)?;
    Ok(Thumbnail { bytes: out, width, height })
}

/// Shells out to ffmpeg to grab one frame — no Rust video-decoding dependency
/// needed for a thumbnail.
fn decode_video_frame(bytes: &[u8], content_type: &str) -> anyhow::Result<image::DynamicImage> {
    let ext = match content_type {
        "video/quicktime" => "mov",
        _ => "mp4",
    };
    let dir = tempfile::tempdir()?;
    let input_path = dir.path().join(format!("input.{ext}"));
    let output_path = dir.path().join("frame.jpg");
    std::fs::write(&input_path, bytes)?;

    let status = std::process::Command::new("ffmpeg")
        .args(["-y", "-i"])
        .arg(&input_path)
        .args(["-vframes", "1", "-q:v", "3"])
        .arg(&output_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    anyhow::ensure!(status.success(), "ffmpeg frame extraction failed");

    Ok(image::open(&output_path)?)
}

/// Shells out to `heif-convert` (ships with libheif) rather than binding
/// libheif's C API from Rust — one less FFI surface, same tool already needed
/// as a system dependency for HEIC support.
fn decode_heic(bytes: &[u8]) -> anyhow::Result<image::DynamicImage> {
    let dir = tempfile::tempdir()?;
    let input_path = dir.path().join("input.heic");
    let output_path = dir.path().join("output.png");
    std::fs::write(&input_path, bytes)?;

    let status = std::process::Command::new("heif-convert")
        .arg(&input_path)
        .arg(&output_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    anyhow::ensure!(status.success(), "heif-convert failed");

    Ok(image::open(&output_path)?)
}

