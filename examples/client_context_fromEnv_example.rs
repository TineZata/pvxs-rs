fn main() {
    let ctx: *mut pvxs::Context = pvxs::wrapper::client_context_from_env();
    if ctx.is_null() {
        println!("Failed to create context from environment");
        return;
    }
    println!("Context created from environment");
    pvxs::wrapper::context_close();
    println!("Context closed");
}