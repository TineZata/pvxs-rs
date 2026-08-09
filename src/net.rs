// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
//! PvAccess transport: UDP channel search + TCP GET/PUT session.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

use crate::client::ClientConfig;
use crate::proto::*;
use crate::pvdata::{
    build_pv_request_all, decode_field_desc_cached, decode_into_value, read_bitset,
};
use crate::{PvxsError, Result, Value};

// ── Frame builders ────────────────────────────────────────────────────────

fn frame(from_server: bool, cmd: u8, payload: Vec<u8>) -> Vec<u8> {
    let mut out = encode_header(from_server, cmd, payload.len() as u32).to_vec();
    out.extend_from_slice(&payload);
    out
}

fn build_search(seq_id: u32, cid: u32, pv_name: &str, my_port: u16) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&seq_id.to_le_bytes()); // sequenceId
    p.push(0x00); // flags: broadcast OK
    p.extend_from_slice(&[0u8; 3]); // reserved
    p.extend_from_slice(&[0u8; 16]); // responseAddress: IN6ADDR_ANY
    p.extend_from_slice(&my_port.to_le_bytes()); // responsePort
    encode_size(0, &mut p); // protocol count = 0 (server picks TCP)
    encode_size(1, &mut p); // channel count = 1
    p.extend_from_slice(&cid.to_le_bytes()); // channelID
    encode_string(pv_name, &mut p); // channelName
    frame(false, CMD_SEARCH, p)
}

fn build_connection_validated() -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&(16u32 * 1024 * 1024).to_le_bytes()); // clientReceiveBufferSize
    p.extend_from_slice(&0x10u16.to_le_bytes()); // registryMaxSize
    p.extend_from_slice(&0u16.to_le_bytes()); // qosCode
    encode_string("anonymous", &mut p); // authPlugin
    p.push(STATUS_OK_NOMSG); // authData: null
    frame(false, CMD_CONNECTION_VALIDATED, p)
}

fn build_create_channel(cid: u32, pv_name: &str) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&1u16.to_le_bytes()); // count
    p.extend_from_slice(&cid.to_le_bytes()); // channelID
    encode_string(pv_name, &mut p); // channelName
    frame(false, CMD_CREATE_CHANNEL, p)
}

fn build_get_init(sid: u32, ioid: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&sid.to_le_bytes());
    p.extend_from_slice(&ioid.to_le_bytes());
    p.push(0x08); // subcmd = INIT
    p.extend_from_slice(build_pv_request_all());
    frame(false, CMD_GET, p)
}

fn build_get(sid: u32, ioid: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&sid.to_le_bytes());
    p.extend_from_slice(&ioid.to_le_bytes());
    p.push(0x00); // subcmd = GET
    frame(false, CMD_GET, p)
}

fn build_put_init(sid: u32, ioid: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&sid.to_le_bytes());
    p.extend_from_slice(&ioid.to_le_bytes());
    p.push(0x08); // subcmd = INIT
    p.extend_from_slice(build_pv_request_all());
    frame(false, CMD_PUT, p)
}

fn build_monitor_init(sid: u32, ioid: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&sid.to_le_bytes());
    p.extend_from_slice(&ioid.to_le_bytes());
    p.push(0x08); // subcmd = INIT
    p.extend_from_slice(build_pv_request_all());
    frame(false, CMD_MONITOR, p)
}

fn build_monitor_start(sid: u32, ioid: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&sid.to_le_bytes());
    p.extend_from_slice(&ioid.to_le_bytes());
    p.push(0x44); // subcmd = START (pipeline bit included)
    frame(false, CMD_MONITOR, p)
}

fn build_monitor_stop(sid: u32, ioid: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&sid.to_le_bytes());
    p.extend_from_slice(&ioid.to_le_bytes());
    p.push(0x40); // subcmd = STOP
    frame(false, CMD_MONITOR, p)
}

fn build_destroy_request(sid: u32, ioid: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&sid.to_le_bytes());
    p.extend_from_slice(&ioid.to_le_bytes());
    frame(false, CMD_DESTROY_REQUEST, p)
}

fn build_destroy_channel(cid: u32, sid: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&cid.to_le_bytes());
    p.extend_from_slice(&sid.to_le_bytes());
    frame(false, CMD_DESTROY_CHANNEL, p)
}

