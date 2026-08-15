// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
//! Pure-Rust pvAccess server.
//!
//! All state is held in a worker thread via crossbeam channels.
//! The pvAccess TCP/UDP transport is a TODO — see TODO.md.

use crossbeam_channel as channel;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::task::JoinSet;

use crate::{
    AlarmMetadata, AlarmSeverity, AlarmStatus, ControlMetadata, DisplayMetadata, PvxsError, Result,
};
pub(crate) mod manager;
pub(crate) mod ntenum;
pub(crate) mod ntscalar;

pub use self::manager::{run_worker, ManagerCommand};
pub use self::ntenum::NTEnumMetadataBuilder;
pub use self::ntscalar::NTScalarMetadataBuilder;

use crate::proto::{
    decode_header, decode_size, decode_string, encode_header, encode_size, encode_string,
    read_i32_le, read_u32_le, take_byte, CMD_BEACON, CMD_CONNECTION_VALIDATED,
    CMD_CONNECTION_VALIDATION, CMD_CREATE_CHANNEL, CMD_DESTROY_CHANNEL, CMD_DESTROY_REQUEST,
    CMD_GET, CMD_MONITOR, CMD_PUT, CMD_SEARCH, CMD_SEARCH_RESPONSE, STATUS_OK_NOMSG,
    TYPE_CACHE_DEFINE, TYPE_FLOAT64, TYPE_INT16, TYPE_INT32, TYPE_STRING,
};
use crate::FieldType;

fn parse_search_request(payload: &[u8]) -> Option<(u32, u32)> {
    let mut cur = payload;
    if cur.len() < 4 {
        return None;
    }
    let seq_id = u32::from_le_bytes([cur[0], cur[1], cur[2], cur[3]]);
    // flags(1) + reserved(3) + responseAddress(16) + responsePort(2)
    if cur.len() < 4 + 1 + 3 + 16 + 2 {
        return None;
    }
    cur = &cur[4 + 1 + 3 + 16 + 2..];

    let proto_count = decode_size(&mut cur)?;
    for _ in 0..proto_count {
        let _ = decode_string(&mut cur)?;
    }

    let channel_count = decode_size(&mut cur)?;
    if channel_count == 0 {
        return None;
    }
    if cur.len() < 4 {
        return None;
    }
    let cid = u32::from_le_bytes([cur[0], cur[1], cur[2], cur[3]]);
    Some((seq_id, cid))
}

fn search_response_addr(ip: Ipv4Addr) -> [u8; 16] {
    let mut out = [0u8; 16];
    out[10] = 0xFF;
    out[11] = 0xFF;
    out[12..16].copy_from_slice(&ip.octets());
    out
}

fn build_search_response(seq_id: u32, cid: u32, iface_ip: Ipv4Addr, tcp_port: u16) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&[0u8; 12]); // server GUID (placeholder)
    payload.extend_from_slice(&seq_id.to_le_bytes());
    payload.extend_from_slice(&search_response_addr(iface_ip));
    payload.extend_from_slice(&tcp_port.to_le_bytes());
    encode_size(1, &mut payload); // found channel count
    payload.extend_from_slice(&cid.to_le_bytes());

    let mut out = encode_header(true, CMD_SEARCH_RESPONSE, payload.len() as u32).to_vec();
    out.extend_from_slice(&payload);
    out
}

fn build_beacon(seq_id: u32, tcp_port: u16) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&[0u8; 12]); // server GUID placeholder
    payload.extend_from_slice(&seq_id.to_le_bytes());
    payload.push(0); // changeCount
    payload.extend_from_slice(&search_response_addr(Ipv4Addr::UNSPECIFIED));
    payload.extend_from_slice(&tcp_port.to_le_bytes());
    let mut out = encode_header(true, CMD_BEACON, payload.len() as u32).to_vec();
    out.extend_from_slice(&payload);
    out
}

fn status_ok() -> Vec<u8> {
    vec![STATUS_OK_NOMSG]
}

fn status_error(msg: &str) -> Vec<u8> {
    let mut out = vec![0x02];
    encode_string(msg, &mut out);
    encode_string("", &mut out);
    out
}

fn wrap_type_desc(desc: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(TYPE_CACHE_DEFINE);
    encode_size(0, &mut out);
    out.extend_from_slice(&desc);
    out
}

fn desc_value_scalar(type_code: u8) -> Vec<u8> {
    let mut d = Vec::new();
    d.push(0x80); // structure
    d.push(0x00); // empty type_id
    d.push(0x01); // one field
    encode_string("value", &mut d);
    d.push(type_code);
    d
}

fn desc_value_string_array() -> Vec<u8> {
    let mut d = Vec::new();
    d.push(0x80); // structure
    d.push(0x00); // empty type_id
    d.push(0x01); // one field
    encode_string("value", &mut d);
    d.push(TYPE_STRING | 0x08);
    d
}

fn desc_value_enum() -> Vec<u8> {
    let mut d = Vec::new();
    d.push(0x80); // structure
    d.push(0x00); // empty type_id
    d.push(0x03); // three fields

    encode_string("value", &mut d);
    d.push(TYPE_INT16);

    encode_string("value.index", &mut d);
    d.push(TYPE_INT16);

    encode_string("value.choices", &mut d);
    d.push(TYPE_STRING | 0x08);

    d
}

fn bitset_value_present() -> Vec<u8> {
    vec![1, 0b0000_0011]
}

