use std::error::Error;

use pvxs::{
    Context, MonitorBuilder, MonitorEvent, NTEnumMetadataBuilder, NTScalarMetadataBuilder, Rpc,
    Server, Value,
};

fn assert_error<T: Error>() {}

#[allow(dead_code)]
fn context_surface(context: &mut Context) -> pvxs::Result<()> {
    let _ = context.get("test:pv", 1.0)?;
    let _ = context.info("test:pv", 1.0)?;
    context.put_double("test:pv", 1.0, 1.0)?;
    context.put_int32("test:pv", 1, 1.0)?;
    context.put_string("test:pv", "value", 1.0)?;
    context.put_enum("test:pv", 1, 1.0)?;
    context.put_double_array("test:pv", vec![1.0], 1.0)?;
    context.put_int32_array("test:pv", vec![1], 1.0)?;
    context.put_string_array("test:pv", vec!["value".to_string()], 1.0)?;
    let _ = context.monitor("test:pv")?;
    let _ = context.monitor_builder("test:pv")?;
    let _ = context.rpc("test:rpc")?;
    Ok(())
}

#[allow(dead_code)]
fn monitor_builder_surface(builder: MonitorBuilder) -> pvxs::Result<()> {
    extern "C" fn event() {}

    let _ = builder
        .connect_exception(true)
        .disconnect_exception(true)
        .event(event)
        .exec_with_callback(1)?;
    Ok(())
}

#[allow(dead_code)]
fn rpc_surface(mut rpc: Rpc) -> pvxs::Result<Value> {
    rpc.arg_string("command", "start")?
        .arg_double("value", 1.0)?
        .arg_int32("count", 1)?
        .arg_bool("enabled", true)?;
    rpc.execute(1.0)
}

#[allow(dead_code)]
fn server_surface(server: &Server) -> pvxs::Result<()> {
    let _ = server.handle();
    let _ = server.tcp_port();
    let _ = server.udp_port();
    server.create_pv_double("test:double", 0.0, NTScalarMetadataBuilder::new())?;
    server.create_pv_int32("test:int32", 0, NTScalarMetadataBuilder::new())?;
    server.create_pv_string("test:string", "", NTScalarMetadataBuilder::new())?;
    server.create_pv_enum(
        "test:enum",
        vec!["OFF", "ON"],
        0,
        NTEnumMetadataBuilder::new(),
    )?;
    server.post_double("test:double", 1.0)?;
    server.remove_pv("test:double")?;
    Ok(())
}

#[test]
fn monitor_event_implements_error() {
    assert_error::<MonitorEvent>();
}