// ── TCP frame I/O ─────────────────────────────────────────────────────────

/// Read one complete pvAccess frame from the TCP stream.
async fn read_frame(stream: &mut TcpStream) -> std::io::Result<(bool, u8, Vec<u8>)> {
    let mut hdr = [0u8; 8];
    stream.read_exact(&mut hdr).await?;
    let (from_server, cmd, payload_len) = decode_header(&hdr).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "bad pvA magic/version")
    })?;
    let mut payload = vec![0u8; payload_len as usize];
    if payload_len > 0 {
        stream.read_exact(&mut payload).await?;
    }
    Ok((from_server, cmd, payload))
}

/// Read frames, discarding all until one with `want_cmd` arrives.
async fn expect_frame(stream: &mut TcpStream, want_cmd: u8) -> std::io::Result<Vec<u8>> {
    loop {
        let (_, cmd, payload) = read_frame(stream).await?;
        if cmd == want_cmd {
            return Ok(payload);
        }
        // Skip BEACON, ECHO, CONNECTION_VALIDATED echo, unsolicited frames
    }
}

// ── UDP Search ────────────────────────────────────────────────────────────

async fn search(config: &ClientConfig, pv_name: &str) -> Result<SocketAddr> {
    let sock = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| PvxsError::new(format!("UDP bind: {e}")))?;
    sock.set_broadcast(true)
        .map_err(|e| PvxsError::new(format!("SO_BROADCAST: {e}")))?;

    let my_port = sock
        .local_addr()
        .map_err(|e| PvxsError::new(format!("local_addr: {e}")))?
        .port();

    let datagram = build_search(1, 1, pv_name, my_port);

    let targets: Vec<String> = if !config.addr_list.is_empty() {
        config
            .addr_list
            .iter()
            .map(|a| {
                if a.contains(':') {
                    a.clone()
                } else {
                    format!("{}:{}", a, config.broadcast_port)
                }
            })
            .collect()
    } else {
        vec![format!("255.255.255.255:{}", config.broadcast_port)]
    };

    for target in &targets {
        sock.send_to(&datagram, target.as_str())
            .await
            .map_err(|e| PvxsError::new(format!("UDP send to {target}: {e}")))?;
    }

    // Wait for SEARCH_RESPONSE
    let mut buf = vec![0u8; 4096];
    loop {
        let (n, from) = sock
            .recv_from(&mut buf)
            .await
            .map_err(|e| PvxsError::new(format!("UDP recv: {e}")))?;

        let pkt = &buf[..n];
        if pkt.len() < 8 {
            continue;
        }

        let hdr: &[u8; 8] = pkt[..8].try_into().unwrap();
        let Some((true, CMD_SEARCH_RESPONSE, _)) = decode_header(hdr) else {
            continue;
        };

        // SEARCH_RESPONSE payload: guid[12] seqId(u32) addr[16] port(u16) …
        let payload = &pkt[8..];
        if payload.len() < 34 {
            continue;
        }

        let addr_bytes: [u8; 16] = payload[16..32].try_into().unwrap();
        let port = u16::from_le_bytes([payload[32], payload[33]]);

        let server_ip = if addr_bytes == [0u8; 16] {
            from.ip()
        } else if addr_bytes[..12] == [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF] {
            IpAddr::V4(Ipv4Addr::new(
                addr_bytes[12],
                addr_bytes[13],
                addr_bytes[14],
                addr_bytes[15],
            ))
        } else {
            IpAddr::V6(std::net::Ipv6Addr::from(addr_bytes))
        };

        return Ok(SocketAddr::new(server_ip, port));
    }
}

// ── TCP GET session ───────────────────────────────────────────────────────

