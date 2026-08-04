// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{Context, MonitorEvent, NTScalarMetadataBuilder, Server};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

static EVENT_COUNT: AtomicUsize = AtomicUsize::new(0);

extern "C" fn test_event_callback() {
    EVENT_COUNT.fetch_add(1, Ordering::SeqCst);
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_env() -> MutexGuard<'static, ()> {
    match env_lock().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn setup_client_for(server: &Server) -> Context {
    std::env::set_var("EPICS_PVA_ADDR_LIST", format!("127.0.0.1:{}", server.udp_port()));
    std::env::set_var("EPICS_PVA_AUTO_ADDR_LIST", "NO");
    std::env::set_var("EPICS_PVA_BROADCAST_PORT", server.udp_port().to_string());
    Context::from_env().expect("context from env")
}

#[test]
fn test_monitor_connection_and_disconnection_events_off() {
    let _guard = lock_env();
    let pv = "callback:test:stop";
    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double(pv, 2.71, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let mut mon = ctx
        .monitor_builder(pv)
        .expect("builder")
        .connect_exception(false)
        .disconnect_exception(false)
        .exec()
        .expect("exec");
    mon.start().expect("start");
    thread::sleep(Duration::from_millis(300));

    let mut got_connected = false;
    let mut got_disconnected = false;
    let mut data_count = 0;
    for _ in 0..20 {
        match mon.pop() {
            Ok(Some(_)) => data_count += 1,
            Ok(None) => break,
            Err(MonitorEvent::Connected(_)) => got_connected = true,
            Err(MonitorEvent::Disconnected(_)) => got_disconnected = true,
            Err(_) => {}
        }
    }

    assert!(!got_connected);
    assert!(!got_disconnected);
    assert!(data_count > 0, "expected at least initial value data");

    mon.stop().expect("stop");
    server.stop_drop().expect("stop server");
}

#[test]
fn test_monitor_connection_on_and_disconnection_off() {
    let _guard = lock_env();
    let pv = "callback:test:stop";
    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double(pv, 2.71, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let mut mon = ctx
        .monitor_builder(pv)
        .expect("builder")
        .connect_exception(true)
        .disconnect_exception(false)
        .exec()
        .expect("exec");
    mon.start().expect("start");
    thread::sleep(Duration::from_millis(300));

    let mut got_connected = false;
    for _ in 0..20 {
        match mon.pop() {
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(MonitorEvent::Connected(_)) => {
                got_connected = true;
                break;
            }
            Err(_) => {}
        }
    }

    assert!(got_connected, "expected connection event when enabled");

    mon.stop().expect("stop");
    server.stop_drop().expect("stop server");
}

#[test]
fn test_monitor_connection_off_disconnection_on() {
    let _guard = lock_env();
    let pv = "callback:test:stop";
    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double(pv, 2.71, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let mut mon = ctx
        .monitor_builder(pv)
        .expect("builder")
        .connect_exception(false)
        .disconnect_exception(true)
        .exec()
        .expect("exec");

    mon.start().expect("start");
    thread::sleep(Duration::from_millis(300));

    server.stop_drop().expect("stop server");
    thread::sleep(Duration::from_millis(400));

    let mut got_disconnected = false;
    for _ in 0..40 {
        match mon.pop() {
            Ok(Some(_)) => {}
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(MonitorEvent::Disconnected(_)) | Err(MonitorEvent::Finished(_)) => {
                got_disconnected = true;
                break;
            }
            Err(_) => {}
        }
    }
    assert!(
        got_disconnected,
        "expected disconnected or finished event when server stops"
    );
}

#[test]
fn test_monitor_multiple_client_monitors() {
    let _guard = lock_env();
    let pv = "callback:test:mask";
    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double(pv, 1.23, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let mut mon_masked = ctx
        .monitor_builder(pv)
        .expect("builder")
        .connect_exception(false)
        .disconnect_exception(false)
        .exec()
        .expect("exec");

    let mut mon_enabled = ctx
        .monitor_builder(pv)
        .expect("builder")
        .connect_exception(true)
        .disconnect_exception(false)
        .exec()
        .expect("exec");

    mon_masked.start().expect("start masked");
    mon_enabled.start().expect("start enabled");
    thread::sleep(Duration::from_millis(300));

    let mut masked_connected = false;
    let mut enabled_connected = false;
    for _ in 0..20 {
        if let Err(MonitorEvent::Connected(_)) = mon_masked.pop() {
            masked_connected = true;
        }
        if let Err(MonitorEvent::Connected(_)) = mon_enabled.pop() {
            enabled_connected = true;
        }
    }

    assert!(!masked_connected);
    assert!(enabled_connected);

    mon_masked.stop().expect("stop masked");
    mon_enabled.stop().expect("stop enabled");
    server.stop_drop().expect("stop server");
}

#[test]
fn test_monitor_event_callback_registration() {
    let _guard = lock_env();
    let pv = "callback:test:event";
    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double(pv, 0.0, NTScalarMetadataBuilder::new())
        .expect("create pv");

    EVENT_COUNT.store(0, Ordering::SeqCst);

    let mut ctx = setup_client_for(&server);
    let mut mon = ctx
        .monitor_builder(pv)
        .expect("builder")
        .connect_exception(true)
        .disconnect_exception(false)
        .event(test_event_callback)
        .exec()
        .expect("exec");

    mon.start().expect("start");
    thread::sleep(Duration::from_millis(350));

    let start_count = EVENT_COUNT.load(Ordering::SeqCst);
    assert!(start_count > 0, "expected callback on connect event");

    // Keep a smoke PUT so this test also exercises post-start monitor activity.
    let _ = ctx.put_double(pv, 1.0, 1.0);

    mon.stop().expect("stop");
    server.stop_drop().expect("stop server");
}