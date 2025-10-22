fn my_callback(value: i32) -> i32 {
    println!("Callback called with: {}", value);
    value * 2
}

fn main() {
    let is_connected = pvxs::client_is_pv_connected("pv:name".to_string());
}