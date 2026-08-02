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
fn test_pv_remote_double_array_get_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:double:array";
    let initial = vec![1.1, 2.2, 3.3, 4.4, 5.5];

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double_array(name, initial.clone(), NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    ctx.put_double_array(name, initial.clone(), timeout)
        .expect("put initial array");
    let got = ctx
        .get(name, timeout)
        .expect("get initial array")
        .get_field_double_array("value")
        .expect("value");
    assert_eq!(got, initial);

    let large: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    ctx.put_double_array(name, large.clone(), timeout)
        .expect("put large array");
    let got = ctx
        .get(name, timeout)
        .expect("get large array")
        .get_field_double_array("value")
        .expect("value");
    assert_eq!(got.len(), large.len());

    ctx.put_double_array(name, vec![], timeout)
        .expect("put empty array");
    let got = ctx
        .get(name, timeout)
        .expect("get empty array")
        .get_field_double_array("value")
        .expect("value");
    assert!(got.is_empty());

    server.stop_drop().expect("stop server");
}

#[test]
fn test_pv_remote_double_array_special_values() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:double:array:special";
    let special = vec![
        0.0,
        -0.0,
        f64::MIN,
        f64::MAX,
        f64::MIN_POSITIVE,
        1e-308,
        1e308,
        std::f64::consts::PI,
        std::f64::consts::E,
    ];

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double_array(name, vec![0.0], NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    ctx.put_double_array(name, special.clone(), timeout)
        .expect("put special array");
    let got = ctx
        .get(name, timeout)
        .expect("get special array")
        .get_field_double_array("value")
        .expect("value");

    for (expected, actual) in special.iter().zip(got.iter()) {
        if expected.is_finite() {
            assert_eq!(expected, actual);
        }
    }

    server.stop_drop().expect("stop server");
}