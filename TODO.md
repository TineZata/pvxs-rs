# pvxs-rs — TODO

Tracking remaining work to reach parity with `pvxs-sys` (same public API surface,
zero C++/DLL/EPICS_BASE requirement).

---

## Blocking: crate does not compile

- [x] **Create `src/lib.rs`** — done; exports `PvxsError`, `Result`, and all
      public API items so consumers can write `use pvxs::Context;`

---

## Missing source files

- [x] **`src/lib.rs`** — created
- [x] **`src/server.rs`** — created; full in-memory implementation of `Server`,
      `ServerHandle`, `NTScalarMetadataBuilder`, `NTEnumMetadataBuilder`, and all
      `Fetched*` types; alarm computation wired into `post_*` and `create_pv_*`;
      9 unit tests covering create / post / fetch / remove / alarms / handles

---

## epics-rs crate usage

### `epics-libcom-rs` — core dependency (added to `Cargo.toml`)

`epics-libcom-rs` is declared as an unconditional `[dependency]` via git in
`Cargo.toml`. Currently used:

| call site | symbol used |
|---|---|
| `src/client/monitor.rs` `get_update` | `runtime::task::sleep_until`, `runtime::select!`, `runtime::task::Instant` |

Still aspirational (not yet wired):

| libcom module | pvxs-rs use site |
|---|---|
| `runtime::task::spawn` | drive the pvAccess TCP client/server loops |
| `runtime::task::sleep` | replace the busy-sleep in `Monitor::get_update()` |
| `runtime::task::interval` | periodic beacon sender in `Server` |
| `net::async_udp_v4` | pvAccess search broadcaster (client) + search responder (server) |
| `net::iface_map::IfaceMap` | enumerate NICs when `EPICS_PVA_AUTO_ADDR_LIST=YES` |
| `net::loopback_mcast::bind_loopback_mcast` | loopback integration tests |
| `net::ORIGIN_TAG_MCAST_GROUP` | pvAccess wire constant embedded in search datagrams |

The planned roles for epics-libcom-rs are:

runtime task execution:

drive the pvAccess TCP client/server loops
replace the simple sleep in monitor polling
provide periodic beacon scheduling
networking helpers:

UDP broadcast/search handling
interface enumeration for auto-address-list logic
loopback multicast support for integration tests

Pending tasks:
- [x] Replace the busy-sleep loop in `Monitor::get_update()` with
      `epics_libcom_rs::runtime::task::sleep_until` — done: `get_update` now
      uses `block_in_place` / `Handle::block_on` to drive an async future;
      `tokio::sync::Notify` wakes the waiter immediately on publish so
      latency is no longer bounded by a 10 ms poll interval.
- [x] Wire `Server` beacon loop through `epics_libcom_rs::runtime::task::interval`
      instead of a raw `tokio::time::interval` call — done: `Server::start_*`
      now spawns a dedicated beacon scheduler thread using a current-thread
      tokio runtime + `task::interval`, with stop signaling integrated into
      `stop_drop()` and `Drop`
- [ ] Resolve the git dep to a semver release once `epics-libcom-rs` is
      published to crates.io and remove the `git = "…"` override

### `epics-ca-rs` — NO, not applicable

`epics-ca-rs` implements **Channel Access** (CA) — a completely separate EPICS
network protocol (ports 5064/5065, big-endian 16-byte header, DBR types).
pvxs-rs implements **pvAccess** (PVA) — different wire format, different ports
(5075/5076), different type system (PVField/FieldDesc). The two protocols are not
interchangeable and share no on-wire encoding.

---

## pvAccess network transport (largest missing piece)

The client now has a working phase-1 pvAccess GET/PUT transport stack implemented in `src/net.rs` and wired through the client context; monitor and server-network work remain outstanding.

### Dependency decision (researched and revised 2026-07-25)