fn bitset_enum_present() -> Vec<u8> {
    // root + value + value.index + value.choices
    vec![1, 0b0000_1111]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChannelType {
    Double,
    Int32,
    String,
    Enum,
    DoubleArray,
    Int32Array,
    StringArray,
}

#[derive(Clone, Debug)]
struct ChannelInfo {
    sid: u32,
    cid: u32,
    pv_name: String,
    ty: ChannelType,
}

fn detect_channel_type(tx: &channel::Sender<ManagerCommand>, pv_name: &str) -> Option<ChannelType> {
    let (r_tx, r_rx) = channel::bounded(1);
    let _ = tx.send(ManagerCommand::FetchDouble {
        name: pv_name.to_string(),
        reply: r_tx,
    });
    if let Ok(Ok(_)) = r_rx.recv() {
        return Some(ChannelType::Double);
    }

    let (r_tx, r_rx) = channel::bounded(1);
    let _ = tx.send(ManagerCommand::FetchInt32 {
        name: pv_name.to_string(),
        reply: r_tx,
    });
    if let Ok(Ok(_)) = r_rx.recv() {
        return Some(ChannelType::Int32);
    }

    let (r_tx, r_rx) = channel::bounded(1);
    let _ = tx.send(ManagerCommand::FetchString {
        name: pv_name.to_string(),
        reply: r_tx,
    });
    if let Ok(Ok(_)) = r_rx.recv() {
        return Some(ChannelType::String);
    }

    let (r_tx, r_rx) = channel::bounded(1);
    let _ = tx.send(ManagerCommand::FetchEnum {
        name: pv_name.to_string(),
        reply: r_tx,
    });
    if let Ok(Ok(_)) = r_rx.recv() {
        return Some(ChannelType::Enum);
    }

    let (r_tx, r_rx) = channel::bounded(1);
    let _ = tx.send(ManagerCommand::FetchDoubleArray {
        name: pv_name.to_string(),
        reply: r_tx,
    });
    if let Ok(Ok(_)) = r_rx.recv() {
        return Some(ChannelType::DoubleArray);
    }

    let (r_tx, r_rx) = channel::bounded(1);
    let _ = tx.send(ManagerCommand::FetchInt32Array {
        name: pv_name.to_string(),
        reply: r_tx,
    });
    if let Ok(Ok(_)) = r_rx.recv() {
        return Some(ChannelType::Int32Array);
    }

    let (r_tx, r_rx) = channel::bounded(1);
    let _ = tx.send(ManagerCommand::FetchStringArray {
        name: pv_name.to_string(),
        reply: r_tx,
    });
    if let Ok(Ok(_)) = r_rx.recv() {
        return Some(ChannelType::StringArray);
    }

    None
}

fn value_desc_for(ty: ChannelType) -> Vec<u8> {
    match ty {
        ChannelType::Double => wrap_type_desc(desc_value_scalar(TYPE_FLOAT64)),
        ChannelType::Int32 => wrap_type_desc(desc_value_scalar(TYPE_INT32)),
        ChannelType::String => wrap_type_desc(desc_value_scalar(TYPE_STRING)),
        ChannelType::Enum => wrap_type_desc(desc_value_enum()),
        ChannelType::DoubleArray => wrap_type_desc(desc_value_scalar(TYPE_FLOAT64 | 0x08)),
        ChannelType::Int32Array => wrap_type_desc(desc_value_scalar(TYPE_INT32 | 0x08)),
        ChannelType::StringArray => wrap_type_desc(desc_value_string_array()),
    }
}

fn encode_current_value_payload(
    tx: &channel::Sender<ManagerCommand>,
    ch: &ChannelInfo,
) -> Option<Vec<u8>> {
    let mut out = if ch.ty == ChannelType::Enum {
        bitset_enum_present()
    } else {
        bitset_value_present()
    };
    match ch.ty {
        ChannelType::Double => {
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::FetchDouble {
                name: ch.pv_name.clone(),
                reply: r_tx,
            });
            let v = r_rx.recv().ok()?.ok()?;
            out.extend_from_slice(&v.value.to_bits().to_le_bytes());
        }
        ChannelType::Int32 => {
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::FetchInt32 {
                name: ch.pv_name.clone(),
                reply: r_tx,
            });
            let v = r_rx.recv().ok()?.ok()?;
            out.extend_from_slice(&v.value.to_le_bytes());
        }
        ChannelType::String => {
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::FetchString {
                name: ch.pv_name.clone(),
                reply: r_tx,
            });
            let v = r_rx.recv().ok()?.ok()?;
            encode_string(&v.value, &mut out);
        }
        ChannelType::Enum => {
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::FetchEnum {
                name: ch.pv_name.clone(),
                reply: r_tx,
            });
            let v = r_rx.recv().ok()?.ok()?;
            out.extend_from_slice(&v.value.to_le_bytes());
            out.extend_from_slice(&v.value.to_le_bytes());
            out.extend_from_slice(&(v.value_choices.len() as u32).to_le_bytes());
            for choice in v.value_choices {
                encode_string(&choice, &mut out);
            }
        }
        ChannelType::DoubleArray => {
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::FetchDoubleArray {
                name: ch.pv_name.clone(),
                reply: r_tx,
            });
            let v = r_rx.recv().ok()?.ok()?;
            out.extend_from_slice(&(v.value.len() as u32).to_le_bytes());
            for x in &v.value {
                out.extend_from_slice(&x.to_bits().to_le_bytes());
            }
        }
        ChannelType::Int32Array => {
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::FetchInt32Array {
                name: ch.pv_name.clone(),
                reply: r_tx,
            });
            let v = r_rx.recv().ok()?.ok()?;
            out.extend_from_slice(&(v.value.len() as u32).to_le_bytes());
            for x in &v.value {
                out.extend_from_slice(&x.to_le_bytes());
            }
        }
        ChannelType::StringArray => {
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::FetchStringArray {
                name: ch.pv_name.clone(),
                reply: r_tx,
            });
            let v = r_rx.recv().ok()?.ok()?;
            out.extend_from_slice(&(v.value.len() as u32).to_le_bytes());
            for x in &v.value {
                encode_string(x, &mut out);
            }
        }
    }
    Some(out)
}

fn decode_put_and_apply(
    tx: &channel::Sender<ManagerCommand>,
    ch: &ChannelInfo,
    cur: &mut &[u8],
) -> std::result::Result<(), String> {
    let bitset_len = decode_size(cur).ok_or_else(|| "PUT missing BitSet size".to_string())?;
    if cur.len() < bitset_len {
        return Err("PUT truncated BitSet".to_string());
    }
    *cur = &cur[bitset_len..];

    match ch.ty {
        ChannelType::Double => {
            if cur.len() < 8 {
                return Err("PUT double payload too short".to_string());
            }
            let bits = u64::from_le_bytes([
                cur[0], cur[1], cur[2], cur[3], cur[4], cur[5], cur[6], cur[7],
            ]);
            let value = f64::from_bits(bits);
            *cur = &cur[8..];
            if !cur.is_empty() {
                return Err("PUT payload type mismatch".to_string());
            }
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::PostDouble {
                name: ch.pv_name.clone(),
                value,
                reply: r_tx,
            });
            r_rx.recv()
                .map_err(|_| "server worker stopped".to_string())?
                .map_err(|e| e.to_string())
        }
        ChannelType::Int32 => {
            let value =
                read_i32_le(cur).ok_or_else(|| "PUT int32 payload too short".to_string())?;
            if !cur.is_empty() {
                return Err("PUT payload type mismatch".to_string());
            }
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::PostInt32 {
                name: ch.pv_name.clone(),
                value,
                reply: r_tx,
            });
            r_rx.recv()
                .map_err(|_| "server worker stopped".to_string())?
                .map_err(|e| e.to_string())
        }
        ChannelType::String => {
            let value = decode_string(cur).ok_or_else(|| "PUT string decode failed".to_string())?;
            if !cur.is_empty() {
                return Err("PUT payload type mismatch".to_string());
            }
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::PostString {
                name: ch.pv_name.clone(),
                value,
                reply: r_tx,
            });
            r_rx.recv()
                .map_err(|_| "server worker stopped".to_string())?
                .map_err(|e| e.to_string())
        }
        ChannelType::Enum => {
            if cur.len() < 2 {
                return Err("PUT enum payload too short".to_string());
            }
            let value = i16::from_le_bytes([cur[0], cur[1]]);
            *cur = &cur[2..];
            if !cur.is_empty() {
                return Err("PUT payload type mismatch".to_string());
            }
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::PostEnum {
                name: ch.pv_name.clone(),
                value,
                reply: r_tx,
            });
            r_rx.recv()
                .map_err(|_| "server worker stopped".to_string())?
                .map_err(|e| e.to_string())
        }
        ChannelType::DoubleArray => {
            let n =
                read_u32_le(cur).ok_or_else(|| "PUT double[] missing count".to_string())? as usize;
            let mut values = Vec::with_capacity(n);
            for _ in 0..n {
                if cur.len() < 8 {
                    return Err("PUT double[] truncated".to_string());
                }
                let bits = u64::from_le_bytes([
                    cur[0], cur[1], cur[2], cur[3], cur[4], cur[5], cur[6], cur[7],
                ]);
                *cur = &cur[8..];
                values.push(f64::from_bits(bits));
            }
            if !cur.is_empty() {
                return Err("PUT payload type mismatch".to_string());
            }
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::PostDoubleArray {
                name: ch.pv_name.clone(),
                value: values,
                reply: r_tx,
            });
            r_rx.recv()
                .map_err(|_| "server worker stopped".to_string())?
                .map_err(|e| e.to_string())
        }
        ChannelType::Int32Array => {
            let n =
                read_u32_le(cur).ok_or_else(|| "PUT int32[] missing count".to_string())? as usize;
            let mut values = Vec::with_capacity(n);
            for _ in 0..n {
                let v = read_i32_le(cur).ok_or_else(|| "PUT int32[] truncated".to_string())?;
                values.push(v);
            }
            if !cur.is_empty() {
                return Err("PUT payload type mismatch".to_string());
            }
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::PostInt32Array {
                name: ch.pv_name.clone(),
                value: values,
                reply: r_tx,
            });
            r_rx.recv()
                .map_err(|_| "server worker stopped".to_string())?
                .map_err(|e| e.to_string())
        }
        ChannelType::StringArray => {
            let n =
                read_u32_le(cur).ok_or_else(|| "PUT string[] missing count".to_string())? as usize;
            let mut values = Vec::with_capacity(n);
            for _ in 0..n {
                let v =
                    decode_string(cur).ok_or_else(|| "PUT string[] decode failed".to_string())?;
                values.push(v);
            }
            if !cur.is_empty() {
                return Err("PUT payload type mismatch".to_string());
            }
            let (r_tx, r_rx) = channel::bounded(1);
            let _ = tx.send(ManagerCommand::PostStringArray {
                name: ch.pv_name.clone(),
                value: values,
                reply: r_tx,
            });
            r_rx.recv()
                .map_err(|_| "server worker stopped".to_string())?
                .map_err(|e| e.to_string())
        }
    }
}

