//! Simple client example demonstrating PUT operations
//!
//! This example shows how to:
//! - Create a PVXS client
//! - Put different types of values to PVs
//! - Handle errors appropriately

use pvxs::{Client, Error};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Get PV name and value from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <PV_NAME> <VALUE>", args[0]);
        eprintln!("Examples:");
        eprintln!("  {} MY:DOUBLE:PV 42.5", args[0]);
        eprintln!("  {} MY:STRING:PV \"Hello World\"", args[0]);
        eprintln!("  {} MY:INT:PV 123", args[0]);
        std::process::exit(1);
    }

    let pv_name = &args[1];
    let value_str = &args[2];

    println!("🔌 Creating PVXS client...");
    
    // Create client using environment configuration
    let mut client = Client::new()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    println!("✅ Client created successfully");
    
    // First, check if PV exists by getting its info
    println!("🔍 Checking if PV exists: {}", pv_name);
    match client.exists(pv_name, 3.0) {
        Ok(true) => println!("✅ PV found"),
        Ok(false) => {
            eprintln!("❌ PV not found: {}", pv_name);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("❌ Error checking PV: {}", e);
            std::process::exit(1);
        }
    }

    // Try to determine the value type and put accordingly
    println!("📤 Putting value '{}' to PV: {}", value_str, pv_name);

    let result = try_put_value(&mut client, pv_name, value_str);
    
    match result {
        Ok(value_type) => {
            println!("✅ Successfully put {} value to PV", value_type);
            
            // Try to read back the value to confirm
            println!("🔄 Reading back value to confirm...");
            match client.get(pv_name, 3.0) {
                Ok(value) => {
                    println!("✅ Confirmed new value: {}", value);
                    
                    // Show alarm status in case the new value caused an alarm
                    let alarm_info = value.alarm_info();
                    if alarm_info.has_alarm() {
                        println!("⚠️  Alarm status: {}", alarm_info);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Could not read back value: {}", e);
                }
            }
            
            println!();
            println!("🎉 PUT operation completed successfully!");
        }
        
        Err(Error::Timeout { timeout }) => {
            eprintln!("❌ Timeout after {}s during PUT operation", timeout);
            eprintln!("💡 The PV might be read-only or the IOC might be busy");
            std::process::exit(1);
        }
        
        Err(Error::PvNotFound { pv_name }) => {
            eprintln!("❌ PV disappeared during operation: {}", pv_name);
            std::process::exit(1);
        }
        
        Err(Error::ConnectionError { message }) => {
            eprintln!("❌ Connection error during PUT: {}", message);
            eprintln!("💡 The PV might be read-only or access rights might be restricted");
            std::process::exit(1);
        }
        
        Err(Error::TypeConversion { message }) => {
            eprintln!("❌ Type conversion error: {}", message);
            eprintln!("💡 Try a different value format or check the PV's expected type");
            std::process::exit(1);
        }
        
        Err(e) => {
            eprintln!("❌ Unexpected error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Try to put a value, attempting different data types
fn try_put_value(client: &mut Client, pv_name: &str, value_str: &str) -> Result<&'static str, Error> {
    // First try to parse as a floating-point number
    if let Ok(double_val) = value_str.parse::<f64>() {
        client.put_double(pv_name, double_val, 5.0)?;
        return Ok("double");
    }
    
    // For now, only double values are supported in epics-pvxs-sys
    // Convert integer strings to doubles
    if let Ok(int_val) = value_str.parse::<i32>() {
        client.put_double(pv_name, int_val as f64, 5.0)?;
        return Ok("integer (as double)");
    }
    
    // String values not yet supported - convert to double if possible
    Err(Error::type_conversion(format!(
        "Value '{}' cannot be converted to double. Only double values are currently supported.", 
        value_str
    )))
}