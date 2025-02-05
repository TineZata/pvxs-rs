fn initialisation_callback() {
    println!("Init callback");
}

fn result_callback() {
    println!("Result callback");
}

fn main() {
    
    //let init_callback = StdFunction64::new(initialisation_callback);
    //let result_callback = StdFunction64::new(result_callback);

    //let mut builder: pvxs::std_types::GetBuilder = pvxs::get_context_info("PV:Integer".to_string());
    // Assign a callback to on_init
    //builder.on_init(init_callback);
    // Assign a callback to on_result
    //builder.on_result(result_callback);

}