async fn read_frame(stream: &mut TcpStream) -> std::io::Result<(bool, u8, Vec<u8>)> {
    let mut hdr = [0u8; 8];
    stream.read_exact(&mut hdr).await?;
    let (from_server, cmd, payload_len) = decode_header(&hdr)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "bad pvA header"))?;
    let mut payload = vec![0u8; payload_len as usize];
    if payload_len > 0 {
        stream.read_exact(&mut payload).await?;
    }
    Ok((from_server, cmd, payload))
}

fn make_frame(from_server: bool, cmd: u8, payload: Vec<u8>) -> Vec<u8> {
    let mut out = encode_header(from_server, cmd, payload.len() as u32).to_vec();
    out.extend_from_slice(&payload);
    out
}

async fn handle_client(
    mut stream: TcpStream,
    tx: channel::Sender<ManagerCommand>,
) -> std::io::Result<()> {
    let mut validation = Vec::new();
    validation.extend_from_slice(&(16u32 * 1024 * 1024).to_le_bytes());
    validation.extend_from_slice(&0x10u16.to_le_bytes());
    encode_size(1, &mut validation);
    encode_string("anonymous", &mut validation);
    stream
        .write_all(&make_frame(true, CMD_CONNECTION_VALIDATION, validation))
        .await?;

    loop {
        let (_, cmd, _) = read_frame(&mut stream).await?;
        if cmd == CMD_CONNECTION_VALIDATED {
            break;
        }
    }

    use std::collections::HashMap;
    let mut channels: HashMap<u32, ChannelInfo> = HashMap::new();
    let mut sid_next: u32 = 1;
    let (push_tx, mut push_rx) = tokio::sync::mpsc::unbounded_channel::<manager::MonitorPush>();
    // active_monitors: ioid → (sid, pv_name, sub_id)
    let mut active_monitors: HashMap<u32, (u32, String, u64)> = HashMap::new();

    loop {
        tokio::select! {
            frame = read_frame(&mut stream) => {
                let (_, cmd, payload) = match frame {
                    Ok(f) => f,
                    Err(_) => break,
                };

                match cmd {
            CMD_CREATE_CHANNEL => {
                let mut cur = payload.as_slice();
                if cur.len() < 2 {
                    continue;
                }
                cur = &cur[2..]; // count
                let cid = match read_u32_le(&mut cur) {
                    Some(v) => v,
                    None => continue,
                };
                let pv_name = match decode_string(&mut cur) {
                    Some(s) => s,
                    None => continue,
                };

                let mut resp = Vec::new();
                resp.extend_from_slice(&cid.to_le_bytes());
                let sid = sid_next;
                sid_next = sid_next.wrapping_add(1);
                resp.extend_from_slice(&sid.to_le_bytes());

                if let Some(ty) = detect_channel_type(&tx, &pv_name) {
                    channels.insert(
                        sid,
                        ChannelInfo {
                            sid,
                            cid,
                            pv_name,
                            ty,
                        },
                    );
                    resp.extend_from_slice(&status_ok());
                } else {
                    resp.extend_from_slice(&status_error("PV not found"));
                }
                stream
                    .write_all(&make_frame(true, CMD_CREATE_CHANNEL, resp))
                    .await?;
            }

            CMD_GET => {
                let mut cur = payload.as_slice();
                let sid = match read_u32_le(&mut cur) {
                    Some(v) => v,
                    None => continue,
                };
                let ioid = match read_u32_le(&mut cur) {
                    Some(v) => v,
                    None => continue,
                };
                let subcmd = take_byte(&mut cur).unwrap_or(0);

                let mut resp = Vec::new();
                resp.extend_from_slice(&ioid.to_le_bytes());
                resp.push(subcmd);

                if let Some(ch) = channels.get(&sid) {
                    if subcmd & 0x08 != 0 {
                        resp.extend_from_slice(&status_ok());
                        resp.extend_from_slice(&value_desc_for(ch.ty));
                    } else {
                        resp.extend_from_slice(&status_ok());
                        if let Some(vbytes) = encode_current_value_payload(&tx, ch) {
                            resp.extend_from_slice(&vbytes);
                        } else {
                            resp.clear();
                            resp.extend_from_slice(&ioid.to_le_bytes());
                            resp.push(subcmd);
                            resp.extend_from_slice(&status_error("GET fetch failed"));
                        }
                    }
                } else {
                    resp.extend_from_slice(&status_error("Unknown SID"));
                }

                stream.write_all(&make_frame(true, CMD_GET, resp)).await?;
            }

            CMD_PUT => {
                let mut cur = payload.as_slice();
                let sid = match read_u32_le(&mut cur) {
                    Some(v) => v,
                    None => continue,
                };
                let ioid = match read_u32_le(&mut cur) {
                    Some(v) => v,
                    None => continue,
                };
                let subcmd = take_byte(&mut cur).unwrap_or(0);

                let mut resp = Vec::new();
                resp.extend_from_slice(&ioid.to_le_bytes());
                resp.push(subcmd);

                if let Some(ch) = channels.get(&sid) {
                    if subcmd & 0x08 != 0 {
                        resp.extend_from_slice(&status_ok());
                        resp.extend_from_slice(&value_desc_for(ch.ty));
                    } else {
                        match decode_put_and_apply(&tx, ch, &mut cur) {
                            Ok(()) => resp.extend_from_slice(&status_ok()),
                            Err(msg) => resp.extend_from_slice(&status_error(&msg)),
                        }
                    }
                } else {
                    resp.extend_from_slice(&status_error("Unknown SID"));
                }

                stream.write_all(&make_frame(true, CMD_PUT, resp)).await?;
            }

            CMD_MONITOR => {
                let mut cur = payload.as_slice();
                let sid = match read_u32_le(&mut cur) {
                    Some(v) => v,
                    None => continue,
                };
                let ioid = match read_u32_le(&mut cur) {
                    Some(v) => v,
                    None => continue,
                };
                let subcmd = take_byte(&mut cur).unwrap_or(0);

                if let Some(ch) = channels.get(&sid) {
                    if subcmd & 0x08 != 0 {
                        // INIT: send type descriptor.
                        let mut resp = Vec::new();
                        resp.extend_from_slice(&ioid.to_le_bytes());
                        resp.push(subcmd);
                        resp.extend_from_slice(&status_ok());
                        resp.extend_from_slice(&value_desc_for(ch.ty));
                        stream
                            .write_all(&make_frame(true, CMD_MONITOR, resp))
                            .await?;
                    } else if subcmd & 0x04 != 0 {
                        // START: send initial value and subscribe for future pushes.
                        let mut resp = Vec::new();
                        resp.extend_from_slice(&ioid.to_le_bytes());
                        resp.push(0x00);
                        resp.extend_from_slice(&status_ok());
                        if let Some(vbytes) = encode_current_value_payload(&tx, ch) {
                            resp.extend_from_slice(&vbytes);
                        }
                        stream
                            .write_all(&make_frame(true, CMD_MONITOR, resp))
                            .await?;
                        let (reply_tx, reply_rx) = channel::bounded(1);
                        let _ = tx.send(ManagerCommand::SubscribeMonitor {
                            pv_name: ch.pv_name.clone(),
                            ioid,
                            tx: push_tx.clone(),
                            reply: reply_tx,
                        });
                        if let Ok(sub_id) = reply_rx.recv() {
                            active_monitors.insert(ioid, (sid, ch.pv_name.clone(), sub_id));
                        }
                    } else if subcmd & 0x40 != 0 {
                        // STOP: cancel subscription.
                        if let Some((_, pv_name, sub_id)) = active_monitors.remove(&ioid) {
                            let _ = tx.send(ManagerCommand::UnsubscribeMonitor { pv_name, sub_id });
                        }
                    }
                }
            }

            CMD_DESTROY_REQUEST => {
                let mut cur = payload.as_slice();
                let _ = read_u32_le(&mut cur); // sid (unused here)
                if let Some(ioid) = read_u32_le(&mut cur) {
                    if let Some((_, pv_name, sub_id)) = active_monitors.remove(&ioid) {
                        let _ = tx.send(ManagerCommand::UnsubscribeMonitor { pv_name, sub_id });
                    }
                }
            }
            CMD_DESTROY_CHANNEL => {
                let mut cur = payload.as_slice();
                let cid = match read_u32_le(&mut cur) {
                    Some(v) => v,
                    None => continue,
                };
                let sid = match read_u32_le(&mut cur) {
                    Some(v) => v,
                    None => continue,
                };
                if let Some(ch) = channels.get(&sid) {
                    if ch.cid == cid && ch.sid == sid {
                        let to_unsub: Vec<_> = active_monitors
                            .iter()
                            .filter(|(_, (s, _, _))| *s == sid)
                            .map(|(ioid, (_, pv, sub))| (*ioid, pv.clone(), *sub))
                            .collect();
                        for (ioid, pv_name, sub_id) in to_unsub {
                            active_monitors.remove(&ioid);
                            let _ = tx.send(ManagerCommand::UnsubscribeMonitor {
                                pv_name,
                                sub_id,
                            });
                        }
                        channels.remove(&sid);
                    }
                }
            }
            _ => {}
        }
            }
            push = push_rx.recv() => {
                if let Some(manager::MonitorPush { ioid, payload: push_payload }) = push {
                    let mut resp = Vec::new();
                    resp.extend_from_slice(&ioid.to_le_bytes());
                    resp.push(0x00); // subcmd = data
                    resp.extend_from_slice(&status_ok());
                    resp.extend_from_slice(&push_payload);
                    if stream
                        .write_all(&make_frame(true, CMD_MONITOR, resp))
                        .await
                        .is_err()
                    {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Fetched value types (mirror pvxs-sys exactly)
// ============================================================================

/// Snapshot of a fetched double PV value plus its metadata.
#[derive(Debug, Clone)]
pub struct FetchedDouble {
    /// Current numeric value.
    pub value: f64,
    /// Alarm severity for the current sample.
    pub alarm_severity: AlarmSeverity,
    /// Alarm status for the current sample.
    pub alarm_status: AlarmStatus,
    /// Human-readable alarm message.
    pub alarm_message: String,
    /// Optional display metadata for the PV.
    pub display_metadata: Option<DisplayMetadata>,
    /// Optional control metadata for the PV.
    pub control_metadata: Option<ControlMetadata>,
    /// Optional alarm metadata for the PV.
    pub alarm_metadata: Option<AlarmMetadata>,
}

/// Snapshot of a fetched int32 PV value plus its metadata.
#[derive(Debug, Clone)]
pub struct FetchedInt32 {
    /// Current numeric value.
    pub value: i32,
    /// Alarm severity for the current sample.
    pub alarm_severity: AlarmSeverity,
    /// Alarm status for the current sample.
    pub alarm_status: AlarmStatus,
    /// Human-readable alarm message.
    pub alarm_message: String,
    /// Optional display metadata for the PV.
    pub display_metadata: Option<DisplayMetadata>,
    /// Optional control metadata for the PV.
    pub control_metadata: Option<ControlMetadata>,
    /// Optional alarm metadata for the PV.
    pub alarm_metadata: Option<AlarmMetadata>,
}

/// Snapshot of a fetched string PV value plus its metadata.
#[derive(Debug, Clone)]
pub struct FetchedString {
    /// Current string value.
    pub value: String,
    /// Alarm severity for the current sample.
    pub alarm_severity: AlarmSeverity,
    /// Alarm status for the current sample.
    pub alarm_status: AlarmStatus,
    /// Human-readable alarm message.
    pub alarm_message: String,
}

/// Snapshot of a fetched double-array PV value plus its metadata.
#[derive(Debug, Clone)]
pub struct FetchedDoubleArray {
    /// Current array value.
    pub value: Vec<f64>,
    /// Alarm severity for the current sample.
    pub alarm_severity: AlarmSeverity,
    /// Alarm status for the current sample.
    pub alarm_status: AlarmStatus,
    /// Human-readable alarm message.
    pub alarm_message: String,
    /// Optional display metadata for the PV.
    pub display_metadata: Option<DisplayMetadata>,
    /// Optional control metadata for the PV.
    pub control_metadata: Option<ControlMetadata>,
    /// Optional alarm metadata for the PV.
    pub alarm_metadata: Option<AlarmMetadata>,
}

/// Snapshot of a fetched int32-array PV value plus its metadata.
#[derive(Debug, Clone)]
pub struct FetchedInt32Array {
    /// Current array value.
    pub value: Vec<i32>,
    /// Alarm severity for the current sample.
    pub alarm_severity: AlarmSeverity,
    /// Alarm status for the current sample.
    pub alarm_status: AlarmStatus,
    /// Human-readable alarm message.
    pub alarm_message: String,
    /// Optional display metadata for the PV.
    pub display_metadata: Option<DisplayMetadata>,
    /// Optional control metadata for the PV.
    pub control_metadata: Option<ControlMetadata>,
    /// Optional alarm metadata for the PV.
    pub alarm_metadata: Option<AlarmMetadata>,
}

/// Snapshot of a fetched string-array PV value plus its metadata.
#[derive(Debug, Clone)]
pub struct FetchedStringArray {
    /// Current array value.
    pub value: Vec<String>,
    /// Alarm severity for the current sample.
    pub alarm_severity: AlarmSeverity,
    /// Alarm status for the current sample.
    pub alarm_status: AlarmStatus,
    /// Human-readable alarm message.
    pub alarm_message: String,
}

/// Snapshot of a fetched enum PV value plus its metadata.
#[derive(Debug, Clone)]
pub struct FetchedEnum {
    /// Currently selected enum index.
    pub value: i16,
    /// Available enum choice names.
    pub value_choices: Vec<String>,
    /// Alarm severity for the current sample.
    pub alarm_severity: AlarmSeverity,
    /// Alarm status for the current sample.
    pub alarm_status: AlarmStatus,
    /// Human-readable alarm message.
    pub alarm_message: String,
}

// ============================================================================
// SharedPV / StaticSource compatibility types
// ============================================================================

/// A shared process variable that can be hosted by a server.
///
/// A mailbox PV accepts client reads and writes, while a readonly PV accepts
/// client reads but can still be updated by its server through the `post_*`
/// methods.
///
/// # Example
///
/// ```rust
/// use pvxs::server::{SharedPV, NTScalarMetadataBuilder};
///
/// let mut pv = SharedPV::create_mailbox()?;
/// assert!(!pv.is_open());
/// pv.close()?;
/// # Ok::<(), pvxs::PvxsError>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct SharedPV {
    state: Arc<Mutex<SharedPVState>>,
}

#[derive(Debug, Default)]
struct SharedPVState {
    readonly: bool,
    value: Option<crate::Value>,
}

#[allow(dead_code)]
impl SharedPV {
    /// Create a mailbox PV that accepts client reads and writes.
    pub fn create_mailbox() -> Result<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(SharedPVState::default())),
        })
    }

    /// Create a PV that accepts client reads but rejects client writes.
    ///
    /// The server may still update the PV through the `post_*` methods.
    pub fn create_readonly() -> Result<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(SharedPVState {
                readonly: true,
                value: None,
            })),
        })
    }

    /// Open the PV with a double value and scalar metadata.
    pub(crate) fn open_double(
        &mut self,
        initial_value: f64,
        _metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.set_value(crate::Value::nt_scalar_double(initial_value))?;
        Ok(())
    }

    /// Open the PV with a double-array value and scalar-array metadata.
    pub(crate) fn open_double_array(
        &mut self,
        initial_value: Vec<f64>,
        _metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.set_value(crate::Value::nt_scalar_array_double(initial_value))?;
        Ok(())
    }

    /// Open the PV with an int32 value and scalar metadata.
    pub(crate) fn open_int32(
        &mut self,
        initial_value: i32,
        _metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.set_value(crate::Value::nt_scalar_int32(initial_value))?;
        Ok(())
    }

    /// Open the PV with an int32-array value and scalar-array metadata.
    pub(crate) fn open_int32_array(
        &mut self,
        initial_value: Vec<i32>,
        _metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.set_value(crate::Value::nt_scalar_array_int32(initial_value))?;
        Ok(())
    }

    /// Open the PV with a string value and scalar metadata.
    pub(crate) fn open_string(
        &mut self,
        initial_value: &str,
        _metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.set_value(crate::Value::nt_scalar_string(initial_value))?;
        Ok(())
    }

    /// Open the PV with a string-array value and scalar-array metadata.
    pub(crate) fn open_string_array(
        &mut self,
        initial_value: Vec<String>,
        _metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.set_value(crate::Value::nt_scalar_array_string(initial_value))?;
        Ok(())
    }

    /// Open the PV with enum choices, a selected index, and enum metadata.
    pub(crate) fn open_enum(
        &mut self,
        choices: Vec<&str>,
        selected_index: i16,
        _metadata: NTEnumMetadataBuilder,
    ) -> Result<()> {
        let choices: Vec<String> = choices.into_iter().map(|s| s.to_string()).collect();
        self.set_value(crate::Value::nt_enum(selected_index, choices))?;
        Ok(())
    }

    /// Check whether this PV currently has an open value.
    pub fn is_open(&self) -> bool {
        self.state
            .lock()
            .map(|state| state.value.is_some())
            .unwrap_or(false)
    }

    /// Close this PV and discard its current value.
    pub fn close(&mut self) -> Result<()> {
        self.state()?.value = None;
        Ok(())
    }

    /// Post a double value and notify users of this PV.
    ///
    /// For a double-array PV, this replaces the first array element.
    pub fn post_double(&mut self, value: f64) -> Result<()> {
        let current = self.fetch()?;
        let updated = match current.type_of("value") {
            Some(FieldType::Double) => crate::Value::nt_scalar_double(value),
            Some(FieldType::DoubleArray) => {
                let mut values = current.get_field_double_array("value")?;
                let first = values
                    .first_mut()
                    .ok_or_else(|| PvxsError::new("Cannot update an empty double array"))?;
                *first = value;
                crate::Value::nt_scalar_array_double(values)
            }
            _ => return Err(PvxsError::new("SharedPV does not contain a double value")),
        };
        self.set_value(updated)
    }

    /// Post an int32 value and notify users of this PV.
    ///
    /// For an int32-array PV, this replaces the first array element.
    pub fn post_int32(&mut self, value: i32) -> Result<()> {
        let current = self.fetch()?;
        let updated = match current.type_of("value") {
            Some(FieldType::Int32) => crate::Value::nt_scalar_int32(value),
            Some(FieldType::Int32Array) => {
                let mut values = current.get_field_int32_array("value")?;
                let first = values
                    .first_mut()
                    .ok_or_else(|| PvxsError::new("Cannot update an empty int32 array"))?;
                *first = value;
                crate::Value::nt_scalar_array_int32(values)
            }
            _ => return Err(PvxsError::new("SharedPV does not contain an int32 value")),
        };
        self.set_value(updated)
    }

    /// Post a string value and notify users of this PV.
    pub fn post_string(&mut self, value: &str) -> Result<()> {
        self.require_type(FieldType::String)?;
        self.set_value(crate::Value::nt_scalar_string(value))
    }

    /// Post an enum index and preserve the PV's existing choices.
    pub fn post_enum(&mut self, value: i16) -> Result<()> {
        let current = self.fetch()?;
        if current.type_of("value") != Some(FieldType::Enum) {
            return Err(PvxsError::new("SharedPV does not contain an enum value"));
        }
        let choices = current.get_field_string_array("value.choices")?;
        if value < 0 || value as usize >= choices.len() {
            return Err(PvxsError::new("Enum index is outside the choices array"));
        }
        self.set_value(crate::Value::nt_enum(value, choices))
    }

    /// Post a non-empty double array and notify users of this PV.
    pub fn post_double_array(&mut self, value: &[f64]) -> Result<()> {
        if value.is_empty() {
            return Err(PvxsError::new("Cannot post empty double array"));
        }
        self.require_type(FieldType::DoubleArray)?;
        self.set_value(crate::Value::nt_scalar_array_double(value.to_vec()))
    }

    /// Post a non-empty int32 array and notify users of this PV.
    pub fn post_int32_array(&mut self, value: &[i32]) -> Result<()> {
        if value.is_empty() {
            return Err(PvxsError::new("Cannot post empty int32 array"));
        }
        self.require_type(FieldType::Int32Array)?;
        self.set_value(crate::Value::nt_scalar_array_int32(value.to_vec()))
    }

    /// Post a non-empty string array and notify users of this PV.
    pub fn post_string_array(&mut self, value: &[String]) -> Result<()> {
        if value.is_empty() {
            return Err(PvxsError::new("Cannot post empty string array"));
        }
        self.require_type(FieldType::StringArray)?;
        self.set_value(crate::Value::nt_scalar_array_string(value.to_vec()))
    }

    /// Fetch the current structured value.
    pub fn fetch(&self) -> Result<crate::Value> {
        self.current()
            .ok_or_else(|| PvxsError::new("SharedPV is closed"))
    }

    /// Return the current value, or `None` when the PV is closed.
    pub(crate) fn current(&self) -> Option<crate::Value> {
        self.state.lock().ok()?.value.clone()
    }

    fn state(&self) -> Result<std::sync::MutexGuard<'_, SharedPVState>> {
        self.state
            .lock()
            .map_err(|_| PvxsError::new("SharedPV state lock poisoned"))
    }

    fn set_value(&self, value: crate::Value) -> Result<()> {
        self.state()?.value = Some(value);
        Ok(())
    }

    fn require_type(&self, expected: FieldType) -> Result<()> {
        let current = self.fetch()?;
        if current.type_of("value") == Some(expected) {
            Ok(())
        } else {
            Err(PvxsError::new(format!(
                "SharedPV does not contain a {expected:?} value"
            )))
        }
    }

    fn is_readonly(&self) -> Result<bool> {
        Ok(self.state()?.readonly)
    }
}

