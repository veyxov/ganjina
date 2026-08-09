use std::io::Cursor;

/// EXIF taken-at is stored as local time + optional UTC offset, never collapsed
/// to a single instant — most cameras record local time with no timezone, and
/// assuming UTC silently corrupts sort order whenever the camera's clock/tz was
/// wrong (a documented recurring issue in both Immich and PhotoPrism).
#[derive(Default, Debug)]
pub struct ExifData {
    pub taken_at_local: Option<chrono::NaiveDateTime>,
    pub taken_at_offset_minutes: Option<i32>,
    pub camera: Option<String>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
}

/// Only JPEG/TIFF carry EXIF in a form `kamadak-exif` reads reliably. HEIC EXIF
/// extraction is skipped for now (kamadak-exif doesn't parse the HEIF box
/// container) — thumbnails matter more urgently for HEIC than metadata, so this
/// is an accepted gap rather than a blocker.
pub fn extract(bytes: &[u8], content_type: &str) -> Option<ExifData> {
    if !matches!(content_type, "image/jpeg" | "image/tiff") {
        return None;
    }

    let exif = exif::Reader::new()
        .read_from_container(&mut Cursor::new(bytes))
        .ok()?;

    let mut data = ExifData::default();

    if let Some(field) = exif
        .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
        .or_else(|| exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY))
    {
        if let exif::Value::Ascii(ref vec) = field.value {
            if let Some(s) = vec.first() {
                if let Ok(s) = std::str::from_utf8(s) {
                    data.taken_at_local =
                        chrono::NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S").ok();
                }
            }
        }
    }

    if let Some(field) = exif.get_field(exif::Tag::OffsetTimeOriginal, exif::In::PRIMARY) {
        if let exif::Value::Ascii(ref vec) = field.value {
            if let Some(s) = vec.first() {
                if let Ok(s) = std::str::from_utf8(s) {
                    data.taken_at_offset_minutes = parse_offset_minutes(s);
                }
            }
        }
    }

    if let Some(field) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY) {
        data.camera = Some(field.display_value().to_string().trim_matches('"').to_string());
    }

    if let (Some(lat), Some(lon)) = (
        gps_coord(&exif, exif::Tag::GPSLatitude, exif::Tag::GPSLatitudeRef),
        gps_coord(&exif, exif::Tag::GPSLongitude, exif::Tag::GPSLongitudeRef),
    ) {
        data.gps_lat = Some(lat);
        data.gps_lon = Some(lon);
    }

    Some(data)
}

/// EXIF OffsetTimeOriginal is a string like "+02:00" or "-05:30".
fn parse_offset_minutes(s: &str) -> Option<i32> {
    let s = s.trim();
    let (sign, rest) = match s.chars().next()? {
        '+' => (1, &s[1..]),
        '-' => (-1, &s[1..]),
        _ => return None,
    };
    let (h, m) = rest.split_once(':')?;
    let h: i32 = h.parse().ok()?;
    let m: i32 = m.parse().ok()?;
    Some(sign * (h * 60 + m))
}

fn gps_coord(exif: &exif::Exif, tag: exif::Tag, ref_tag: exif::Tag) -> Option<f64> {
    let field = exif.get_field(tag, exif::In::PRIMARY)?;
    let exif::Value::Rational(ref vec) = field.value else {
        return None;
    };
    if vec.len() < 3 {
        return None;
    }
    let degrees = vec[0].to_f64() + vec[1].to_f64() / 60.0 + vec[2].to_f64() / 3600.0;

    let is_negative = exif
        .get_field(ref_tag, exif::In::PRIMARY)
        .map(|f| matches!(f.display_value().to_string().as_str(), "S" | "W"))
        .unwrap_or(false);

    Some(if is_negative { -degrees } else { degrees })
}
