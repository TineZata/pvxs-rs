// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
//! PvAccess wire primitives — header, PvaSize, string, status encoding/decoding.

pub const MAGIC: u8 = 0xCA;
pub const VERSION: u8 = 0x02;
pub const FLAG_LE: u8 = 0x00;
pub const FLAG_FROM_SERVER: u8 = 0x40;

// Application command codes (pvAccess spec §4.1)
pub const CMD_BEACON: u8 = 0x00;
pub const CMD_CONNECTION_VALIDATION: u8 = 0x01; // server → client
#[allow(dead_code)]
pub const CMD_ECHO: u8 = 0x02;
pub const CMD_SEARCH: u8 = 0x03;
pub const CMD_SEARCH_RESPONSE: u8 = 0x04;
pub const CMD_CREATE_CHANNEL: u8 = 0x07;
pub const CMD_DESTROY_CHANNEL: u8 = 0x08;
pub const CMD_CONNECTION_VALIDATED: u8 = 0x09; // client → server
pub const CMD_GET: u8 = 0x0A;
pub const CMD_PUT: u8 = 0x0B;
pub const CMD_MONITOR: u8 = 0x0D;
pub const CMD_DESTROY_REQUEST: u8 = 0x0F;
#[allow(dead_code)]
pub const CMD_GET_FIELD: u8 = 0x11;
#[allow(dead_code)]
pub const CMD_RPC: u8 = 0x14;
#[allow(dead_code)]
pub const CMD_ORIGIN_TAG: u8 = 0x16;

// Status type bytes
pub const STATUS_OK_NOMSG: u8 = 0xFF;
pub const STATUS_OK: u8 = 0x00;
// 0x01 = WARNING, 0x02 = ERROR, 0x03 = FATAL

// pvData field type codes
pub const TYPE_BOOL: u8 = 0x00;
pub const TYPE_INT8: u8 = 0x20;
pub const TYPE_INT16: u8 = 0x21;
pub const TYPE_INT32: u8 = 0x22;
pub const TYPE_INT64: u8 = 0x23;
pub const TYPE_UINT8: u8 = 0x24;
pub const TYPE_UINT16: u8 = 0x25;
pub const TYPE_UINT32: u8 = 0x26;
pub const TYPE_UINT64: u8 = 0x27;
pub const TYPE_FLOAT32: u8 = 0x42;
pub const TYPE_FLOAT64: u8 = 0x43;
pub const TYPE_STRING: u8 = 0x60;
pub const TYPE_STRUCT: u8 = 0x80;
pub const TYPE_UNION: u8 = 0x81;
pub const TYPE_ANY: u8 = 0x82;
pub const TYPE_NULL: u8 = 0xFF;
// Array = scalar_code | 0x08  (e.g. double[] = 0x4B)
// Structure array = 0x88, union array = 0x89, any array = 0x8A

// Type-cache markers (prefix wrapping a FieldDesc in server responses)
pub const TYPE_CACHE_DEFINE: u8 = 0xFD; // define type at cache slot N
pub const TYPE_CACHE_REF: u8 = 0xFE;    // reuse type from cache slot N

// ── Header ────────────────────────────────────────────────────────────────

/// Encode an 8-byte pvAccess frame header (always little-endian).
#[inline]
pub fn encode_header(from_server: bool, cmd: u8, payload_len: u32) -> [u8; 8] {
    let flags = if from_server { FLAG_FROM_SERVER | FLAG_LE } else { FLAG_LE };
    let len = payload_len.to_le_bytes();
    [MAGIC, VERSION, flags, cmd, len[0], len[1], len[2], len[3]]
}

/// Decode a pvAccess frame header.
/// Returns `(from_server, command, payload_len)` or `None` if magic/version mismatch.
#[inline]
pub fn decode_header(buf: &[u8; 8]) -> Option<(bool, u8, u32)> {
    if buf[0] != MAGIC || buf[1] != VERSION {
        return None;
    }
    let from_server = buf[2] & FLAG_FROM_SERVER != 0;
    let cmd = buf[3];
    let payload_len = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    Some((from_server, cmd, payload_len))
}

// ── PvaSize ───────────────────────────────────────────────────────────────

