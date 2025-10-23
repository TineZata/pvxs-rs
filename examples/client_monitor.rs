//! Client example demonstrating PV monitoring
//!
//! This example shows how to:
//! - Create a PVXS client
//! - Monitor PV changes over time
//! - Handle connection state changes

use pvxs::{Client, Error};
use std::env;
use std::time::{Duration, Instant};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Get PV name from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <PV_NAME> [DURATION_SECONDS]", args[0]);
        eprintln!("Examples:");
        eprintln!("  {} MY:PV:NAME", args[0]);
        eprintln!("  {} MY:PV:NAME 30", args[0]);
        std::process::exit(1);
    }

    let pv_name = &args[1];
    let duration = if args.len() == 3 {
        args[2].parse::<u64>().unwrap_or(10)
    } else {
        10
    };

    println!("🔌 Creating PVXS client...");
    
    // Create client using environment configuration
    let mut client = Client::new()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    println!("✅ Client created successfully");
    
    // Check if PV exists first
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

    println!("📡 Starting to monitor PV: {} for {} seconds", pv_name, duration);
    println!("💡 Press Ctrl+C to stop early");
    println!();
    println!("{:<20} | {:<15} | {:<10} | {:<20} | {}", 
             "Timestamp", "Value", "Alarm", "Delta Time", "Notes");
    println!("{}", "-".repeat(80));

    // Simple polling-based monitoring since epics-pvxs-sys might not have
    // full monitor/subscription support yet
    let start_time = Instant::now();
    let mut last_value: Option<String> = None;
    let mut last_poll_time = start_time;
    let mut poll_count = 0;
    
    while start_time.elapsed().as_secs() < duration {
        let poll_start = Instant::now();
        
        match client.get(pv_name, 1.0) {
            Ok(value) => {
                let current_value = value.to_string();
                let alarm_info = value.alarm_info();
                let timestamp = if let Some(ts) = value.timestamp() {
                    format!("{:.3}", ts.as_f64())
                } else {
                    "N/A".to_string()
                };
                
                let delta_time = if poll_count > 0 {
                    format!("{:.3}s", poll_start.duration_since(last_poll_time).as_secs_f64())
                } else {
                    "Initial".to_string()
                };
                
                let notes = if let Some(ref last) = last_value {
                    if *last != current_value {
                        "VALUE CHANGED"
                    } else {
                        ""
                    }
                } else {
                    "First read"
                };
                
                // Only print if value changed or it's the first read or an alarm condition
                if last_value.as_ref() != Some(&current_value) || 
                   poll_count == 0 || 
                   alarm_info.has_alarm() ||
                   notes == "VALUE CHANGED" {
                    
                    let alarm_str = if alarm_info.has_alarm() {
                        format!("{}", alarm_info.severity)
                    } else {
                        "OK".to_string()
                    };
                    
                    println!("{:<20} | {:<15} | {:<10} | {:<20} | {}", 
                             timestamp, 
                             current_value, 
                             alarm_str,
                             delta_time,
                             notes);
                }
                
                last_value = Some(current_value);
            }
            
            Err(Error::Timeout { .. }) => {
                if poll_count == 0 {
                    println!("{:<20} | {:<15} | {:<10} | {:<20} | {}", 
                             "N/A", "TIMEOUT", "DISCONN", "N/A", "Connection lost");
                }
            }
            
            Err(e) => {
                if poll_count == 0 {
                    println!("{:<20} | {:<15} | {:<10} | {:<20} | {}", 
                             "N/A", "ERROR", "ERROR", "N/A", format!("Error: {}", e));
                }
            }
        }
        
        poll_count += 1;
        last_poll_time = poll_start;
        
        // Sleep for a short interval (1 second) before next poll
        thread::sleep(Duration::from_millis(1000));
    }
    
    println!();
    println!("⏱️  Monitoring completed after {} seconds", start_time.elapsed().as_secs());
    println!("📊 Total polls: {}", poll_count);
    
    // Get final value and show summary
    match client.get(pv_name, 2.0) {
        Ok(final_value) => {
            println!("🏁 Final value: {}", final_value);
            let alarm_info = final_value.alarm_info();
            if alarm_info.has_alarm() {
                println!("⚠️  Final alarm status: {}", alarm_info);
            }
        }
        Err(e) => {
            println!("❌ Could not get final value: {}", e);
        }
    }
    
    println!();
    println!("🎉 Monitoring completed successfully!");

    Ok(())
}