//use pvxs::pvxs_library::PvxsLibrary;
//use pvxs::client_context::ClientContext;

///
/// This example demonstrates how to get the information of a PV from the server.
/// Create an IOC which has this record:
/// ```
/// record(ai, "rec:X") {
///     info(Q:group, {
///         "grp:name": {
///             "X": {+channel:"VAL"}
///         }
///     })
///     field(DESC, "Analog input record example rec:X")
///     field(EGU, "mm")
///     field(PREC, "3")
/// }
/// ```
/// 
fn main() {

    /*let pvxs_library: PvxsLibrary = match PvxsLibrary::new() {
        Ok(lib) => lib,
        Err(_) => panic!("Failed to load the PvxsLibrary"),
    };*/

    /*let ctx: *mut ClientContext = ClientContext::context_from_env();
    if ctx.is_null() {
        println!("Failed to create context from environment");
        return;
    }
    else {
        let pv_name = "rec:X";
        let info: Result<pvxs::GetBuilder, String> = pvxs::wrapper::client_context_info(ctx, pv_name);
    }
    println!("Context created from environment");*/
}