use pvxs::{Context, NTScalarMetadataBuilder, Server};
use std::sync::{Mutex, MutexGuard, OnceLock};

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
fn test_pv_remote_double_get_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let initial_value = 3.14159;
    let name = "remote:double";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let first = ctx.get(name, timeout).expect("initial get");
    assert_eq!(first.get_field_double("value").expect("value"), initial_value);

    server.stop_drop().expect("stop server");
    assert!(ctx.get(name, 0.2).is_err());

    let server = Server::start_isolated().expect("restart server");
    server
        .create_pv_double(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("recreate pv");

    let mut ctx = setup_client_for(&server);
    let new_value = 2.71828;
    ctx.put_double(name, new_value, timeout).expect("put double");
    let second = ctx.get(name, timeout).expect("second get");
    assert_eq!(second.get_field_double("value").expect("value"), new_value);

    server.stop_drop().expect("stop server");
}

#[test]
fn test_pv_remote_double_precision() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:double:precision";
    let precise = 1.23456789012345_f64;

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double(name, precise, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let value = ctx.get(name, timeout).expect("get precise");
    let got = value.get_field_double("value").expect("value");
    assert!((got - precise).abs() < 1e-14);

    let small = 1e-15;
    ctx.put_double(name, small, timeout).expect("put small");
    let value = ctx.get(name, timeout).expect("get small");
    let got = value.get_field_double("value").expect("value");
    assert!((got - small).abs() < 1e-16);

    server.stop_drop().expect("stop server");
}