async fn pva_get_inner(server: SocketAddr, pv_name: &str, op_timeout: Duration) -> Result<Value> {
    let mut stream = timeout(Duration::from_secs(5), TcpStream::connect(server))
        .await
        .map_err(|_| PvxsError::new("TCP connect timeout"))?
        .map_err(|e| PvxsError::new(format!("TCP connect: {e}")))?;

    // Step 4: Recv CONNECTION_VALIDATION from server
    let _cv = timeout(
        op_timeout,
        expect_frame(&mut stream, CMD_CONNECTION_VALIDATION),
    )
    .await
    .map_err(|_| PvxsError::new("timeout waiting for CONNECTION_VALIDATION"))?
    .map_err(|e| PvxsError::new(format!("read CONNECTION_VALIDATION: {e}")))?;

    // Step 5: Send CONNECTION_VALIDATED (anonymous)
    stream
        .write_all(&build_connection_validated())
        .await
        .map_err(|e| PvxsError::new(format!("write CONNECTION_VALIDATED: {e}")))?;

    // Step 7: Send CREATE_CHANNEL (server may echo CONNECTION_VALIDATED; expect_frame skips it)
    let cid: u32 = 1;
    let ioid: u32 = 1;
    stream
        .write_all(&build_create_channel(cid, pv_name))
        .await
        .map_err(|e| PvxsError::new(format!("write CREATE_CHANNEL: {e}")))?;

    // Step 8: Recv CREATE_CHANNEL response (cmd 0x07 from server)
    let cc = timeout(op_timeout, expect_frame(&mut stream, CMD_CREATE_CHANNEL))
        .await
        .map_err(|_| {
            PvxsError::new(format!(
                "timeout waiting for CREATE_CHANNEL response for '{pv_name}'"
            ))
        })?
        .map_err(|e| PvxsError::new(format!("read CREATE_CHANNEL response: {e}")))?;

    let mut cur = cc.as_slice();
    let _rcid = read_u32_le(&mut cur)
        .ok_or_else(|| PvxsError::new("CREATE_CHANNEL response truncated (cid)"))?;
    let sid = read_u32_le(&mut cur)
        .ok_or_else(|| PvxsError::new("CREATE_CHANNEL response truncated (sid)"))?;
    if !decode_status(&mut cur) {
        return Err(PvxsError::new(format!(
            "server rejected CREATE_CHANNEL for '{pv_name}'"
        )));
    }

    // Step 9: Send GET INIT
    stream
        .write_all(&build_get_init(sid, ioid))
        .await
        .map_err(|e| PvxsError::new(format!("write GET INIT: {e}")))?;

    // Step 10: Recv GET INIT response (cmd 0x0A from server, subcmd has INIT bit)
    let gi = timeout(op_timeout, expect_frame(&mut stream, CMD_GET))
        .await
        .map_err(|_| PvxsError::new("timeout waiting for GET INIT response"))?
        .map_err(|e| PvxsError::new(format!("read GET INIT response: {e}")))?;

    let mut cur = gi.as_slice();
    let _rioid =
        read_u32_le(&mut cur).ok_or_else(|| PvxsError::new("GET INIT response: missing ioid"))?;
    let _subcmd = take_byte(&mut cur);
    if !decode_status(&mut cur) {
        return Err(PvxsError::new(format!("GET INIT failed for '{pv_name}'")));
    }
    let desc = decode_field_desc_cached(&mut cur).ok_or_else(|| {
        PvxsError::new(format!(
            "could not parse FieldDesc from GET INIT for '{pv_name}'"
        ))
    })?;

    // Step 11: Send GET
    stream
        .write_all(&build_get(sid, ioid))
        .await
        .map_err(|e| PvxsError::new(format!("write GET: {e}")))?;

    // Step 12: Recv GET data (cmd 0x0A from server, subcmd = 0x00)
    let gd = timeout(op_timeout, expect_frame(&mut stream, CMD_GET))
        .await
        .map_err(|_| PvxsError::new("timeout waiting for GET data"))?
        .map_err(|e| PvxsError::new(format!("read GET data: {e}")))?;

    let mut cur = gd.as_slice();
    let _rioid = read_u32_le(&mut cur).ok_or_else(|| PvxsError::new("GET data: missing ioid"))?;
    let _subcmd = take_byte(&mut cur);
    if !decode_status(&mut cur) {
        return Err(PvxsError::new(format!("GET data failed for '{pv_name}'")));
    }
    let bits = read_bitset(&mut cur)
        .ok_or_else(|| PvxsError::new("GET data: could not read BitSet"))?
        .to_vec(); // own the bytes before cur moves

    // Step 13: Decode value payload → Value
    let mut value = Value::new();
    let mut bit_counter: usize = 0;
    decode_into_value(&mut cur, &desc, &bits, &mut bit_counter, "", &mut value).ok_or_else(
        || PvxsError::new(format!("failed to decode pvData payload for '{pv_name}'")),
    )?;

    // Steps 14–15: Cleanup (best-effort; ignore errors, connection closes anyway)
    let _ = stream.write_all(&build_destroy_request(sid, ioid)).await;
    let _ = stream.write_all(&build_destroy_channel(cid, sid)).await;

    Ok(value)
}