`epics-pva-rs` has a complete, correct pvAccess codec in `pub` modules
(`codec.rs` ~700 LOC, `decode.rs` ~2100 LOC, `pv_request.rs` ~2764 LOC,
all behind `default-features = false`).  Its `epics-base-rs` transitive dep is
pure Rust — zero C++, zero DLLs.

However: **`epics-pva-rs` is a full Rust reimplementation of PVXS (≡ pvDataCPP +
pvAccessCPP)**, and `epics-base-rs` is the entire IOC core — record system,
database, iocsh, calc engine.  That is a very large, rapidly-evolving workspace we
do not control, for a use-case that only needs ~1 kLOC of wire-format codec.
pvxs-rs should own its protocol stack, not wrap another implementation.

> PVXS is functionally equivalent to the pvDataCPP and pvAccessCPP modules,
> and is foreseen to eventually supplant those (EPICS 7.1+).
> — docs.epics-controls.org/en/latest/pv-access/overview.html

**Decision: implement the minimal pvAccess codec from scratch inside pvxs-rs**,
using the wire-format reference below (verified from the pvAccess spec and the
epics-pva-rs source as a read-only reference).  Only deps needed: `tokio` (already
transitive through `epics-libcom-rs`) for async I/O.

### Wire format reference (verified from epics-pva-rs source)

**Header** — 8 bytes, always little-endian in this crate:
```
byte 0   : 0xCA  (magic)
byte 1   : 0x02  (PVA_VERSION)
byte 2   : flags — bit7=0→LE, bit6=0→client / bit6=1→server, bit0=0→application
byte 3   : command code
bytes 4-7: payload_length : u32 LE
```

**Command codes (application layer)**:
```
0x00 BEACON              0x01 CONNECTION_VALIDATION (server→client)
0x03 SEARCH              0x04 SEARCH_RESPONSE
0x07 CREATE_CHANNEL      0x09 CONNECTION_VALIDATED  (client→server)
0x0A GET                 0x0B PUT
0x0D MONITOR             0x10 DESTROY_REQUEST
0x11 DESTROY_CHANNEL     0x12 GET_FIELD
0x14 RPC                 0x16 ORIGIN_TAG
```
(Responses reuse the same code as requests but with flags bit6=1.)

**PvaSize encoding**:
```
0x00–0xFE : 1 byte (value 0–254)
0xFF      : null / absent
```
(Sizes > 254 not needed for typical PV names; extend later if required.)

**String** = PvaSize(len) + len UTF-8 bytes

**Status**:
```
0xFF      = OK (no message, short form)
0x00 + string(msg) + string(stack) = OK with message
0x01 + ...  = WARNING   0x02 + ... = ERROR   0x03 + ... = FATAL
```

**Type codes (pvData FieldDesc)**:
```
0x00 bool   0x20 int8   0x21 int16  0x22 int32  0x23 int64
0x24 uint8  0x25 uint16 0x26 uint32 0x27 uint64
0x42 float32  0x43 float64  0x60 string
Array = scalar_code | 0x08  (e.g. double[] = 0x4B)
0x80 structure   0x81 union   0x82 any (variant union)
0x88 structure[] 0xFF null
```

**Type-cache markers** (prefix wrapping a FieldDesc in server responses):
```
0xFD + PvaSize(slot) + inline_type_desc  = "define type at cache slot N"
0xFE + PvaSize(slot)                     = "reuse type from cache slot N"
```
For a fresh single-use TCP connection the server always sends 0xFD (first define).

**FieldDesc structure encoding**:
```
0x80                    ← struct tag
PvaSize(len)+bytes      ← type_id (e.g. "epics:nt/NTScalar:1.0")
PvaSize(field_count)
  for each field:
    PvaSize(name_len)+name_bytes
    <recursive FieldDesc>
```

**BitSet** (marks which fields are present in a GET/PUT/Monitor payload):
```
PvaSize(byte_count) + byte_count bytes
bit i (LSB-first) = field at DFS position i is present
```

