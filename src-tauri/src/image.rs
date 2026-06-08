use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

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
