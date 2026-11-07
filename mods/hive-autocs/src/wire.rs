//! Bedrock wire basics: the relay's batch framing, packet headers and the few field types the
//! packets this mod reads use.

/// The low bits of a packet header that hold the packet ID.
const ID_MASK: u32 = 0x3ff;

/// A cursor over packet bytes. Every read fails with `None` instead of panicking, so a packet
/// this mod misreads is simply left alone.
pub struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    pub fn rest(&self) -> &'a [u8] {
        &self.bytes[self.at..]
    }

    pub fn u8(&mut self) -> Option<u8> {
        let byte = *self.bytes.get(self.at)?;
        self.at += 1;
        Some(byte)
    }

    pub fn bool(&mut self) -> Option<bool> {
        self.u8().map(|byte| byte != 0)
    }

    pub fn u32_le(&mut self) -> Option<u32> {
        let bytes = self.take(4)?;
        Some(u32::from_le_bytes(bytes.try_into().ok()?))
    }

    pub fn var_u32(&mut self) -> Option<u32> {
        let mut value = 0u32;
        for shift in (0..35).step_by(7) {
            let byte = self.u8()?;
            value |= u32::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Some(value);
            }
        }
        None
    }

    pub fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(count)?;
        let bytes = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(bytes)
    }

    /// A `var_u32`-prefixed string; invalid UTF-8 is replaced rather than rejected.
    pub fn string(&mut self) -> Option<String> {
        let length = self.var_u32()? as usize;
        Some(String::from_utf8_lossy(self.take(length)?).into_owned())
    }
}

pub fn write_var_u32(out: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        out.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

pub fn write_string(out: &mut Vec<u8>, text: &str) {
    write_var_u32(out, text.len() as u32);
    out.extend_from_slice(text.as_bytes());
}

/// One packet of a batch: its ID and its whole encoding, header included.
pub struct Packet<'a> {
    pub id: u32,
    pub bytes: &'a [u8],
}

impl<'a> Packet<'a> {
    /// The fields after the header.
    pub fn body(&self) -> Reader<'a> {
        let mut reader = Reader::new(self.bytes);
        reader.var_u32();
        reader
    }
}

/// Splits a batch body (each packet as a `var_u32` length and its bytes) into packets; `None`
/// when the framing does not add up.
pub fn split_batch(batch: &[u8]) -> Option<Vec<Packet<'_>>> {
    let mut reader = Reader::new(batch);
    let mut packets = Vec::new();
    while !reader.rest().is_empty() {
        let length = reader.var_u32()? as usize;
        let bytes = reader.take(length)?;
        let header = Reader::new(bytes).var_u32()?;
        packets.push(Packet { id: header & ID_MASK, bytes });
    }
    Some(packets)
}

/// Appends one packet, given its ID and body, to a batch body.
pub fn push_packet(batch: &mut Vec<u8>, id: u32, body: &[u8]) {
    let mut packet = Vec::with_capacity(body.len() + 2);
    write_var_u32(&mut packet, id);
    packet.extend_from_slice(body);
    write_var_u32(batch, packet.len() as u32);
    batch.extend_from_slice(&packet);
}

/// Appends an already encoded packet to a batch body.
pub fn push_encoded(batch: &mut Vec<u8>, packet: &[u8]) {
    write_var_u32(batch, packet.len() as u32);
    batch.extend_from_slice(packet);
}

/// What happens to one packet of a batch: `None` keeps it; `Some(None)` drops it;
/// `Some(Some(bytes))` replaces it with another encoded packet.
pub type Change = Option<Option<Vec<u8>>>;

/// The batch with each packet changed as `change` says, or `None` when every packet is kept. The
/// new batch is built only from the first changed packet on, so a batch passing through (most
/// of them, chunk data included) is never copied. `size` is the original batch's length.
pub fn rewrite<'a>(
    packets: &[Packet<'a>],
    size: usize,
    mut change: impl FnMut(&Packet<'a>) -> Change,
) -> Option<Vec<u8>> {
    let mut out: Option<Vec<u8>> = None;
    for (index, packet) in packets.iter().enumerate() {
        match (change(packet), &mut out) {
            (None, None) => {}
            (None, Some(batch)) => push_encoded(batch, packet.bytes),
            (Some(substitute), _) => {
                let batch = out.get_or_insert_with(|| {
                    let mut batch = Vec::with_capacity(size);
                    for earlier in &packets[..index] {
                        push_encoded(&mut batch, earlier.bytes);
                    }
                    batch
                });
                if let Some(bytes) = substitute {
                    push_encoded(batch, &bytes);
                }
            }
        }
    }
    out
}
