use pvxs::{
    AlarmSeverity, AlarmStatus, Context, ControlMetadata, NTScalarMetadataBuilder, Server,
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

#[test]
fn test_control_limits_reject_out_of_bounds() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv_name = "test:control:reject";
    let metadata = NTScalarMetadataBuilder::new().control(ControlMetadata {
        limit_low: 0.0,
        limit_high: 100.0,
        min_step: 0.1,
    });

    server
        .create_pv_double(pv_name, 50.0, metadata)
        .expect("create pv");
    let mut ctx = setup_client_for(&server);

    server.post_double(pv_name, 75.0).expect("post valid value");
    thread::sleep(Duration::from_millis(40));
    let value = ctx.get(pv_name, 2.0).expect("get");
    assert_eq!(value.get_field_double("value").expect("value"), 75.0);

    assert!(server.post_double(pv_name, 150.0).is_err());
    thread::sleep(Duration::from_millis(40));
    let value = ctx.get(pv_name, 2.0).expect("get high reject");
    assert_eq!(value.get_field_double("value").expect("value"), 75.0);
    if let Ok(severity) = value.get_field_int32("alarm.severity") {
        assert_eq!(severity, AlarmSeverity::Invalid as i32);
    }
    if let Ok(status) = value.get_field_int32("alarm.status") {
        assert_eq!(status, AlarmStatus::RecordStatus as i32);
    }

    assert!(server.post_double(pv_name, -10.0).is_err());
    thread::sleep(Duration::from_millis(40));
    let value = ctx.get(pv_name, 2.0).expect("get low reject");
    assert_eq!(value.get_field_double("value").expect("value"), 75.0);

    server.stop_drop().expect("stop server");
}

#[test]
fn test_control_limits_boundary_values() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv_name = "test:control:boundary";
    let metadata = NTScalarMetadataBuilder::new().control(ControlMetadata {
        limit_low: -50.0,
        limit_high: 50.0,
        min_step: 1.0,
    });
    server
        .create_pv_double(pv_name, 0.0, metadata)
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    server.post_double(pv_name, -50.0).expect("post lower boundary");
    thread::sleep(Duration::from_millis(40));
    assert_eq!(
        ctx.get(pv_name, 2.0)
            .expect("get lower boundary")
            .get_field_double("value")
            .expect("value"),
        -50.0
    );

    server.post_double(pv_name, 50.0).expect("post upper boundary");
    thread::sleep(Duration::from_millis(40));
    assert_eq!(
        ctx.get(pv_name, 2.0)
            .expect("get upper boundary")
            .get_field_double("value")
            .expect("value"),
        50.0
    );

    server.stop_drop().expect("stop server");
}

#[test]
fn test_control_limits_int32() {
    let _guard = lock_env();
    let server = Server::start_isolated().expect("start server");
    let pv_name = "test:control:int32";
    let metadata = NTScalarMetadataBuilder::new().control(ControlMetadata {
        limit_low: 0.0,
        limit_high: 255.0,
        min_step: 1.0,
    });

    server
        .create_pv_int32(pv_name, 128, metadata)
        .expect("create int32 pv");
    let mut ctx = setup_client_for(&server);

    server.post_int32(pv_name, 200).expect("post valid");
    thread::sleep(Duration::from_millis(40));
    assert_eq!(
        ctx.get(pv_name, 2.0)
            .expect("get valid")
            .get_field_int32("value")
            .expect("value"),
        200
    );

    assert!(server.post_int32(pv_name, 300).is_err());
    thread::sleep(Duration::from_millis(40));
    let value = ctx.get(pv_name, 2.0).expect("get invalid");
    assert_eq!(value.get_field_int32("value").expect("value"), 200);
    if let Ok(severity) = value.get_field_int32("alarm.severity") {
        assert_eq!(severity, AlarmSeverity::Invalid as i32);
    }

    server.stop_drop().expect("stop server");
}