use once_cell::sync::Lazy;
use std::io::Write;
use std::sync::Mutex;

// Kept alive for the whole session: on X11 the clipboard is owned by the
// process that set it, so dropping the handle right away would clear it.
static CLIPBOARD: Lazy<Mutex<Option<arboard::Clipboard>>> =
    Lazy::new(|| Mutex::new(arboard::Clipboard::new().ok()));

/// Copy text to the system clipboard.
///
/// Uses the native clipboard (arboard) and additionally emits an OSC 52
/// escape so the terminal emulator can set the clipboard itself — that is
/// what makes copying work over SSH or on headless machines. Errors only
/// when both paths fail.
pub fn copy(text: &str) -> Result<(), String> {
    let native_ok = match CLIPBOARD.lock() {
        Ok(mut guard) => match guard.as_mut() {
            Some(cb) => cb.set_text(text.to_string()).is_ok(),
            None => false,
        },
        Err(_) => false,
    };

    let osc52_ok = copy_osc52(text).is_ok();

    if native_ok || osc52_ok {
        Ok(())
    } else {
        Err("clipboard unavailable".to_string())
    }
}

fn copy_osc52(text: &str) -> Result<(), std::io::Error> {
    let mut out = std::io::stdout();
    write!(out, "\x1b]52;c;{}\x07", base64_encode(text.as_bytes()))?;
    out.flush()
}

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[n as usize & 63] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(
            base64_encode("João 3:16".as_bytes()),
            "Sm/Do28gMzoxNg=="
        );
    }
}
