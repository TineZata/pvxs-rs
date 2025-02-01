use pvxs::std_types::{StdFunction, GetBuilder};

fn initialisation_callback() {
    println!("Init callback");
}

fn result_callback() {
    println!("Result callback");
}

fn main() {
    
    let init_callback = StdFunction::new(initialisation_callback);
    let result_callback = StdFunction::new(result_callback);

    let mut builder = pvxs::get_context_info("PV:Integer".to_string());
    // Assign a callback to on_init
    builder.on_init(init_callback);
    // Assign a callback to on_result
    builder.on_result(result_callback);

}