**pvRequest bytes for "get all fields"** (field()):
```
0x80 0x00 0x01 0x05 'f''i''e''l''d' 0x80 0x00 0x00
 │    │    │    └── "field"          │    │    └── 0 subfields = select all
 │    │    └── 1 outer field         │    └── no type_id
 │    └── no type_id                 └── sub-structure tag
 └── structure tag
```
No value bytes follow (empty sub-structures carry no value payload).

**GET subcmd flags**: 0x08 = INIT, 0x00 = GET, 0x10 = DESTROY

### Files to create (phase 1 — GET working)

| File | Purpose | Est. LOC |
|---|---|---|
| `src/proto.rs` | Header encode/decode, PvaSize, string, status primitives | ~180 |
| `src/pvdata.rs` | `FieldDesc` decode, BitSet, recursive value decode → `Value` | ~400 |
| `src/net.rs` | UDP search + TCP framing + full pvAccess GET session | ~450 |

Then update `src/client.rs` (~60 lines) and `src/lib.rs` (add `mod` declarations).

Wire-format reference (verified from pvAccess spec + epics-pva-rs source) is in the
section below — no runtime dep on epics-pva-rs needed.

### TCP session sequence for `Context::get()`

```
1. UDP: bind random port → broadcast SEARCH to 255.255.255.255:5076
         (or each addr in EPICS_PVA_ADDR_LIST)
         payload: seq_id(u32) flags(0x00) reserved[3] addr[16]=0 port(u16)
                  proto_count=0  channel_count=1  cid(u32) pv_name(string)

2. UDP recv: SEARCH_RESPONSE (cmd 0x04, from server flags 0x40)
         extract server_addr (bytes 4..20) and server_tcp_port (bytes 20..22)
         extract found_cids list; confirm our cid is in it

3. TCP connect to server_addr:server_tcp_port

4. Recv: CONNECTION_VALIDATION (cmd 0x01, flags 0x40)
         read bufSize(u32) regSize(u16) authPlugins([string])

5. Send: CONNECTION_VALIDATED (cmd 0x09, flags 0x00)
         payload: bufSize(u32)=16M  regSize(u16)=0x10  qos(u16)=0
                  authPlugin(string)="anonymous"  authData=0xFF(null)

6. Recv: CONNECTION_VALIDATED echo (cmd 0x09, flags 0x40) — server confirms OK
         (skip any BEACON or other frames that arrive before this)

7. Send: CREATE_CHANNEL (cmd 0x07, flags 0x00)
         payload: count(u16)=1  cid(u32)  pv_name(string)

8. Recv: CREATE_CHANNEL response (cmd 0x07, flags 0x40)
         cid(u32)  sid(u32)  status

9. Send: GET INIT (cmd 0x0A, flags 0x00)
         payload: sid(u32)  ioid(u32)  subcmd=0x08  pv_request_bytes

10. Recv: GET INIT response (cmd 0x0A, flags 0x40)
          ioid(u32)  subcmd(u8)  status  0xFD+slot  FieldDesc

11. Send: GET (cmd 0x0A, flags 0x00)
          payload: sid(u32)  ioid(u32)  subcmd=0x00

12. Recv: GET data (cmd 0x0A, flags 0x40)
          ioid(u32)  subcmd(u8)  status  BitSet  value_bytes

13. Decode value_bytes using FieldDesc from step 10 → build Value

14. Send: DESTROY_REQUEST (cmd 0x10, flags 0x00)
          payload: sid(u32)  ioid(u32)
    Send: DESTROY_CHANNEL (cmd 0x11, flags 0x00)
          payload: cid(u32)  sid(u32)
15. Close TCP socket
```

### Phase 1 task list — client GET

- [x] **`src/proto.rs`**: `encode_header`, `decode_header`, `encode_size`,
      `decode_size`, `encode_string`, `decode_string`, `decode_status`
