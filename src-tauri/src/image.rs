use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[cfg(test)]
use std::sync::atomic::{AtomicU32, Ordering};

pub fn image_dims(path: &Path) -> Option<(u32, u32)> {
    let mut f = std::fs::File::open(path).ok()?;
    let mut head = [0u8; 26];
    let n = f.read(&mut head).ok()?;
    if n < 10 {
        return None;
    }

    // PNG: 8-byte signature + IHDR chunk (4 len + 4 type + 4 w + 4 h)
    if n >= 24 && &head[..8] == b"\x89PNG\r\n\x1a\n" && &head[12..16] == b"IHDR" {
        let w = u32::from_be_bytes(head[16..20].try_into().ok()?);
        let h = u32::from_be_bytes(head[20..24].try_into().ok()?);
        return Some((w, h));
    }

    // GIF: 6-byte signature + 2-byte w + 2-byte h (little-endian)
    if n >= 10 && (&head[..6] == b"GIF87a" || &head[..6] == b"GIF89a") {
        let w = u16::from_le_bytes(head[6..8].try_into().ok()?) as u32;
        let h = u16::from_le_bytes(head[8..10].try_into().ok()?) as u32;
        return Some((w, h));
    }

    // JPEG: FF D8 marker
    if n >= 2 && head[0] == 0xFF && head[1] == 0xD8 {
        return jpeg_dims(f);
    }

    None
}

fn jpeg_dims(mut f: std::fs::File) -> Option<(u32, u32)> {
    f.seek(SeekFrom::Start(2)).ok()?;
    loop {
        let mut b = [0u8; 1];
        // Scan for FF marker prefix
        loop {
            f.read_exact(&mut b).ok()?;
            if b[0] == 0xFF {
                break;
            }
        }
        // Skip fill bytes
        loop {
            f.read_exact(&mut b).ok()?;
            if b[0] != 0xFF {
                break;
            }
        }
        let marker = b[0];

        match marker {
            // SOF markers (C0-CF, excluding DHT=C4, JPG=C8, DAC=CC)
            0xC0..=0xCF if marker != 0xC4 && marker != 0xC8 && marker != 0xCC => {
                // 2-byte segment length + 1-byte precision + 2-byte height + 2-byte width
                let mut data = [0u8; 7];
                f.read_exact(&mut data).ok()?;
                let h = u16::from_be_bytes([data[3], data[4]]) as u32;
                let w = u16::from_be_bytes([data[5], data[6]]) as u32;
                return Some((w, h));
            }
            // Markers with no length field
            0xD0..=0xD9 | 0x01 => continue,
            // All other markers: skip over segment
            _ => {
                let mut seg = [0u8; 2];
                f.read_exact(&mut seg).ok()?;
                let len = u16::from_be_bytes(seg) as i64;
                if len < 2 {
                    return None;
                }
                f.seek(SeekFrom::Current(len - 2)).ok()?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn tmp_path(ext: &str) -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("backdrop_imgtest_{id}.{ext}"))
    }

    #[test]
    fn png_dims_returned() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]);
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&1920u32.to_be_bytes());
        bytes.extend_from_slice(&1080u32.to_be_bytes());
        let path = tmp_path("png");
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(image_dims(&path), Some((1920, 1080)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn gif87a_dims_returned() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GIF87a");
        bytes.extend_from_slice(&800u16.to_le_bytes());
        bytes.extend_from_slice(&600u16.to_le_bytes());
        let path = tmp_path("gif");
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(image_dims(&path), Some((800, 600)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn gif89a_dims_returned() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GIF89a");
        bytes.extend_from_slice(&1280u16.to_le_bytes());
        bytes.extend_from_slice(&720u16.to_le_bytes());
        let path = tmp_path("gif");
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(image_dims(&path), Some((1280, 720)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn jpeg_sof0_dims_returned() {
        // Minimal JPEG: SOI then SOF0 segment directly (no APP segments)
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0xFF, 0xD8]); // SOI
        bytes.extend_from_slice(&[0xFF, 0xC0]); // SOF0
        bytes.extend_from_slice(&[0x00, 0x11]); // segment length = 17
        bytes.push(0x08); // precision
        bytes.extend_from_slice(&200u16.to_be_bytes()); // height
        bytes.extend_from_slice(&320u16.to_be_bytes()); // width
        let path = tmp_path("jpg");
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(image_dims(&path), Some((320, 200)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn jpeg_with_app0_segment_dims_returned() {
        // JPEG with an APP0 segment before SOF0, exercising the skip-segment branch
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0xFF, 0xD8]); // SOI
        bytes.extend_from_slice(&[0xFF, 0xE0]); // APP0
        bytes.extend_from_slice(&[0x00, 0x10]); // length = 16 (14 data bytes after 2-byte length)
        bytes.extend_from_slice(&[0u8; 14]); // APP0 payload
        bytes.extend_from_slice(&[0xFF, 0xC0]); // SOF0
        bytes.extend_from_slice(&[0x00, 0x11]); // length = 17
        bytes.push(0x08); // precision
        bytes.extend_from_slice(&480u16.to_be_bytes()); // height
        bytes.extend_from_slice(&640u16.to_be_bytes()); // width
        let path = tmp_path("jpg");
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(image_dims(&path), Some((640, 480)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn unknown_format_returns_none() {
        let path = tmp_path("bin");
        std::fs::write(&path, b"not an image at all").unwrap();
        assert_eq!(image_dims(&path), None);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn missing_file_returns_none() {
        let path = std::path::Path::new("/tmp/backdrop_nonexistent_imgtest_file.png");
        assert_eq!(image_dims(path), None);
    }

    #[test]
    fn too_short_returns_none() {
        let path = tmp_path("bin");
        std::fs::write(&path, b"\x89PNG").unwrap(); // less than 10 bytes
        assert_eq!(image_dims(&path), None);
        std::fs::remove_file(&path).ok();
    }
}
