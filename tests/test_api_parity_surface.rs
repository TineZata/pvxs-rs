// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{
    configure_logging_from_env, set_logger_level, Context, NTEnumMetadataBuilder,
    NTScalarMetadataBuilder, Server, SharedPV, StaticSource,
};
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
fn logging_apis_are_callable() {
    configure_logging_from_env().expect("configure logging");
    set_logger_level("pvxs.tcp.setup", "CRIT").expect("set logger level");
}

#[test]
fn context_info_returns_value_payload() {
    let _guard = lock_env();

    let server = Server::start_isolated().expect("start server");
    server
        .create_pv_double("parity:info", 12.5, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let mut ctx = setup_client_for(&server);
    let info = ctx.info("parity:info", 2.0).expect("info");
    assert_eq!(info.get_field_double("value").expect("value"), 12.5);

    server.stop_drop().expect("stop server");
}

#[test]
fn sharedpv_and_staticsource_surface_is_usable() {
    let mut pv = SharedPV::create_mailbox().expect("create mailbox");
    pv.open_double(1.5, NTScalarMetadataBuilder::new())
        .expect("open double");
    pv.post(pvxs::Value::nt_scalar_double(2.5)).expect("post value");

    let mut enum_pv = SharedPV::create_readonly().expect("create readonly");
    enum_pv
        .open_enum(
            vec!["OFF", "ON"],
            0,
            NTEnumMetadataBuilder::new(),
        )
        .expect("open enum");

    let mut src = StaticSource::new();
    src.add("parity:pv", pv).expect("add pv");
    src.add("parity:enum", enum_pv).expect("add enum pv");
}