- [x] **`src/pvdata.rs`**: `FieldDesc` enum + `decode_field_desc()` (handles 0xFD
      inline case; 0xFE → error for now); `decode_value()` walks DFS tree against
      BitSet and populates `crate::Value`; `build_pv_request_all()` returns the
      9-byte pvRequest for `field()`
- [x] **`src/net.rs`**: async `search()` (UDP broadcast, parse SEARCH_RESPONSE);
      async `pva_get()` (TCP session steps 3–15 above); sync wrapper using
      `tokio::Runtime` stored in `Context`
- [x] **`src/client/context.rs`**: store `tokio::runtime::Runtime` in `Context`; replace
      the GET stub with `crate::net::blocking_get(&self._config, &self._rt, pv_name, timeout)`
- [x] **`src/lib.rs`**: add `mod proto;` `mod pvdata;` `mod net;`
- [x] **`Cargo.toml`**: promote tokio to a regular dep with
      `features = ["rt-multi-thread", "net", "io-util", "time"]`

### Phase 2 task list — client PUT

- [x] `net.rs`: `async pva_put()` — same handshake + CREATE_CHANNEL, then
      PUT INIT (subcmd=0x08 + pv_request) → PUT (subcmd=0x00 + BitSet + value bytes)
      → recv PUT response → DESTROY_REQUEST + DESTROY_CHANNEL
- [x] `client/context.rs`: wire `put_double`, `put_int32`, `put_string`, `put_enum`,
      and the `_array` variants through `blocking_put`

### Phase 3 task list — Monitor (subscription)

- [x] `net.rs`: long-lived TCP connection; MONITOR INIT → MONITOR START;
      spawn background task that reads MONITOR data frames and pushes into
      `MonitorInner` queue via `Arc<Mutex<...>>`; send MONITOR STOP + DESTROY on drop
      — done: `start_monitor()` now spawns an async monitor session task with
      MONITOR INIT/START handshake, frame decode loop, and STOP + DESTROY cleanup
      on session drop/stop
- [x] `client.rs`: `Monitor::start()` spawns the background task
      — done in `src/client/monitor.rs`: `start()` launches the monitor session
      and an event-dispatch task that updates queue/connection state/events

### Phase 4 task list — server network

- [x] UDP search responder using `epics_libcom_rs::net::async_udp_v4`
      — done: server now binds per-NIC UDP sockets via `AsyncUdpV4`, listens
      for SEARCH frames, and replies with SEARCH_RESPONSE on the receiving NIC;
      lifecycle is integrated with `stop_drop()`/`Drop`
- [x] Periodic beacon via `epics_libcom_rs::runtime::task::interval`
      — done: beacon thread now sends periodic UDP BEACON frames using
      `task::interval` and `AsyncUdpV4::fanout_to` (best-effort broadcast)
- [x] TCP listener (port 5075); dispatch Get/Put/Monitor to in-memory registry
      — done: server now accepts pvAccess TCP connections, handles
      CONNECTION_VALIDATION/CREATE_CHANNEL and dispatches GET/PUT/MONITOR
      requests against the manager-backed in-memory PV registry
- [x] `tcp_port()` / `udp_port()` return actual bound port numbers
      — done: startup threads bind sockets/listeners first, report real bound
      ports back to `ServerHandle`, and lifecycle is wired through `stop_drop`/`Drop`

### Transport — implementation notes

- `epics-pva-rs` used as a **read-only reference** for the wire format only —
  not added as a runtime dependency
- `src/proto.rs` and `src/pvdata.rs` own the codec; no external codec dep
- Use `tokio::net::UdpSocket` for UDP search (set `SO_BROADCAST`)
- Use `tokio::net::TcpStream` with `AsyncReadExt::read_exact` for TCP framing
- One `tokio::runtime::Runtime` stored in `Context` (created at `Context::from_env()`)
  to avoid per-call runtime overhead
- Skip type cache for phase 1 (always expect 0xFD inline; 0xFE → error with hint)
- Anonymous auth only for phase 1; TLS in a later phase (no heavy dep required)

---

