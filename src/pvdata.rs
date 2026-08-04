// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
//! PvData type-system decoder: FieldDesc, BitSet, and value payload → `Value`.

use crate::proto::*;
use crate::Value;

// ── FieldDesc ─────────────────────────────────────────────────────────────

/// Mirrors the pvData FieldDesc type tree used in GET INIT introspection.
#[derive(Debug, Clone)]
pub enum FieldDesc {
    /// A scalar field with its pvData type code (TYPE_FLOAT64, TYPE_INT32, …).
    Scalar(u8),
    /// A 1-D array field; stores the *element* scalar type code.
    ScalarArray(u8),
    Structure {
        #[allow(dead_code)]
        type_id: String,
        fields: Vec<(String, FieldDesc)>,
    },
    Union {
        #[allow(dead_code)]
        type_id: String,
        fields: Vec<(String, FieldDesc)>,
    },
    Any,
    /// Placeholder for types we cannot decode (type-cache 0xFE refs, unknowns).
    Opaque,
}

/// Decode a FieldDesc that may be prefixed by a type-cache marker (0xFD/0xFE).
pub fn decode_field_desc_cached(cur: &mut &[u8]) -> Option<FieldDesc> {
    let tag = *cur.first()?;
    match tag {
        TYPE_CACHE_DEFINE => {
            *cur = &cur[1..];
            decode_size(cur)?; // slot id — we don't maintain a cache
            decode_field_desc(cur)
        }
        TYPE_CACHE_REF => {
            *cur = &cur[1..];
            decode_size(cur)?; // slot id
            // Phase 1: type cache refs unsupported; caller gets Opaque
            Some(FieldDesc::Opaque)
        }
        _ => decode_field_desc(cur),
    }
}

/// Decode an inline FieldDesc (no type-cache prefix).
pub fn decode_field_desc(cur: &mut &[u8]) -> Option<FieldDesc> {
    let type_code = take_byte(cur)?;
    match type_code {
        TYPE_NULL => None,

        TYPE_STRUCT => {
            let type_id = decode_string(cur)?;
            let n = decode_size(cur)?;
            let mut fields = Vec::with_capacity(n);
            for _ in 0..n {
                let name = decode_string(cur)?;
                let desc = decode_field_desc(cur)?;
                fields.push((name, desc));
            }
            Some(FieldDesc::Structure { type_id, fields })
        }

        TYPE_UNION => {
            let type_id = decode_string(cur)?;
            let n = decode_size(cur)?;
            let mut fields = Vec::with_capacity(n);
            for _ in 0..n {
                let name = decode_string(cur)?;
                let desc = decode_field_desc(cur)?;
                fields.push((name, desc));
            }
            Some(FieldDesc::Union { type_id, fields })
        }

        TYPE_ANY => Some(FieldDesc::Any),

        // Structure array (0x88): element struct type_id + field descs
        0x88 => {
            let _type_id = decode_string(cur)?;
            let n = decode_size(cur)?;
            for _ in 0..n {
                skip_string(cur)?;
                decode_field_desc(cur)?;
            }
            Some(FieldDesc::ScalarArray(TYPE_STRUCT))
        }

        // Union array (0x89) / any array (0x8A) — skip element descriptor
        0x89 => {
            let _type_id = decode_string(cur)?;
            let n = decode_size(cur)?;
            for _ in 0..n {
                skip_string(cur)?;
                decode_field_desc(cur)?;
            }
            Some(FieldDesc::ScalarArray(TYPE_UNION))
        }
        0x8A => Some(FieldDesc::ScalarArray(TYPE_ANY)),

        // Bool array
        0x08 => Some(FieldDesc::ScalarArray(TYPE_BOOL)),

        // Scalar arrays: type_code has bit 0x08 set (and is not 0x80/0x81/0x82/0x88–0x8A)
        code if code & 0x08 != 0 => Some(FieldDesc::ScalarArray(code & !0x08)),

        // Plain scalar
        code => Some(FieldDesc::Scalar(code)),
    }
}

// ── BitSet ────────────────────────────────────────────────────────────────

/// Read a BitSet from the cursor: PvaSize(N) + N bytes (LSB-first).
pub fn read_bitset<'a>(cur: &mut &'a [u8]) -> Option<&'a [u8]> {
    let n = decode_size(cur)?;
    if cur.len() < n { return None; }
    let bits = &cur[..n];
    *cur = &cur[n..];
    Some(bits)
}

#[inline]
fn bit_set(bits: &[u8], i: usize) -> bool {
    let byte_idx = i / 8;
    let bit_idx = i % 8;
    bits.get(byte_idx).is_some_and(|b| (b >> bit_idx) & 1 == 1)
}

// ── Value decode ──────────────────────────────────────────────────────────