// ── TCP PUT helpers ───────────────────────────────────────────────────────

/// Encode a single scalar value as PUT payload bytes (BitSet + value).
fn encode_put_payload(put_value: &PutValue) -> Vec<u8> {
    let mut p = Vec::new();
    // BitSet: 1 byte (bit 1 set = "value" field present; bit 0 is root structure)
    encode_size(1, &mut p); // BitSet byte count = 1
    p.push(0b0000_0010); // bit 1 = field index 1 (the "value" leaf)

    match put_value {
        PutValue::Double(v) => p.extend_from_slice(&v.to_bits().to_le_bytes()),
        PutValue::Int32(v) => p.extend_from_slice(&v.to_le_bytes()),
        PutValue::String(s) => encode_string(s, &mut p),
        PutValue::Enum(v) => p.extend_from_slice(&(v.to_le_bytes())),
        PutValue::DoubleArray(values) => {
            p.extend_from_slice(&(values.len() as u32).to_le_bytes());
            for value in values {
                p.extend_from_slice(&value.to_bits().to_le_bytes());
            }
        }
        PutValue::Int32Array(values) => {
            p.extend_from_slice(&(values.len() as u32).to_le_bytes());
            for value in values {
                p.extend_from_slice(&value.to_le_bytes());
            }
        }
        PutValue::StringArray(values) => {
            p.extend_from_slice(&(values.len() as u32).to_le_bytes());
            for value in values {
                encode_string(value, &mut p);
            }
        }
    }
    p
}

/// Typed value for a PUT operation.
pub enum PutValue<'a> {
    Double(f64),
    Int32(i32),
    String(&'a str),
    Enum(i16),
    DoubleArray(Vec<f64>),
    Int32Array(Vec<i32>),
    StringArray(Vec<String>),
}

/// Monitor-stream events emitted by the network task.
#[derive(Debug)]
pub enum MonitorNetEvent {
    Connected,
    Disconnected(String),
    Value(Value),
    RemoteError(String),
    ClientError(String),
    Finished,
}

/// Handle for an active pvAccess monitor session.
pub struct MonitorSession {
    stop_tx: Option<oneshot::Sender<()>>,
}