## API surface gaps (parity with `pvxs-sys`)

- [x] **`src/server.rs`** — full public surface implemented:
  `Server::start_from_env / start_isolated / handle / tcp_port / udp_port /
  create_pv_* / post_* / fetch_* / remove_pv / stop_drop`;  `ServerHandle`
  (clone-able, thread-safe); `NTScalarMetadataBuilder / NTEnumMetadataBuilder`;
  `FetchedDouble / Int32 / String / Enum / DoubleArray / Int32Array / StringArray`
- [x] **`Value` introspection** — `field_names() -> Vec<String>`,
      `type_of(field) -> Option<FieldType>`, `FieldType` enum
- [x] **NTScalar / NTEnum / NTScalarArray builders** — convenience constructors
      that pre-populate the standard normative-type field layout (`value`,
      `alarm.severity`, `alarm.status`, `alarm.message`, `timeStamp.*`, etc.)
- [x] **`Value` display / timestamp fields** — `set_display_limit_low/high`,
      `set_display_units/precision/description`;
      `set_timestamp_seconds/nanos` and matching getters
- [x] **`Value::set_field_bool` / `get_field_bool`** — added; maps to `FieldValue::Bool`
- [x] **`Value::set_field_int64` / `get_field_int64`** — added for timestamp fields
- [x] **`AlarmMetadata` pub setters** — expose builder-style API (all fields are
      already `pub`; a builder wrapper can be added if needed)

---

## Testing

- [x] Unit tests for `Value` field set/get round-trips (all scalar types) — 8 tests
- [x] Unit tests for `compute_alarm_for_scalar` boundary conditions — 6 tests
- [x] Unit tests for `AlarmSeverity`/`AlarmStatus` `From<i32>` conversions — 4 tests
- [x] Server integration tests (in-process, no network) — 10 tests
- [x] Monitor in-process queue/FIFO tests — 2 tests
- [x] Monitor `get_update` wake-on-publish test — 1 test
- [x] Net encode/decode tests — 1 test; all 32 unit tests pass
- [x] Fix failing doc-test in `src/value.rs` line 14 (`cargo test` exits non-zero)
- [x] Integration test: in-process server + client via loopback (requires network
      transport layer) — done: added `tests/client_server_loopback.rs` covering
      GET/PUT/MONITOR round-trips, missing-PV errors, type-mismatch PUT errors,
      and post-stop request behavior; plus `tests/parity_remote_suite.rs`
      for pvxs-sys parity scenarios (string encoding, double precision/specials,
      large/special arrays, enum transition/bounds, and multi-client consistency)
- [ ] `cargo test` must pass with zero warnings before 0.1 release

---

## Documentation & project hygiene

- [ ] Fill `Cargo.toml` `repository`, `keywords`, `categories` fields
- [ ] Add `README.md` migration guide: how to swap `pvxs-sys` for `pvxs-rs`
      (change `Cargo.toml` dep, remove `EPICS_BASE` env var, remove C++ toolchain)
- [ ] `#![deny(missing_docs)]` once public API is stable
- [ ] CI: add a GitHub Actions workflow that runs `cargo check`, `cargo test`,
      and `cargo clippy -- -D warnings` on Windows, Linux, macOS
- [ ] Publish checklist: verify no `pvxs-sys`, `cxx`, or C native libs in the
      dependency tree (all deps must be pure Rust)

---

## Optional / future

- [ ] **Async public API** (feature-gated `async`): surface `AsyncContext::get()`,
      `AsyncContext::monitor()` returning `futures::Stream`; the transport layer
      already uses `epics-libcom-rs` async internally, this just exposes it
- [ ] **TLS** (feature-gated `tls`): pvAccess over TLS (PVA-TLS spec)
- [ ] **Semver pin**: once `epics-libcom-rs` is published to crates.io, replace
      `git = "https://github.com/epics-rs/epics-rs"` in `Cargo.toml` with a
      `version = "x.y"` specifier