/// Walk the FieldDesc DFS tree, reading present fields from `cur` into `out`.
///
/// `bit` is the DFS position counter (call with `&mut 0` at the root).
/// `path` is the dot-separated field path prefix (call with `""` at the root).
pub fn decode_into_value(
    cur: &mut &[u8],
    desc: &FieldDesc,
    bits: &[u8],
    bit: &mut usize,
    path: &str,
    out: &mut Value,
) -> Option<()> {
    match desc {
        FieldDesc::Structure { fields, .. } => {
            let my_bit = *bit;
            *bit += 1;
            if !bit_set(bits, my_bit) {
                // Whole structure absent — advance bit counters but read nothing
                advance_bits(desc, bit);
                return Some(());
            }
            for (name, child) in fields {
                let child_path = if path.is_empty() {
                    name.clone()
                } else {
                    format!("{}.{}", path, name)
                };
                decode_into_value(cur, child, bits, bit, &child_path, out)?;
            }
            Some(())
        }

        FieldDesc::Scalar(type_code) => {
            let my_bit = *bit;
            *bit += 1;
            if bit_set(bits, my_bit) {
                read_scalar_into(cur, *type_code, path, out)?;
            } else {
                // Field absent — no bytes in payload
            }
            Some(())
        }

        FieldDesc::ScalarArray(elem_type) => {
            let my_bit = *bit;
            *bit += 1;
            if bit_set(bits, my_bit) {
                read_array_into(cur, *elem_type, path, out)?;
            }
            Some(())
        }

        FieldDesc::Union { fields, .. } => {
            let my_bit = *bit;
            *bit += 1;
            if !bit_set(bits, my_bit) {
                *bit += fields.len(); // skip selected-field bits
                return Some(());
            }
            // Union: selector byte (which field is active) + selected FieldDesc value
            let selector = take_byte(cur)?;
            for (i, (name, child)) in fields.iter().enumerate() {
                let child_path = if path.is_empty() { name.clone() } else { format!("{}.{}", path, name) };
                if i == selector as usize {
                    decode_into_value(cur, child, bits, bit, &child_path, out)?;
                } else {
                    *bit += 1;
                }
            }
            Some(())
        }

        FieldDesc::Any | FieldDesc::Opaque => {
            *bit += 1;
            Some(()) // Skip; not decodable in phase 1
        }
    }
}

/// Advance the bit counter past a FieldDesc sub-tree without reading any bytes.
fn advance_bits(desc: &FieldDesc, bit: &mut usize) {
    match desc {
        FieldDesc::Structure { fields, .. } => {
            for (_, child) in fields {
                advance_bits(child, bit);
            }
        }
        FieldDesc::Union { fields, .. } => {
            *bit += fields.len();
        }
        _ => {} // Scalar, ScalarArray, Any, Opaque already advanced by caller
    }
}

fn read_scalar_into(cur: &mut &[u8], type_code: u8, path: &str, out: &mut Value) -> Option<()> {
    match type_code {
        TYPE_BOOL => { let v = take_byte(cur)? != 0; out.set_field_bool(path, v); }
        TYPE_INT8 | TYPE_UINT8 => { let v = take_byte(cur)?; out.set_field_int32(path, v as i32); }
        TYPE_INT16 | TYPE_UINT16 => { let v = read_u16_le(cur)?; out.set_field_int32(path, v as i32); }
        TYPE_INT32 | TYPE_UINT32 => { let v = read_i32_le(cur)?; out.set_field_int32(path, v); }
        TYPE_INT64 | TYPE_UINT64 => { let v = read_i64_le(cur)?; out.set_field_int64(path, v); }
        TYPE_FLOAT32 => { let v = read_f32_le(cur)?; out.set_field_double(path, v as f64); }
        TYPE_FLOAT64 => { let v = read_f64_le(cur)?; out.set_field_double(path, v); }
        TYPE_STRING => { let v = decode_string(cur)?; out.set_field_string(path, v); }
        _ => {} // Unknown scalar type — skip gracefully (no bytes consumed)
    }
    Some(())
}

fn read_array_into(cur: &mut &[u8], elem_type: u8, path: &str, out: &mut Value) -> Option<()> {
    let count = read_u32_le(cur)? as usize;
    match elem_type {
        TYPE_FLOAT64 => {
            let mut arr = Vec::with_capacity(count);
            for _ in 0..count { arr.push(read_f64_le(cur)?); }
            out.set_field_double_array(path, arr);
        }
        TYPE_INT32 | TYPE_UINT32 => {
            let mut arr = Vec::with_capacity(count);
            for _ in 0..count { arr.push(read_i32_le(cur)?); }
            out.set_field_int32_array(path, arr);
        }
        TYPE_STRING => {
            let mut arr = Vec::with_capacity(count);
            for _ in 0..count { arr.push(decode_string(cur)?); }
            out.set_field_string_array(path, arr);
        }
        _ => {
            // Unknown element type — nothing decoded for this array
        }
    }
    Some(())
}

// ── pvRequest builder ─────────────────────────────────────────────────────

/// Build the 9-byte pvRequest for `field()` — request all fields.
///
/// Wire format: structure { structure field { } }
/// No value bytes follow (empty sub-structures carry no value payload).
pub fn build_pv_request_all() -> &'static [u8] {
    &[
        0x80, // structure tag
        0x00, // empty type_id
        0x01, // 1 outer field
        // Field name "field": PvaSize(5) + bytes
        0x05, b'f', b'i', b'e', b'l', b'd',
        0x80, // sub-structure tag
        0x00, // empty type_id
        0x00, // 0 subfields = select all
    ]
}