/// A named collection of shared PVs that can be attached to a server.
///
/// # Example
///
/// ```rust
/// use pvxs::{SharedPV, StaticSource};
///
/// let mut source = StaticSource::create()?;
/// let mut temperature = SharedPV::create_readonly()?;
/// source.add_pv("temperature", &mut temperature)?;
/// # Ok::<(), pvxs::PvxsError>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct StaticSource {
    pvs: HashMap<String, SharedPV>,
}

impl StaticSource {
    /// Create an empty static source.
    pub fn create() -> Result<Self> {
        Ok(Self::default())
    }

    /// Add a shared PV under the provided name.
    pub fn add_pv(&mut self, name: &str, pv: &mut SharedPV) -> Result<()> {
        self.insert(name, pv.clone())
    }

    /// Remove a PV from this source.
    pub fn remove_pv(&mut self, name: &str) -> Result<()> {
        self.pvs
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| PvxsError::new(format!("PV '{name}' does not exist")))
    }

    /// Close every PV in this source.
    pub fn close_all(&mut self) -> Result<()> {
        for pv in self.pvs.values_mut() {
            pv.close()?;
        }
        Ok(())
    }

    fn insert(&mut self, name: &str, pv: SharedPV) -> Result<()> {
        if self.pvs.contains_key(name) {
            return Err(PvxsError::new(format!("PV '{name}' already exists")));
        }
        self.pvs.insert(name.to_string(), pv);
        Ok(())
    }
}

