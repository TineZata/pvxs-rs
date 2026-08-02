use pvxs::{
    AlarmMetadata, AlarmSeverity, Context, ControlMetadata, NTScalarMetadataBuilder, Server,
};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Duration;

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

fn lifecycle_meta(i: i32) -> AlarmMetadata {
    AlarmMetadata::new()
        .active(true)
        .low_alarm_limit((i * 10) as f64)
        .low_warning_limit((i * 10 + 10) as f64)
        .high_warning_limit((i * 10 + 80) as f64)
        .high_alarm_limit((i * 10 + 90) as f64)
        .low_alarm_severity(AlarmSeverity::Major)
        .low_warning_severity(AlarmSeverity::Minor)
        .high_warning_severity(AlarmSeverity::Minor)
        .high_alarm_severity(AlarmSeverity::Major)
        .hysteresis(0)
}

#[test]
fn test_create_multiple_pvs_with_alarms() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    for i in 0..5 {
        let pv_name = format!("test:lifecycle:pv{}", i);
        let metadata = NTScalarMetadataBuilder::new().alarm_metadata(lifecycle_meta(i));
        server
            .create_pv_double(&pv_name, 50.0, metadata)
            .expect("create lifecycle pv");
    }

    let mut ctx = setup_client_for(&server);
    for i in 0..5 {
        let pv_name = format!("test:lifecycle:pv{}", i);
        let value = ctx.get(&pv_name, 2.0).expect("get lifecycle pv");
        assert_eq!(value.get_field_double("value").expect("value"), 50.0);
    }

    server.stop_drop().expect("stop server");
}

#[test]
fn test_remove_pv_with_alarms() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv_name = "test:lifecycle:remove";
    let metadata = NTScalarMetadataBuilder::new().alarm_metadata(lifecycle_meta(1));

    server
        .create_pv_double(pv_name, 50.0, metadata)
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    assert_eq!(
        ctx.get(pv_name, 2.0)
            .expect("get before remove")
            .get_field_double("value")
            .expect("value"),
        50.0
    );

    server.remove_pv(pv_name).expect("remove pv");
    thread::sleep(Duration::from_millis(80));
    assert!(ctx.get(pv_name, 1.0).is_err());

    server.stop_drop().expect("stop server");
}

#[test]
fn test_duplicate_pv_creation() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv_name = "test:lifecycle:duplicate";
    let metadata_first = NTScalarMetadataBuilder::new().alarm_metadata(lifecycle_meta(1));
    server
        .create_pv_double(pv_name, 50.0, metadata_first)
        .expect("create first pv");

    let metadata_second = NTScalarMetadataBuilder::new().alarm_metadata(lifecycle_meta(1));
    let result = server.create_pv_double(pv_name, 75.0, metadata_second);
    assert!(result.is_err());

    server.stop_drop().expect("stop server");
}

#[test]
fn test_post_to_nonexistent_pv() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let result = server.post_double("test:lifecycle:nonexistent", 42.0);
    assert!(result.is_err());
    server.stop_drop().expect("stop server");
}

#[test]
fn test_alarm_persistence_across_posts() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv = "test:lifecycle:persistence";
    let metadata = NTScalarMetadataBuilder::new().control(ControlMetadata {
        limit_low: 0.0,
        limit_high: 100.0,
        min_step: 0.1,
    });
    server.create_pv_double(pv, 50.0, metadata).expect("create pv");

    let mut ctx = setup_client_for(&server);
    server.post_double(pv, 75.0).expect("post valid");
    thread::sleep(Duration::from_millis(40));
    assert_eq!(
        ctx.get(pv, 2.0)
            .expect("get valid")
            .get_field_double("value")
            .expect("value"),
        75.0
    );

    assert!(server.post_double(pv, 150.0).is_err());
    thread::sleep(Duration::from_millis(40));
    assert_eq!(
        ctx.get(pv, 2.0)
            .expect("get rejected")
            .get_field_double("value")
            .expect("value"),
        75.0
    );

    server.post_double(pv, 25.0).expect("post valid again");
    thread::sleep(Duration::from_millis(40));
    assert_eq!(
        ctx.get(pv, 2.0)
            .expect("get valid again")
            .get_field_double("value")
            .expect("value"),
        25.0
    );

    server.stop_drop().expect("stop server");
}

#[test]
fn test_manager_handle_after_stop() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let handle = server.handle();
    let pv = "test:lifecycle:handle";
    handle
        .create_pv_double(pv, 42.0, NTScalarMetadataBuilder::new())
        .expect("create via handle");
    server.stop_drop().expect("stop server");

    let result = handle.post_double(pv, 100.0);
    assert!(result.is_err());
}

#[test]
fn test_mixed_pv_types_with_alarms() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");

    let metadata_double = NTScalarMetadataBuilder::new().alarm_metadata(lifecycle_meta(1));
    let metadata_int = NTScalarMetadataBuilder::new().control(ControlMetadata {
        limit_low: 0.0,
        limit_high: 255.0,
        min_step: 1.0,
    });

    server
        .create_pv_double("test:lifecycle:mixed:double", 50.0, metadata_double)
        .expect("create double pv");
    server
        .create_pv_int32("test:lifecycle:mixed:int32", 128, metadata_int)
        .expect("create int32 pv");
    server
        .create_pv_string(
            "test:lifecycle:mixed:string",
            "test",
            NTScalarMetadataBuilder::new(),
        )
        .expect("create string pv");

    let mut ctx = setup_client_for(&server);
    assert_eq!(
        ctx.get("test:lifecycle:mixed:double", 2.0)
            .expect("get double")
            .get_field_double("value")
            .expect("value"),
        50.0
    );
    assert_eq!(
        ctx.get("test:lifecycle:mixed:int32", 2.0)
            .expect("get int32")
            .get_field_int32("value")
            .expect("value"),
        128
    );
    assert_eq!(
        ctx.get("test:lifecycle:mixed:string", 2.0)
            .expect("get string")
            .get_field_string("value")
            .expect("value"),
        "test"
    );

    server.stop_drop().expect("stop server");
}