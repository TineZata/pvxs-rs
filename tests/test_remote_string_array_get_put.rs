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
fn test_pv_remote_string_array_get_put() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:string:array";
    let initial = vec![
        "First".to_string(),
        "Second".to_string(),
        "Third".to_string(),
        "Fourth".to_string(),
    ];

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_string_array(name, vec!["seed".to_string()], NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    ctx.put_string_array(name, initial.clone(), timeout)
        .expect("put initial array");
    let got = ctx
        .get(name, timeout)
        .expect("get initial array")
        .get_field_string_array("value")
        .expect("value");
    assert_eq!(got, initial);

    let special = vec![
        "Empty: ".to_string(),
        "Spaces and punctuation!@#$".to_string(),
        "Newlines\\nand\\ttabs".to_string(),
    ];
    ctx.put_string_array(name, special.clone(), timeout)
        .expect("put special array");
    let got = ctx
        .get(name, timeout)
        .expect("get special array")
        .get_field_string_array("value")
        .expect("value");
    assert_eq!(got, special);

    let empty = vec!["".to_string(), "non-empty".to_string(), "".to_string()];
    ctx.put_string_array(name, empty.clone(), timeout)
        .expect("put empty-string array");
    let got = ctx
        .get(name, timeout)
        .expect("get empty-string array")
        .get_field_string_array("value")
        .expect("value");
    assert_eq!(got, empty);

    ctx.put_string_array(name, vec![], timeout)
        .expect("put empty array");
    let got = ctx
        .get(name, timeout)
        .expect("get empty array")
        .get_field_string_array("value")
        .expect("value");
    assert!(got.is_empty());

    server.stop_drop().expect("stop server");
}

#[test]
fn test_pv_remote_string_array_large_strings() {
    let _guard = lock_env();
    let timeout = 2.0;
    let name = "remote:string:array:large";

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_string_array(name, vec!["seed".to_string()], NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let large = vec![
        "A".repeat(100),
        "B".repeat(1000),
        "Small".to_string(),
        "C".repeat(500),
    ];
    ctx.put_string_array(name, large.clone(), timeout)
        .expect("put large strings");
    let got = ctx
        .get(name, timeout)
        .expect("get large strings")
        .get_field_string_array("value")
        .expect("value");
    assert_eq!(got, large);

    let many: Vec<String> = (0..100).map(|i| format!("String_{:03}", i)).collect();
    ctx.put_string_array(name, many.clone(), timeout)
        .expect("put many strings");
    let got = ctx
        .get(name, timeout)
        .expect("get many strings")
        .get_field_string_array("value")
        .expect("value");
    assert_eq!(got, many);

    server.stop_drop().expect("stop server");
}