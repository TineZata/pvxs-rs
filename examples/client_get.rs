//! Simple client example demonstrating GET operations
//!
//! This example shows how to:
//! - Create a PVXS client
//! - Get PV values with error handling
//! - Display value information including alarms and timestamps

use pvxs::{Client, Error};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Get PV name from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <PV_NAME>", args[0]);
        eprintln!("Example: {} MY:PV:NAME", args[0]);
        std::process::exit(1);
    }

    let pv_name = &args[1];

    println!("🔌 Creating PVXS client...");
    
    // Create client using environment configuration
    let mut client = Client::new()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    println!("✅ Client created successfully");
    println!("📡 Getting value for PV: {}", pv_name);

    // Get PV value with 5 second timeout
    match client.get(pv_name, 5.0) {
        Ok(value) => {
            println!("✅ Successfully retrieved PV value");
            println!();
            
            // Display basic value
            println!("📊 Value: {}", value);
            
            // Try to get specific field types
            println!();
            println!("🔍 Detailed value information:");
            
            if let Ok(double_val) = value.as_double() {
                println!("  • Double value: {}", double_val);
            }
            
            if let Ok(string_val) = value.as_string() {
                println!("  • String value: \"{}\"", string_val);
            }
            
            if let Ok(int_val) = value.as_int() {
                println!("  • Integer value: {}", int_val);
            }
            
            // Display alarm information
            let alarm_info = value.alarm_info();
            println!("  • Alarm status: {}", alarm_info);
            
            // Display timestamp if available
            if let Some(timestamp) = value.timestamp() {
                println!("  • Timestamp: {} seconds since EPICS epoch", timestamp.as_f64());
                println!("    ({}s + {}ns)", timestamp.seconds_past_epoch, timestamp.nanoseconds);
            } else {
                println!("  • Timestamp: Not available");
            }
            
            println!();
            println!("🎉 Operation completed successfully!");
        }
        
        Err(Error::Timeout { timeout }) => {
            eprintln!("❌ Timeout after {}s - PV may not exist or IOC may be unreachable", timeout);
            eprintln!("💡 Try:");
            eprintln!("   - Check if the IOC is running");
            eprintln!("   - Verify the PV name is correct");
            eprintln!("   - Check network connectivity");
            std::process::exit(1);
        }
        
        Err(Error::PvNotFound { pv_name }) => {
            eprintln!("❌ PV not found: {}", pv_name);
            eprintln!("💡 Try:");
            eprintln!("   - Check the PV name spelling");
            eprintln!("   - Verify the IOC provides this PV");
            eprintln!("   - Use 'pvlist' to see available PVs");
            std::process::exit(1);
        }
        
        Err(Error::ConnectionError { message }) => {
            eprintln!("❌ Connection error: {}", message);
            eprintln!("💡 Try:");
            eprintln!("   - Check EPICS_PVA_ADDR_LIST environment variable");
            eprintln!("   - Verify network configuration");
            eprintln!("   - Check firewall settings");
            std::process::exit(1);
        }
        
        Err(e) => {
            eprintln!("❌ Unexpected error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}