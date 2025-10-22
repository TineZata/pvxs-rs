use pvxs::client::context::Context;

fn main() {
    let ctx: Context = pvxs::get_context_from_env();
    if ctx.pvt._base._ptr.is_null() {
        println!("Failed to create context from environment");
        return;
    }
    println!("Context created from environment");
    dbg!(ctx);
}