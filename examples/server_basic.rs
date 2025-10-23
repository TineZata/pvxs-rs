//! Basic server example demonstrating PV hosting
//!
//! This example shows how to:
//! - Create a PVXS server
//! - Add various types of PVs
//! - Update PV values over time
//! - Handle server lifecycle

#[cfg(feature = "server")]
use pvxs::{Server, server::PvValue, types::AlarmSeverity};
#[cfg(feature = "server")]
use std::time::{Duration, Instant};
#[cfg(feature = "server")]
use std::thread;
#[cfg(feature = "server")]
use std::sync::Arc;
#[cfg(feature = "server")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "server")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🔌 Creating PVXS server...");
    
    // Create server
    let server = Arc::new(Server::new()
        .map_err(|e| format!("Failed to create server: {}", e))?);

    println!("✅ Server created successfully");

    // Add various types of PVs to demonstrate different data types
    println!("📡 Adding Process Variables...");
    
    // Counter PV that increments over time
    server.add_pv("DEMO:COUNTER", PvValue::Int32(0))?;
    println!("  ✓ Added DEMO:COUNTER (integer counter)");
    
    // Temperature simulation PV
    server.add_pv("DEMO:TEMPERATURE", PvValue::Double(20.0))?;
    println!("  ✓ Added DEMO:TEMPERATURE (double, simulated temperature)");
    
    // Status string PV
    server.add_pv("DEMO:STATUS", PvValue::String("Initializing".to_string()))?;
    println!("  ✓ Added DEMO:STATUS (string status)");
    
    // Sine wave PV for continuous changes
    server.add_pv("DEMO:SINE_WAVE", PvValue::Double(0.0))?;
    println!("  ✓ Added DEMO:SINE_WAVE (double, sine wave)");
    
    // Boolean toggle PV
    server.add_pv("DEMO:TOGGLE", PvValue::Bool(false))?;
    println!("  ✓ Added DEMO:TOGGLE (boolean toggle)");
    
    // High precision timestamp
    server.add_pv("DEMO:TIMESTAMP", PvValue::Double(0.0))?;
    println!("  ✓ Added DEMO:TIMESTAMP (double, current time)");

    // Array demonstration
    server.add_pv("DEMO:WAVEFORM", PvValue::DoubleArray(vec![0.0; 10]))?;
    println!("  ✓ Added DEMO:WAVEFORM (double array, 10 elements)");

    println!();
    println!("📋 Server provides the following PVs:");
    for pv_name in server.list_pvs() {
        let value = server.get_pv_value(&pv_name)?;
        println!("  • {} = {}", pv_name, value);
    }

    // Start the server
    println!();
    println!("🚀 Starting server...");
    server.start()?;
    println!("✅ Server started successfully");

    println!();
    println!("📡 Server is now providing PVs. Statistics:");
    println!("   {}", server.stats());
    
    println!();
    println!("💡 You can now connect to these PVs from EPICS clients:");
    println!("   pvget DEMO:COUNTER");
    println!("   pvget DEMO:TEMPERATURE");
    println!("   pvget DEMO:STATUS");
    println!("   pvmonitor DEMO:SINE_WAVE");
    
    println!();
    println!("🔄 Server will run for 60 seconds with automatic updates...");
    println!("💡 Press Ctrl+C to stop early");

    // Set up shutdown handling
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    // In a real application, you'd set up proper signal handling
    // For this example, we'll just run for a limited time
    
    let start_time = Instant::now();
    let mut counter = 0i32;
    let mut toggle_state = false;
    
    // Update status to running
    server.update_pv("DEMO:STATUS", PvValue::String("Running".to_string()))?;
    
    // Simulation loop
    while start_time.elapsed().as_secs() < 60 && running.load(Ordering::Relaxed) {
        let elapsed = start_time.elapsed().as_secs_f64();
        
        // Update counter every second
        counter += 1;
        server.update_pv("DEMO:COUNTER", PvValue::Int32(counter))?;
        
        // Simulate temperature with some variation
        let temp = 20.0 + 5.0 * (elapsed * 0.1).sin() + (fastrand::f64() - 0.5) * 2.0;
        server.update_pv("DEMO:TEMPERATURE", PvValue::Double(temp))?;
        
        // Set alarm if temperature is too high
        if temp > 24.0 {
            server.set_alarm("DEMO:TEMPERATURE", AlarmSeverity::Minor, "High temperature")?;
        } else if temp > 26.0 {
            server.set_alarm("DEMO:TEMPERATURE", AlarmSeverity::Major, "Very high temperature")?;
        } else {
            server.set_alarm("DEMO:TEMPERATURE", AlarmSeverity::None, "")?;
        }
        
        // Update sine wave
        let sine_val = (elapsed * 2.0).sin();
        server.update_pv("DEMO:SINE_WAVE", PvValue::Double(sine_val))?;
        
        // Toggle boolean every 5 seconds
        if counter % 5 == 0 {
            toggle_state = !toggle_state;
            server.update_pv("DEMO:TOGGLE", PvValue::Bool(toggle_state))?;
        }
        
        // Update timestamp
        server.update_pv("DEMO:TIMESTAMP", PvValue::Double(elapsed))?;
        
        // Update waveform with a simple pattern
        let mut waveform: Vec<f64> = (0..10)
            .map(|i| (elapsed + i as f64 * 0.1).sin())
            .collect();
        server.update_pv("DEMO:WAVEFORM", PvValue::DoubleArray(waveform))?;
        
        // Print status every 10 seconds
        if counter % 10 == 0 {
            println!("📊 Server status at {}s:", elapsed as u32);
            println!("   Counter: {}", counter);
            println!("   Temperature: {:.1}°C", temp);
            println!("   Sine wave: {:.3}", sine_val);
            println!("   Toggle: {}", toggle_state);
            println!("   {}", server.stats());
        }
        
        // Sleep for 1 second
        thread::sleep(Duration::from_secs(1));
    }
    
    // Update status before shutdown
    server.update_pv("DEMO:STATUS", PvValue::String("Shutting down".to_string()))?;
    thread::sleep(Duration::from_millis(100)); // Give clients a moment to see the status change
    
    println!();
    println!("🛑 Stopping server...");
    server.stop()?;
    println!("✅ Server stopped successfully");
    
    println!();
    println!("📊 Final statistics:");
    println!("   Runtime: {:.1} seconds", start_time.elapsed().as_secs_f64());
    println!("   Updates performed: {}", counter);
    println!("   PVs provided: {}", server.list_pvs().len());
    
    println!();
    println!("🎉 Server demonstration completed successfully!");

    Ok(())
}

#[cfg(not(feature = "server"))]
fn main() {
    eprintln!("❌ This example requires the 'server' feature to be enabled.");
    eprintln!("💡 Run with: cargo run --features server --example server_basic");
    std::process::exit(1);
}