//! A one-pass reader for the flat JSON the settings file is written in.

/// A flat JSON object read in one pass: each key and where its value starts. Looking a key up
/// scans the text again for every key, and a settings file has about ninety keys and grows with
/// the logs it keeps, which once cost more fuel than `init` is given; this reads the file once and
/// builds no strings.
pub struct Fields<'a> {
    json: &'a str,
    entries: Vec<Entry<'a>>,
    /// Where the last lookup hit. Settings are read in the order they are written, so the next
    /// key asked for is nearly always the next entry.
    next: std::cell::Cell<usize>,
}

/// One key of a [`Fields`]: where its value starts, and for a text value where the text ends.
#[derive(Clone, Copy)]
pub(super) struct Entry<'a> {
    key: &'a str,
    value: usize,
    /// The offset of the closing quote when the value is text.
    text_end: Option<usize>,
}

/// The offset of the first `"` in `bytes` at or after `from`. Eight bytes at a time: this runs
/// over the whole settings file at start, most of it long text, and `str::find` costs about
/// thirteen fuel a byte there.
pub(super) fn find_quote(bytes: &[u8], from: usize) -> Option<usize> {
    const ONES: u64 = 0x0101_0101_0101_0101;
    let mut rest = bytes.get(from..)?;
    let mut at = from;
    while let Some((word, tail)) = rest.split_first_chunk::<8>() {
        let word = u64::from_le_bytes(*word) ^ (ONES * u64::from(b'"'));
        // A byte of `word` is zero where `bytes` has a quote, and this marks the first one (the
        // bytes above it may be marked wrongly, never the ones below).
        let marks = word.wrapping_sub(ONES) & !word & (ONES << 7);
        if marks != 0 {
            return Some(at + (marks.trailing_zeros() / 8) as usize);
        }
        rest = tail;
        at += 8;
    }
    // Fewer than eight bytes left.
    rest.iter()
        .position(|&byte| byte == b'"')
        .map(|offset| at + offset)
}

impl<'a> Fields<'a> {
    /// Finds the keys of `json`: a quoted string followed by a colon.
    pub fn new(json: &'a str) -> Self {
        let bytes = json.as_bytes();
        let mut entries = Vec::with_capacity(96);
        let mut at = 0;
        while let Some(open) = find_quote(bytes, at) {
            let start = open + 1;
            let Some(end) = find_quote(bytes, start) else {
                break;
            };
            at = end + 1;
            let mut value = at;
            while bytes.get(value).is_some_and(u8::is_ascii_whitespace) {
                value += 1;
            }
            if bytes.get(value) != Some(&b':') {
                // A text value, not a key.
                continue;
            }
            value += 1;
            while bytes.get(value).is_some_and(u8::is_ascii_whitespace) {
                value += 1;
            }
            // A text value is passed over now, so it is not read as a key and not found again.
            let text_end = if bytes.get(value) == Some(&b'"') {
                find_quote(bytes, value + 1)
            } else {
                None
            };
            entries.push(Entry {
                key: &json[start..end],
                value,
                text_end,
            });
            at = text_end.map_or(value, |close| close + 1);
        }
        Self {
            json,
            entries,
            next: std::cell::Cell::new(0),
        }
    }

    /// The first entry named `key` at or after the last hit.
    fn entry(&self, key: &str) -> Option<Entry<'a>> {
        let count = self.entries.len();
        let from = self.next.get();
        for step in 0..count {
            let mut index = from + step;
            if index >= count {
                index -= count;
            }
            let entry = self.entries[index];
            if entry.key == key {
                self.next.set(index + 1);
                return Some(entry);
            }
        }
        None
    }

    /// The text from `key`'s value on.
    fn get(&self, key: &str) -> Option<&'a str> {
        Some(&self.json[self.entry(key)?.value..])
    }

    /// Where `key`'s text value lies in the file, without its quotes.
    pub fn text_span(&self, key: &str) -> Option<std::ops::Range<usize>> {
        let entry = self.entry(key)?;
        Some(entry.value + 1..entry.text_end?)
    }
}

/// The text value of `key` in a settings file, for extras the settings do not hold.
pub fn text_field<'a>(fields: &Fields<'a>, key: &str) -> Option<&'a str> {
    string_field(fields, key)
}

pub(super) fn string_field<'a>(fields: &Fields<'a>, key: &str) -> Option<&'a str> {
    fields.json.get(fields.text_span(key)?)
}

pub(super) fn bool_field(fields: &Fields, key: &str) -> Option<bool> {
    let value = fields.get(key)?;
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

pub(super) fn number_field(fields: &Fields, key: &str) -> Option<f32> {
    let value = fields.get(key)?;
    let end = value
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')))
        .unwrap_or(value.len());
    value[..end].parse::<f32>().ok().filter(|v| v.is_finite())
}
