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
fn test_pv_remote_int32_array_get_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:int32:array";
    let initial = vec![10, 20, 30, 40, 50];

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_int32_array(name, initial.clone(), NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    ctx.put_int32_array(name, initial.clone(), timeout)
        .expect("put initial array");
    let got = ctx
        .get(name, timeout)
        .expect("get initial array")
        .get_field_int32_array("value")
        .expect("value");
    assert_eq!(got, initial);

    let negative = vec![-100, -50, 0, 50, 100];
    ctx.put_int32_array(name, negative.clone(), timeout)
        .expect("put negative array");
    let got = ctx
        .get(name, timeout)
        .expect("get negative array")
        .get_field_int32_array("value")
        .expect("value");
    assert_eq!(got, negative);

    let large: Vec<i32> = (0..200).collect();
    ctx.put_int32_array(name, large.clone(), timeout)
        .expect("put large array");
    let got = ctx
        .get(name, timeout)
        .expect("get large array")
        .get_field_int32_array("value")
        .expect("value");
    assert_eq!(got.len(), large.len());

    ctx.put_int32_array(name, vec![], timeout)
        .expect("put empty array");
    let got = ctx
        .get(name, timeout)
        .expect("get empty array")
        .get_field_int32_array("value")
        .expect("value");
    assert!(got.is_empty());

    server.stop_drop().expect("stop server");
}

#[test]
fn test_pv_remote_int32_array_boundary() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:int32:array:boundary";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_int32_array(name, vec![0], NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let boundary = vec![i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX - 1, i32::MAX];
    ctx.put_int32_array(name, boundary.clone(), timeout)
        .expect("put boundary array");
    let got = ctx
        .get(name, timeout)
        .expect("get boundary array")
        .get_field_int32_array("value")
        .expect("value");
    assert_eq!(got, boundary);

    let seq: Vec<i32> = (1..=1000).collect();
    ctx.put_int32_array(name, seq.clone(), timeout)
        .expect("put sequence array");
    let got = ctx
        .get(name, timeout)
        .expect("get sequence array")
        .get_field_int32_array("value")
        .expect("value");
    assert_eq!(got, seq);

    server.stop_drop().expect("stop server");
}