// ============================================================================
// ServerHandle
// ============================================================================

/// Clone-able, thread-safe handle to a running server.
#[derive(Clone)]
pub struct ServerHandle {
    tx: channel::Sender<ManagerCommand>,
    /// Bound TCP port used by the pvAccess server.
    tcp_port: u16,
    /// Bound UDP port used for pvAccess discovery.
    udp_port: u16,
}

impl ServerHandle {
    /// Return the TCP port used by the server.
    pub fn tcp_port(&self) -> u16 {
        self.tcp_port
    }

    /// Return the UDP port used by the server.
    pub fn udp_port(&self) -> u16 {
        self.udp_port
    }

    fn send<T>(&self, cmd: ManagerCommand, rx: channel::Receiver<T>) -> Result<T> {
        self.tx
            .send(cmd)
            .map_err(|_| PvxsError::new("server worker stopped"))?;
        rx.recv()
            .map_err(|_| PvxsError::new("server worker stopped"))
    }

    fn set_readonly(&self, name: &str, readonly: bool) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::SetReadonly {
                name: name.to_string(),
                readonly,
                reply: tx,
            },
            rx,
        )?
    }

    /// Add a shared PV to the running server.
    pub(crate) fn add_shared_pv(&self, name: &str, pv: SharedPV) -> Result<()> {
        let value = pv
            .current()
            .ok_or_else(|| PvxsError::new("SharedPV must be opened before adding"))?;

        let value_type = value
            .type_of("value")
            .ok_or_else(|| PvxsError::new("SharedPV value missing required 'value' field"))?;

        match value_type {
            FieldType::Double => {
                self.create_pv_double(
                    name,
                    value.get_field_double("value")?,
                    NTScalarMetadataBuilder::new(),
                )?;
            }
            FieldType::Int32 => {
                self.create_pv_int32(
                    name,
                    value.get_field_int32("value")?,
                    NTScalarMetadataBuilder::new(),
                )?;
            }
            FieldType::String => {
                self.create_pv_string(
                    name,
                    &value.get_field_string("value")?,
                    NTScalarMetadataBuilder::new(),
                )?;
            }
            FieldType::Enum => {
                let current = value.get_field_enum("value")?;
                let choices_owned = value.get_field_string_array("value.choices")?;
                let choices: Vec<&str> = choices_owned.iter().map(|s| s.as_str()).collect();
                self.create_pv_enum(name, choices, current, NTEnumMetadataBuilder::new())?;
            }
            FieldType::DoubleArray => {
                self.create_pv_double_array(
                    name,
                    value.get_field_double_array("value")?,
                    NTScalarMetadataBuilder::new(),
                )?;
            }
            FieldType::Int32Array => {
                self.create_pv_int32_array(
                    name,
                    value.get_field_int32_array("value")?,
                    NTScalarMetadataBuilder::new(),
                )?;
            }
            FieldType::StringArray => {
                self.create_pv_string_array(
                    name,
                    value.get_field_string_array("value")?,
                    NTScalarMetadataBuilder::new(),
                )?;
            }
            FieldType::Int64 | FieldType::Bool => {
                return Err(PvxsError::new(format!(
                    "SharedPV type '{value_type:?}' is not supported by server registry"
                )));
            }
        }

        if pv.is_readonly()? {
            self.set_readonly(name, true)?;
        }

        Ok(())
    }

    /// Add all shared PVs from a static source to the running server.
    pub(crate) fn add_source(&self, source: StaticSource) -> Result<()> {
        for (name, pv) in source.pvs {
            self.add_shared_pv(&name, pv)?;
        }
        Ok(())
    }

    /// Create a double PV in the in-memory registry.
    pub fn create_pv_double(
        &self,
        name: &str,
        initial: f64,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateDouble {
                name: name.to_string(),
                initial,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    /// Create a double-array PV in the in-memory registry.
    pub fn create_pv_double_array(
        &self,
        name: &str,
        initial: Vec<f64>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateDoubleArray {
                name: name.to_string(),
                initial,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    /// Create an int32 PV in the in-memory registry.
    pub fn create_pv_int32(
        &self,
        name: &str,
        initial: i32,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateInt32 {
                name: name.to_string(),
                initial,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    /// Create an int32-array PV in the in-memory registry.
    pub fn create_pv_int32_array(
        &self,
        name: &str,
        initial: Vec<i32>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateInt32Array {
                name: name.to_string(),
                initial,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    /// Create a string PV in the in-memory registry.
    pub fn create_pv_string(
        &self,
        name: &str,
        initial: &str,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateString {
                name: name.to_string(),
                initial: initial.to_string(),
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    /// Create a string-array PV in the in-memory registry.
    pub fn create_pv_string_array(
        &self,
        name: &str,
        initial: Vec<String>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateStringArray {
                name: name.to_string(),
                initial,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    /// Create an enum PV in the in-memory registry.
    pub fn create_pv_enum(
        &self,
        name: &str,
        choices: Vec<&str>,
        selected_index: i16,
        metadata: NTEnumMetadataBuilder,
    ) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::CreateEnum {
                name: name.to_string(),
                choices: choices.iter().map(|s| s.to_string()).collect(),
                selected_index,
                metadata,
                reply: tx,
            },
            rx,
        )?
    }

    /// Post a new value to an existing double PV.
    pub fn post_double(&self, name: &str, value: f64) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostDouble {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    /// Post a new value to an existing double-array PV.
    pub fn post_double_array(&self, name: &str, value: Vec<f64>) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostDoubleArray {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    /// Post a new value to an existing int32 PV.
    pub fn post_int32(&self, name: &str, value: i32) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostInt32 {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    /// Post a new value to an existing int32-array PV.
    pub fn post_int32_array(&self, name: &str, value: Vec<i32>) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostInt32Array {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    /// Post a new value to an existing string PV.
    pub fn post_string(&self, name: &str, value: &str) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostString {
                name: name.to_string(),
                value: value.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    /// Post a new value to an existing string-array PV.
    pub fn post_string_array(&self, name: &str, value: Vec<String>) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostStringArray {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    /// Post a new value to an existing enum PV.
    pub fn post_enum(&self, name: &str, value: i16) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::PostEnum {
                name: name.to_string(),
                value,
                reply: tx,
            },
            rx,
        )?
    }

    /// Remove an existing PV from the in-memory registry.
    pub fn remove_pv(&self, name: &str) -> Result<()> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::Remove {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    /// Fetch the current value of a double PV.
    pub fn fetch_double(&self, name: &str) -> Result<FetchedDouble> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchDouble {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    /// Fetch the current value of an int32 PV.
    pub fn fetch_int32(&self, name: &str) -> Result<FetchedInt32> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchInt32 {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    /// Fetch the current value of a string PV.
    pub fn fetch_string(&self, name: &str) -> Result<FetchedString> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchString {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    /// Fetch the current value of a double-array PV.
    pub fn fetch_double_array(&self, name: &str) -> Result<FetchedDoubleArray> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchDoubleArray {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    /// Fetch the current value of an int32-array PV.
    pub fn fetch_int32_array(&self, name: &str) -> Result<FetchedInt32Array> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchInt32Array {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    /// Fetch the current value of a string-array PV.
    pub fn fetch_string_array(&self, name: &str) -> Result<FetchedStringArray> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchStringArray {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }

    /// Fetch the current value of an enum PV.
    pub fn fetch_enum(&self, name: &str) -> Result<FetchedEnum> {
        let (tx, rx) = channel::bounded(1);
        self.send(
            ManagerCommand::FetchEnum {
                name: name.to_string(),
                reply: tx,
            },
            rx,
        )?
    }
}

// ============================================================================
// Server
// ============================================================================

/// Pure-Rust pvAccess server with automatic alarm management.
pub struct Server {
    handle: ServerHandle,
    join: Option<thread::JoinHandle<()>>,
    beacon_stop_tx: Option<watch::Sender<bool>>,
    beacon_join: Option<thread::JoinHandle<()>>,
    udp_join: Option<thread::JoinHandle<()>>,
    tcp_join: Option<thread::JoinHandle<()>>,
}

impl Server {
    /// Start a server configured from environment variables.
    pub fn start_from_env() -> Result<Self> {
        let udp_port = std::env::var("EPICS_PVA_BROADCAST_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5076);
        let tcp_port = std::env::var("EPICS_PVA_SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5075);
        Self::start_inner(udp_port, tcp_port)
    }

    /// Start an isolated server (system-assigned ports, ideal for tests).
    pub fn start_isolated() -> Result<Self> {
        Self::start_inner(0, 0)
    }

    fn start_inner(udp_bind_port: u16, tcp_bind_port: u16) -> Result<Self> {
        let (tx, rx) = channel::unbounded::<ManagerCommand>();
        let tx_for_tcp = tx.clone();
        let join = thread::spawn(move || run_worker(rx));

        let (beacon_stop_tx, mut beacon_stop_rx) = watch::channel(false);
        let mut udp_stop_rx = beacon_stop_tx.subscribe();
        let (udp_ready_tx, udp_ready_rx) = mpsc::channel::<u16>();
        let tcp_port_shared = Arc::new(AtomicU16::new(0));
        let tcp_port_for_udp = Arc::clone(&tcp_port_shared);

        let udp_join = thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => {
                    let _ = udp_ready_tx.send(0);
                    return;
                }
            };

            rt.block_on(async move {
                let udp_sock = match if udp_bind_port == 0 {
                    epics_libcom_rs::net::AsyncUdpV4::bind_ephemeral_same_port(true)
                } else {
                    epics_libcom_rs::net::AsyncUdpV4::bind(udp_bind_port, true)
                } {
                    Ok(sock) => sock,
                    Err(_) => {
                        let _ = udp_ready_tx.send(0);
                        return;
                    }
                };
                let udp_port = udp_sock
                    .local_addrs()
                    .first()
                    .map(|sa| sa.port())
                    .unwrap_or(0);
                let _ = udp_ready_tx.send(udp_port);

                let mut buf = vec![0u8; 4096];
                loop {
                    tokio::select! {
                        changed = udp_stop_rx.changed() => {
                            if changed.is_err() || *udp_stop_rx.borrow() {
                                break;
                            }
                        }
                        recv = udp_sock.recv_with_meta(&mut buf) => {
                            let meta = match recv {
                                Ok(meta) => meta,
                                Err(_) => continue,
                            };
                            let pkt = &buf[..meta.n];
                            if pkt.len() < 8 {
                                continue;
                            }

                            let hdr: [u8; 8] = [pkt[0], pkt[1], pkt[2], pkt[3], pkt[4], pkt[5], pkt[6], pkt[7]];
                            let Some((_from_server, cmd, payload_len)) = crate::proto::decode_header(&hdr) else {
                                continue;
                            };
                            if cmd != CMD_SEARCH {
                                continue;
                            }

                            let plen = payload_len as usize;
                            if pkt.len() < 8 + plen {
                                continue;
                            }
                            let payload = &pkt[8..8 + plen];
                            let Some((seq_id, cid)) = parse_search_request(payload) else {
                                continue;
                            };

                            let reply = build_search_response(
                                seq_id,
                                cid,
                                meta.iface_ip,
                                tcp_port_for_udp.load(Ordering::Relaxed),
                            );
                            let _ = udp_sock.send_via(&reply, meta.src, meta.iface_ip).await;
                        }
                    }
                }
            });
        });

        let udp_port = udp_ready_rx
            .recv_timeout(Duration::from_secs(2))
            .unwrap_or(0);

        let mut tcp_stop_rx = beacon_stop_tx.subscribe();
        let (tcp_ready_tx, tcp_ready_rx) = mpsc::channel::<u16>();
        let tcp_join = thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => {
                    let _ = tcp_ready_tx.send(0);
                    return;
                }
            };

            rt.block_on(async move {
                let bind_addr = format!("0.0.0.0:{tcp_bind_port}");
                let listener = match TcpListener::bind(&bind_addr).await {
                    Ok(l) => l,
                    Err(_) => {
                        let _ = tcp_ready_tx.send(0);
                        return;
                    }
                };
                let port = listener.local_addr().ok().map(|a| a.port()).unwrap_or(0);
                let _ = tcp_ready_tx.send(port);
                let mut connections = JoinSet::new();

                loop {
                    tokio::select! {
                        changed = tcp_stop_rx.changed() => {
                            if changed.is_err() || *tcp_stop_rx.borrow() {
                                break;
                            }
                        }
                        accept = listener.accept() => {
                            let (stream, _) = match accept {
                                Ok(v) => v,
                                Err(_) => continue,
                            };
                            let tx = tx_for_tcp.clone();
                            connections.spawn(async move {
                                let _ = handle_client(stream, tx).await;
                            });
                        }
                        completed = connections.join_next(), if !connections.is_empty() => {
                            let _ = completed;
                        }
                    }
                }

                connections.abort_all();
                while connections.join_next().await.is_some() {}
            });
        });

        let tcp_port = tcp_ready_rx
            .recv_timeout(Duration::from_secs(2))
            .unwrap_or(0);
        tcp_port_shared.store(tcp_port, Ordering::Relaxed);

        let beacon_dest_port = if udp_bind_port == 0 {
            udp_port
        } else {
            udp_bind_port
        };

        let beacon_join = thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => return,
            };

            rt.block_on(async move {
                let beacon_sock =
                    epics_libcom_rs::net::AsyncUdpV4::bind_ephemeral_same_port(true).ok();
                let beacon_dest = SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(255, 255, 255, 255),
                    beacon_dest_port,
                ));
                let mut seq_id: u32 = 1;
                let mut ticker = epics_libcom_rs::runtime::task::interval(Duration::from_secs(15));
                loop {
                    tokio::select! {
                        _ = ticker.tick() => {
                            if let Some(sock) = &beacon_sock {
                                let pkt = build_beacon(seq_id, 0);
                                let _ = sock.fanout_to(&pkt, beacon_dest).await;
                                seq_id = seq_id.wrapping_add(1);
                            }
                        }
                        changed = beacon_stop_rx.changed() => {
                            if changed.is_err() || *beacon_stop_rx.borrow() {
                                break;
                            }
                        }
                    }
                }
            });
        });
        Ok(Self {
            handle: ServerHandle {
                tx,
                tcp_port,
                udp_port,
            },
            join: Some(join),
            beacon_stop_tx: Some(beacon_stop_tx),
            beacon_join: Some(beacon_join),
            udp_join: Some(udp_join),
            tcp_join: Some(tcp_join),
        })
    }

    /// Get a clone-able handle to this server for use from other threads.
    pub fn handle(&self) -> ServerHandle {
        self.handle.clone()
    }

    /// TCP port the server is listening on (0 until transport layer is implemented).
    pub fn tcp_port(&self) -> u16 {
        self.handle.tcp_port()
    }

    /// UDP port the server is using (0 until transport layer is implemented).
    pub fn udp_port(&self) -> u16 {
        self.handle.udp_port()
    }

    /// Create a double PV in the in-memory registry.
    pub fn create_pv_double(
        &self,
        name: &str,
        initial: f64,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_double(name, initial, metadata)
    }

    /// Create a double-array PV in the in-memory registry.
    pub fn create_pv_double_array(
        &self,
        name: &str,
        initial: Vec<f64>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_double_array(name, initial, metadata)
    }

    /// Create an int32 PV in the in-memory registry.
    pub fn create_pv_int32(
        &self,
        name: &str,
        initial: i32,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_int32(name, initial, metadata)
    }

    /// Create an int32-array PV in the in-memory registry.
    pub fn create_pv_int32_array(
        &self,
        name: &str,
        initial: Vec<i32>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_int32_array(name, initial, metadata)
    }

    /// Create a string PV in the in-memory registry.
    pub fn create_pv_string(
        &self,
        name: &str,
        initial: &str,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_string(name, initial, metadata)
    }

    /// Create a string-array PV in the in-memory registry.
    pub fn create_pv_string_array(
        &self,
        name: &str,
        initial: Vec<String>,
        metadata: NTScalarMetadataBuilder,
    ) -> Result<()> {
        self.handle.create_pv_string_array(name, initial, metadata)
    }

    /// Create an enum PV in the in-memory registry.
    pub fn create_pv_enum(
        &self,
        name: &str,
        choices: Vec<&str>,
        selected_index: i16,
        metadata: NTEnumMetadataBuilder,
    ) -> Result<()> {
        self.handle
            .create_pv_enum(name, choices, selected_index, metadata)
    }

    /// Add a shared PV to the running server.
    #[allow(dead_code)]
    pub(crate) fn add_shared_pv(&self, name: &str, pv: SharedPV) -> Result<()> {
        self.handle.add_shared_pv(name, pv)
    }

    /// Add all shared PVs from a static source to the running server.
    #[allow(dead_code)]
    pub(crate) fn add_source(&self, source: StaticSource) -> Result<()> {
        self.handle.add_source(source)
    }

    /// Post a new value to an existing double PV.
    pub fn post_double(&self, name: &str, value: f64) -> Result<()> {
        self.handle.post_double(name, value)
    }

    /// Post a new value to an existing double-array PV.
    pub fn post_double_array(&self, name: &str, value: Vec<f64>) -> Result<()> {
        self.handle.post_double_array(name, value)
    }

    /// Post a new value to an existing int32 PV.
    pub fn post_int32(&self, name: &str, value: i32) -> Result<()> {
        self.handle.post_int32(name, value)
    }

    /// Post a new value to an existing int32-array PV.
    pub fn post_int32_array(&self, name: &str, value: Vec<i32>) -> Result<()> {
        self.handle.post_int32_array(name, value)
    }

    /// Post a new value to an existing string PV.
    pub fn post_string(&self, name: &str, value: &str) -> Result<()> {
        self.handle.post_string(name, value)
    }

    /// Post a new value to an existing string-array PV.
    pub fn post_string_array(&self, name: &str, value: Vec<String>) -> Result<()> {
        self.handle.post_string_array(name, value)
    }

    /// Post a new value to an existing enum PV.
    pub fn post_enum(&self, name: &str, value: i16) -> Result<()> {
        self.handle.post_enum(name, value)
    }

    /// Remove an existing PV from the in-memory registry.
    pub fn remove_pv(&self, name: &str) -> Result<()> {
        self.handle.remove_pv(name)
    }

    /// Fetch the current value of a double PV.
    pub fn fetch_double(&self, name: &str) -> Result<FetchedDouble> {
        self.handle.fetch_double(name)
    }

    /// Fetch the current value of an int32 PV.
    pub fn fetch_int32(&self, name: &str) -> Result<FetchedInt32> {
        self.handle.fetch_int32(name)
    }

    /// Fetch the current value of a string PV.
    pub fn fetch_string(&self, name: &str) -> Result<FetchedString> {
        self.handle.fetch_string(name)
    }

    /// Fetch the current value of a double-array PV.
    pub fn fetch_double_array(&self, name: &str) -> Result<FetchedDoubleArray> {
        self.handle.fetch_double_array(name)
    }

    /// Fetch the current value of an int32-array PV.
    pub fn fetch_int32_array(&self, name: &str) -> Result<FetchedInt32Array> {
        self.handle.fetch_int32_array(name)
    }

    /// Fetch the current value of a string-array PV.
    pub fn fetch_string_array(&self, name: &str) -> Result<FetchedStringArray> {
        self.handle.fetch_string_array(name)
    }

    /// Fetch the current value of an enum PV.
    pub fn fetch_enum(&self, name: &str) -> Result<FetchedEnum> {
        self.handle.fetch_enum(name)
    }

    /// Stop the server, consuming it and freeing all resources.
    pub fn stop_drop(mut self) -> Result<()> {
        if let Some(beacon_stop_tx) = self.beacon_stop_tx.take() {
            let _ = beacon_stop_tx.send(true);
        }
        let (tx, rx) = channel::bounded(1);
        self.handle
            .tx
            .send(ManagerCommand::Stop { reply: tx })
            .map_err(|_| PvxsError::new("server worker stopped"))?;
        let result = rx
            .recv()
            .map_err(|_| PvxsError::new("server worker stopped"))?;
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        if let Some(join) = self.beacon_join.take() {
            let _ = join.join();
        }
        if let Some(join) = self.udp_join.take() {
            let _ = join.join();
        }
        if let Some(join) = self.tcp_join.take() {
            let _ = join.join();
        }
        result
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(beacon_stop_tx) = self.beacon_stop_tx.take() {
            let _ = beacon_stop_tx.send(true);
        }

        // If stop_drop was not called, send Stop anyway so the worker exits.
        if self.join.is_some() {
            let (tx, _rx) = channel::bounded(1);
            let _ = self.handle.tx.send(ManagerCommand::Stop { reply: tx });
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }

        if let Some(join) = self.beacon_join.take() {
            let _ = join.join();
        }
        if let Some(join) = self.udp_join.take() {
            let _ = join.join();
        }
        if let Some(join) = self.tcp_join.take() {
            let _ = join.join();
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> Server {
        Server::start_isolated().expect("server start")
    }

    #[test]
    fn create_and_fetch_double() {
        let s = server();
        s.create_pv_double("A", std::f64::consts::PI, NTScalarMetadataBuilder::new())
            .unwrap();
        let f = s.fetch_double("A").unwrap();
        assert!((f.value - std::f64::consts::PI).abs() < 1e-9);
        assert_eq!(f.alarm_severity, AlarmSeverity::NoAlarm);
        s.stop_drop().unwrap();
    }

    #[test]
    fn post_double_updates_value() {
        let s = server();
        s.create_pv_double("B", 0.0, NTScalarMetadataBuilder::new())
            .unwrap();
        s.post_double("B", 42.0).unwrap();
        let f = s.fetch_double("B").unwrap();
        assert!((f.value - 42.0).abs() < 1e-9);
        s.stop_drop().unwrap();
    }

    #[test]
    fn duplicate_pv_name_errors() {
        let s = server();
        s.create_pv_double("C", 0.0, NTScalarMetadataBuilder::new())
            .unwrap();
        assert!(s
            .create_pv_double("C", 1.0, NTScalarMetadataBuilder::new())
            .is_err());
        s.stop_drop().unwrap();
    }

    #[test]
    fn create_and_fetch_int32() {
        let s = server();
        s.create_pv_int32("D", 7, NTScalarMetadataBuilder::new())
            .unwrap();
        let f = s.fetch_int32("D").unwrap();
        assert_eq!(f.value, 7);
        s.stop_drop().unwrap();
    }

    #[test]
    fn create_and_fetch_string() {
        let s = server();
        s.create_pv_string("E", "hello", NTScalarMetadataBuilder::new())
            .unwrap();
        let f = s.fetch_string("E").unwrap();
        assert_eq!(f.value, "hello");
        s.stop_drop().unwrap();
    }

    #[test]
    fn create_and_fetch_enum() {
        let s = server();
        s.create_pv_enum("F", vec!["OFF", "ON"], 1, NTEnumMetadataBuilder::new())
            .unwrap();
        let f = s.fetch_enum("F").unwrap();
        assert_eq!(f.value, 1);
        assert_eq!(f.value_choices, vec!["OFF", "ON"]);
        s.stop_drop().unwrap();
    }

    #[test]
    fn create_and_fetch_double_array() {
        let s = server();
        s.create_pv_double_array("G", vec![1.0, 2.0, 3.0], NTScalarMetadataBuilder::new())
            .unwrap();
        let f = s.fetch_double_array("G").unwrap();
        assert_eq!(f.value, vec![1.0, 2.0, 3.0]);
        s.stop_drop().unwrap();
    }

    #[test]
    fn remove_pv() {
        let s = server();
        s.create_pv_double("H", 0.0, NTScalarMetadataBuilder::new())
            .unwrap();
        s.remove_pv("H").unwrap();
        assert!(s.fetch_double("H").is_err());
        s.stop_drop().unwrap();
    }

    #[test]
    fn control_limit_rejection() {
        use crate::ControlMetadata;
        let s = server();
        let meta = NTScalarMetadataBuilder::new().control(ControlMetadata {
            limit_low: 0.0,
            limit_high: 10.0,
            min_step: 0.0,
        });
        s.create_pv_double("I", 5.0, meta).unwrap();
        // Value outside control limits should be rejected
        assert!(s.post_double("I", 20.0).is_err());
        s.stop_drop().unwrap();
    }

    #[test]
    fn server_handle_clone() {
        let s = server();
        let h = s.handle();
        h.create_pv_double("J", 1.0, NTScalarMetadataBuilder::new())
            .unwrap();
        let f = h.fetch_double("J").unwrap();
        assert!((f.value - 1.0).abs() < 1e-9);
        s.stop_drop().unwrap();
    }
}
