use pvxs::{Client, Result};

fn main() -> Result<()> {
    println!("Creating PVXS client...");
    let mut client = Client::new()?;
    println!("Client created successfully!");

    // Generic put with f64 (currently supported)
    println!("\nPutting double value...");
    match client.put("MY:PV:DOUBLE", 42.5, 5.0) {
        Ok(_) => println!("Successfully put double value 42.5 to MY:PV:DOUBLE"),
        Err(e) => println!("Failed to put double value: {}", e),
    }

    // Generic put with i32 (not yet implemented)
    println!("\nAttempting to put i32 value...");
    match client.put("MY:PV:INT", 123_i32, 5.0) {
        Ok(_) => println!("Successfully put i32 value 123 to MY:PV:INT"),
        Err(e) => println!("Failed to put i32 value: {}", e),
    }

    // Generic put with string (not yet implemented)
    println!("\nAttempting to put string value...");
    match client.put("MY:PV:STRING", "hello", 5.0) {
        Ok(_) => println!("Successfully put string value 'hello' to MY:PV:STRING"),
        Err(e) => println!("Failed to put string value: {}", e),
    }

    // Generic put with String (not yet implemented)
    println!("\nAttempting to put String value...");
    match client.put("MY:PV:STRING2", String::from("world"), 5.0) {
        Ok(_) => println!("Successfully put String value 'world' to MY:PV:STRING2"),
        Err(e) => println!("Failed to put String value: {}", e),
    }

    println!("\nNote: Only f64 is currently supported. i32 and String support will be added when epics-pvxs-sys provides the necessary FFI functions.");

    Ok(())
}