/// Encode a PvaSize (0–254 = 1 byte; larger = 0xFF prefix + i32 LE).
pub fn encode_size(n: usize, out: &mut Vec<u8>) {
    if n < 0xFF {
        out.push(n as u8);
    } else {
        out.push(0xFF);
        out.extend_from_slice(&(n as i32).to_le_bytes());
    }
}

/// Decode a PvaSize. Returns `None` on underflow or the null sentinel (0xFF null form).
pub fn decode_size(cur: &mut &[u8]) -> Option<usize> {
    let b = take_byte(cur)?;
    if b != 0xFF {
        return Some(b as usize);
    }
    // Extended: 0xFF + i32 LE.  Negative = null (absent).
    if cur.len() < 4 {
        return None;
    }
    let n = i32::from_le_bytes([cur[0], cur[1], cur[2], cur[3]]);
    *cur = &cur[4..];
    if n < 0 { None } else { Some(n as usize) }
}

// ── String ────────────────────────────────────────────────────────────────

/// Encode a pvAccess string: PvaSize(len) + UTF-8 bytes.
pub fn encode_string(s: &str, out: &mut Vec<u8>) {
    encode_size(s.len(), out);
    out.extend_from_slice(s.as_bytes());
}

/// Decode a pvAccess string. Returns `None` on parse failure.
pub fn decode_string(cur: &mut &[u8]) -> Option<String> {
    let len = decode_size(cur)?;
    if cur.len() < len {
        return None;
    }
    let s = std::str::from_utf8(&cur[..len]).ok()?.to_owned();
    *cur = &cur[len..];
    Some(s)
}

/// Skip a pvAccess string without allocating.
pub fn skip_string(cur: &mut &[u8]) -> Option<()> {
    let len = decode_size(cur)?;
    if cur.len() < len {
        return None;
    }
    *cur = &cur[len..];
    Some(())
}

// ── Status ────────────────────────────────────────────────────────────────

/// Decode a Status byte sequence. Returns `true` = OK, `false` = error/warning.
/// Skips the message and stack-trace strings when present.
pub fn decode_status(cur: &mut &[u8]) -> bool {
    match take_byte(cur) {
        Some(STATUS_OK_NOMSG) => true,
        Some(STATUS_OK) => {
            skip_string(cur);
            skip_string(cur);
            true
        }
        _ => {
            // WARNING / ERROR / FATAL — skip message + stack
            skip_string(cur);
            skip_string(cur);
            false
        }
    }
}

// ── Scalar readers (little-endian) ───────────────────────────────────────

pub fn take_byte(cur: &mut &[u8]) -> Option<u8> {
    let b = *cur.first()?;
    *cur = &cur[1..];
    Some(b)
}

pub fn read_u16_le(cur: &mut &[u8]) -> Option<u16> {
    if cur.len() < 2 { return None; }
    let v = u16::from_le_bytes([cur[0], cur[1]]);
    *cur = &cur[2..];
    Some(v)
}

pub fn read_u32_le(cur: &mut &[u8]) -> Option<u32> {
    if cur.len() < 4 { return None; }
    let v = u32::from_le_bytes([cur[0], cur[1], cur[2], cur[3]]);
    *cur = &cur[4..];
    Some(v)
}

pub fn read_u64_le(cur: &mut &[u8]) -> Option<u64> {
    if cur.len() < 8 { return None; }
    let v = u64::from_le_bytes([cur[0], cur[1], cur[2], cur[3], cur[4], cur[5], cur[6], cur[7]]);
    *cur = &cur[8..];
    Some(v)
}

pub fn read_i32_le(cur: &mut &[u8]) -> Option<i32> {
    read_u32_le(cur).map(|v| v as i32)
}

pub fn read_i64_le(cur: &mut &[u8]) -> Option<i64> {
    read_u64_le(cur).map(|v| v as i64)
}

pub fn read_f32_le(cur: &mut &[u8]) -> Option<f32> {
    read_u32_le(cur).map(f32::from_bits)
}

pub fn read_f64_le(cur: &mut &[u8]) -> Option<f64> {
    read_u64_le(cur).map(f64::from_bits)
}
