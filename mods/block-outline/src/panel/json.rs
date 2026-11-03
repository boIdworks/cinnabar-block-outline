//! Small JSON writers that avoid the costly float formatting in frame callbacks.

use super::*;

pub(super) fn vector(out: &mut String, values: [f32; 4]) {
    out.push('[');
    for (index, value) in values.into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        fixed(out, value);
    }
    out.push(']');
}

/// Appends `value` with at most three decimals, using only integer arithmetic.
pub(super) fn fixed(out: &mut String, value: f32) {
    let milli = (value * 1000.0).round();
    if !milli.is_finite() {
        out.push('0');
        return;
    }
    let milli = milli as i64;
    if milli < 0 {
        out.push('-');
    }
    let milli = milli.unsigned_abs();
    digits(out, milli / 1000);
    let mut fraction = milli % 1000;
    if fraction != 0 {
        out.push('.');
        let mut scale = 100;
        while fraction != 0 {
            out.push(char::from(b'0' + (fraction / scale) as u8));
            fraction %= scale;
            scale /= 10;
        }
    }
}

pub(super) fn digits(out: &mut String, mut value: u64) {
    let mut buffer = [0u8; 20];
    let mut at = buffer.len();
    loop {
        at -= 1;
        buffer[at] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    for &digit in &buffer[at..] {
        out.push(char::from(digit));
    }
}

/// Escapes `text` as a JSON string literal.
pub(super) fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