impl MonitorSession {
    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for MonitorSession {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn pva_monitor_task(
    config: ClientConfig,
    pv_name: String,
    timeout_secs: f64,
    mut stop_rx: oneshot::Receiver<()>,
    tx: mpsc::UnboundedSender<MonitorNetEvent>,
) {
    let op_timeout = Duration::from_secs_f64(timeout_secs.clamp(0.1, 300.0));

    let server = match timeout(op_timeout, search(&config, &pv_name)).await {
        Ok(Ok(server)) => server,
        Ok(Err(err)) => {
            let _ = tx.send(MonitorNetEvent::ClientError(err.to_string()));
            return;
        }
        Err(_) => {
            let _ = tx.send(MonitorNetEvent::ClientError(format!(
                "search timeout: '{}' not found",
                pv_name
            )));
            return;
        }
    };

    let mut stream = match timeout(Duration::from_secs(5), TcpStream::connect(server)).await {
        Ok(Ok(stream)) => stream,
        Ok(Err(err)) => {
            let _ = tx.send(MonitorNetEvent::ClientError(format!("TCP connect: {err}")));
            return;
        }
        Err(_) => {
            let _ = tx.send(MonitorNetEvent::ClientError(
                "TCP connect timeout".to_string(),
            ));
            return;
        }
    };

    let read_validation = timeout(
        op_timeout,
        expect_frame(&mut stream, CMD_CONNECTION_VALIDATION),
    )
    .await;
    if read_validation.is_err() {
        let _ = tx.send(MonitorNetEvent::ClientError(
            "timeout waiting for CONNECTION_VALIDATION".to_string(),
        ));
        return;
    }
    if let Ok(Err(err)) = read_validation {
        let _ = tx.send(MonitorNetEvent::ClientError(format!(
            "read CONNECTION_VALIDATION: {err}"
        )));
        return;
    }

    if let Err(err) = stream.write_all(&build_connection_validated()).await {
        let _ = tx.send(MonitorNetEvent::ClientError(format!(
            "write CONNECTION_VALIDATED: {err}"
        )));
        return;
    }

    let cid: u32 = 1;
    let ioid: u32 = 1;
    if let Err(err) = stream.write_all(&build_create_channel(cid, &pv_name)).await {
        let _ = tx.send(MonitorNetEvent::ClientError(format!(
            "write CREATE_CHANNEL: {err}"
        )));
        return;
    }

    let cc = match timeout(op_timeout, expect_frame(&mut stream, CMD_CREATE_CHANNEL)).await {
        Ok(Ok(payload)) => payload,
        Ok(Err(err)) => {
            let _ = tx.send(MonitorNetEvent::ClientError(format!(
                "read CREATE_CHANNEL response: {err}"
            )));
            return;
        }
        Err(_) => {
            let _ = tx.send(MonitorNetEvent::ClientError(format!(
                "timeout waiting for CREATE_CHANNEL response for '{}'",
                pv_name
            )));
            return;
        }
    };

    let mut cur = cc.as_slice();
    let _rcid = match read_u32_le(&mut cur) {
        Some(v) => v,
        None => {
            let _ = tx.send(MonitorNetEvent::ClientError(
                "CREATE_CHANNEL response truncated (cid)".to_string(),
            ));
            return;
        }
    };
    let sid = match read_u32_le(&mut cur) {
        Some(v) => v,
        None => {
            let _ = tx.send(MonitorNetEvent::ClientError(
                "CREATE_CHANNEL response truncated (sid)".to_string(),
            ));
            return;
        }
    };
    if !decode_status(&mut cur) {
        let _ = tx.send(MonitorNetEvent::RemoteError(format!(
            "server rejected CREATE_CHANNEL for '{}'",
            pv_name
        )));
        return;
    }

    if let Err(err) = stream.write_all(&build_monitor_init(sid, ioid)).await {
        let _ = tx.send(MonitorNetEvent::ClientError(format!(
            "write MONITOR INIT: {err}"
        )));
        return;
    }

    let mi = match timeout(op_timeout, expect_frame(&mut stream, CMD_MONITOR)).await {
        Ok(Ok(payload)) => payload,
        Ok(Err(err)) => {
            let _ = tx.send(MonitorNetEvent::ClientError(format!(
                "read MONITOR INIT response: {err}"
            )));
            return;
        }
        Err(_) => {
            let _ = tx.send(MonitorNetEvent::ClientError(
                "timeout waiting for MONITOR INIT response".to_string(),
            ));
            return;
        }
    };

    let mut cur = mi.as_slice();
    let _rioid = match read_u32_le(&mut cur) {
        Some(v) => v,
        None => {
            let _ = tx.send(MonitorNetEvent::ClientError(
                "MONITOR INIT response: missing ioid".to_string(),
            ));
            return;
        }
    };
    let _subcmd = take_byte(&mut cur);
    if !decode_status(&mut cur) {
        let _ = tx.send(MonitorNetEvent::RemoteError(format!(
            "MONITOR INIT failed for '{}'",
            pv_name
        )));
        return;
    }
    let desc = match decode_field_desc_cached(&mut cur) {
        Some(desc) => desc,
        None => {
            let _ = tx.send(MonitorNetEvent::ClientError(format!(
                "could not parse FieldDesc from MONITOR INIT for '{}'",
                pv_name
            )));
            return;
        }
    };

    if let Err(err) = stream.write_all(&build_monitor_start(sid, ioid)).await {
        let _ = tx.send(MonitorNetEvent::ClientError(format!(
            "write MONITOR START: {err}"
        )));
        return;
    }
    let _ = tx.send(MonitorNetEvent::Connected);

    loop {
        tokio::select! {
            _ = &mut stop_rx => {
                let _ = stream.write_all(&build_monitor_stop(sid, ioid)).await;
                let _ = stream.write_all(&build_destroy_request(sid, ioid)).await;
                let _ = stream.write_all(&build_destroy_channel(cid, sid)).await;
                let _ = tx.send(MonitorNetEvent::Finished);
                break;
            }
            read = read_frame(&mut stream) => {
                let (_, cmd, payload) = match read {
                    Ok(frame) => frame,
                    Err(err) => {
                        let _ = tx.send(MonitorNetEvent::Disconnected(format!("monitor stream closed: {err}")));
                        break;
                    }
                };

                if cmd != CMD_MONITOR {
                    continue;
                }

                let mut cur = payload.as_slice();
                let _rioid = match read_u32_le(&mut cur) {
                    Some(v) => v,
                    None => {
                        let _ = tx.send(MonitorNetEvent::ClientError("MONITOR frame: missing ioid".to_string()));
                        continue;
                    }
                };
                let _subcmd = take_byte(&mut cur);
                if !decode_status(&mut cur) {
                    let _ = tx.send(MonitorNetEvent::RemoteError(format!("MONITOR data failed for '{}'", pv_name)));
                    continue;
                }

                let bits = match read_bitset(&mut cur) {
                    Some(bits) => bits.to_vec(),
                    None => {
                        let _ = tx.send(MonitorNetEvent::ClientError("MONITOR data: could not read BitSet".to_string()));
                        continue;
                    }
                };

                let mut value = Value::new();
                let mut bit_counter: usize = 0;
                if decode_into_value(&mut cur, &desc, &bits, &mut bit_counter, "", &mut value).is_none() {
                    let _ = tx.send(MonitorNetEvent::ClientError(format!(
                        "failed to decode monitor payload for '{}'",
                        pv_name
                    )));
                    continue;
                }
                let _ = tx.send(MonitorNetEvent::Value(value));
            }
        }
    }
}

pub fn start_monitor(
    config: ClientConfig,
    rt: tokio::runtime::Handle,
    pv_name: String,
    timeout_secs: f64,
) -> Result<(MonitorSession, mpsc::UnboundedReceiver<MonitorNetEvent>)> {
    let (stop_tx, stop_rx) = oneshot::channel();
    let (tx, rx) = mpsc::unbounded_channel();
    rt.spawn(pva_monitor_task(config, pv_name, timeout_secs, stop_rx, tx));
    Ok((
        MonitorSession {
            stop_tx: Some(stop_tx),
        },
        rx,
    ))
}

async fn pva_put_inner(
    server: SocketAddr,
    pv_name: &str,
    put_value: PutValue<'_>,
    op_timeout: Duration,
) -> Result<()> {
    let mut stream = timeout(Duration::from_secs(5), TcpStream::connect(server))
        .await
        .map_err(|_| PvxsError::new("TCP connect timeout"))?
        .map_err(|e| PvxsError::new(format!("TCP connect: {e}")))?;

    let _cv = timeout(
        op_timeout,
        expect_frame(&mut stream, CMD_CONNECTION_VALIDATION),
    )
    .await
    .map_err(|_| PvxsError::new("timeout waiting for CONNECTION_VALIDATION"))?
    .map_err(|e| PvxsError::new(format!("{e}")))?;

    stream
        .write_all(&build_connection_validated())
        .await
        .map_err(|e| PvxsError::new(format!("{e}")))?;

    let cid: u32 = 1;
    let ioid: u32 = 1;
    stream
        .write_all(&build_create_channel(cid, pv_name))
        .await
        .map_err(|e| PvxsError::new(format!("{e}")))?;

    let cc = timeout(op_timeout, expect_frame(&mut stream, CMD_CREATE_CHANNEL))
        .await
        .map_err(|_| PvxsError::new(format!("timeout CREATE_CHANNEL for '{pv_name}'")))?
        .map_err(|e| PvxsError::new(format!("{e}")))?;

    let mut cur = cc.as_slice();
    let _rcid = read_u32_le(&mut cur).ok_or_else(|| PvxsError::new("CREATE_CHANNEL truncated"))?;
    let sid = read_u32_le(&mut cur).ok_or_else(|| PvxsError::new("CREATE_CHANNEL: no sid"))?;
    if !decode_status(&mut cur) {
        return Err(PvxsError::new(format!(
            "server rejected CREATE_CHANNEL for '{pv_name}'"
        )));
    }

    // PUT INIT
    stream
        .write_all(&build_put_init(sid, ioid))
        .await
        .map_err(|e| PvxsError::new(format!("{e}")))?;

    let pi = timeout(op_timeout, expect_frame(&mut stream, CMD_PUT))
        .await
        .map_err(|_| PvxsError::new("timeout PUT INIT response"))?
        .map_err(|e| PvxsError::new(format!("{e}")))?;

    let mut cur = pi.as_slice();
    let _rioid = read_u32_le(&mut cur).ok_or_else(|| PvxsError::new("PUT INIT: no ioid"))?;
    let _subcmd = take_byte(&mut cur);
    if !decode_status(&mut cur) {
        return Err(PvxsError::new(format!("PUT INIT failed for '{pv_name}'")));
    }
    // Skip FieldDesc from PUT INIT response (we don't need it for simple scalar put)
    // Decode just enough to discard the type descriptor
    decode_field_desc_cached(&mut cur);

    // PUT (subcmd = 0x00) — BitSet + value payload
    let value_bytes = encode_put_payload(&put_value);
    let mut p = Vec::new();
    p.extend_from_slice(&sid.to_le_bytes());
    p.extend_from_slice(&ioid.to_le_bytes());
    p.push(0x00); // subcmd = PUT
    p.extend_from_slice(&value_bytes);
    stream
        .write_all(&frame(false, CMD_PUT, p))
        .await
        .map_err(|e| PvxsError::new(format!("{e}")))?;

    let pd = timeout(op_timeout, expect_frame(&mut stream, CMD_PUT))
        .await
        .map_err(|_| PvxsError::new("timeout PUT response"))?
        .map_err(|e| PvxsError::new(format!("{e}")))?;

    let mut cur = pd.as_slice();
    let _rioid = read_u32_le(&mut cur).ok_or_else(|| PvxsError::new("PUT response: no ioid"))?;
    let _subcmd = take_byte(&mut cur);
    if !decode_status(&mut cur) {
        return Err(PvxsError::new(format!("PUT failed for '{pv_name}'")));
    }

    let _ = stream.write_all(&build_destroy_request(sid, ioid)).await;
    let _ = stream.write_all(&build_destroy_channel(cid, sid)).await;
    Ok(())
}

// ── Public synchronous API ────────────────────────────────────────────────

pub fn blocking_get(
    config: &ClientConfig,
    rt: &tokio::runtime::Handle,
    pv_name: &str,
    timeout_secs: f64,
) -> Result<Value> {
    let op_timeout = Duration::from_secs_f64(timeout_secs.clamp(0.1, 300.0));
    rt.block_on(async {
        let server = timeout(op_timeout, search(config, pv_name))
            .await
            .map_err(|_| {
                PvxsError::new(format!(
                    "search timeout: '{pv_name}' not found on the network"
                ))
            })??;
        pva_get_inner(server, pv_name, op_timeout).await
    })
}

pub fn blocking_put(
    config: &ClientConfig,
    rt: &tokio::runtime::Handle,
    pv_name: &str,
    put_value: PutValue<'_>,
    timeout_secs: f64,
) -> Result<()> {
    let op_timeout = Duration::from_secs_f64(timeout_secs.clamp(0.1, 300.0));
    rt.block_on(async {
        let server = timeout(op_timeout, search(config, pv_name))
            .await
            .map_err(|_| PvxsError::new(format!("search timeout: '{pv_name}' not found")))??;
        pva_put_inner(server, pv_name, put_value, op_timeout).await
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_put_payload_double_array_contains_count_and_values() {
        let payload = encode_put_payload(&PutValue::DoubleArray(vec![1.0, 2.5]));
        let mut cur = payload.as_slice();

        assert_eq!(decode_size(&mut cur).unwrap(), 1);
        assert_eq!(cur[0], 0b0000_0010);
        cur = &cur[1..];

        let count = read_u32_le(&mut cur).unwrap();
        assert_eq!(count, 2);
        assert_eq!(read_f64_le(&mut cur).unwrap(), 1.0);
        assert_eq!(read_f64_le(&mut cur).unwrap(), 2.5);
    }
}
