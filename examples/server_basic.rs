//! Basic server example demonstrating PV hosting
//!
//! This example shows how to:
//! - Create a PVXS server
//! - Add various types of PVs
//! - Update PV values over time
//! - Handle server lifecycle

#[cfg(feature = "server")]
use pvxs::Server;
#[cfg(feature = "server")]
use std::time::{Duration, Instant};

#[cfg(feature = "server")]

#[cfg(feature = "server")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🔌 Creating PVXS server...");
    
    // Create server
    let mut server = Server::new()
        .map_err(|e| format!("Failed to create server: {}", e))?;

    println!("✅ Server created successfully");

    // Add various types of PVs to demonstrate different data types
    println!("📡 Adding Process Variables...");
    
    // Counter PV that increments over time
    let mut counter_pv = server.add_int32_pv("DEMO:COUNTER", 0)?;
    println!("  ✓ Added DEMO:COUNTER (integer counter)");
    
    // Temperature simulation PV
    let mut temp_pv = server.add_double_pv("DEMO:TEMPERATURE", 20.0)?;
    println!("  ✓ Added DEMO:TEMPERATURE (double, simulated temperature)");
    
    // Status string PV
    let mut status_pv = server.add_string_pv("DEMO:STATUS", "Initializing")?;
    println!("  ✓ Added DEMO:STATUS (string status)");
    
    // Sine wave PV for continuous changes
    let mut sine_pv = server.add_double_pv("DEMO:SINE_WAVE", 0.0)?;
    println!("  ✓ Added DEMO:SINE_WAVE (double, sine wave)");
    
    // Read-only constant
    let _readonly_pv = server.add_readonly_double_pv("DEMO:CONSTANT", 299792458.0)?;
    println!("  ✓ Added DEMO:CONSTANT (readonly double, speed of light)");

    // Start the server
    println!();
    println!("🚀 Starting server...");
    server.start()?;
    println!("✅ Server started successfully");

    println!();
    println!("📡 Server is now providing PVs on TCP port {} and UDP port {}", 
             server.tcp_port(), server.udp_port());
    
    println!();
    println!("💡 You can now connect to these PVs from EPICS clients:");
    println!("   pvget DEMO:COUNTER");
    println!("   pvget DEMO:TEMPERATURE");
    println!("   pvget DEMO:STATUS");
    println!("   pvmonitor DEMO:SINE_WAVE");
    println!("   pvget DEMO:CONSTANT");
    
    println!();
    println!("🔄 Server will run for 60 seconds with automatic updates...");
    println!("💡 Press Ctrl+C to stop early");

    let start_time = Instant::now();
    let mut counter = 0i32;
    
    // Update status to running
    status_pv.post_string("Running")?;
    
    // Simulation loop
    while start_time.elapsed().as_secs() < 60 {
        let elapsed = start_time.elapsed().as_secs_f64();
        
        // Update counter every second
        counter += 1;
        counter_pv.post_int32(counter)?;
        
        // Simulate temperature with some variation
        let temp = 20.0 + 5.0 * (elapsed * 0.1).sin() + (fastrand::f64() - 0.5) * 2.0;
        
        // Set alarm if temperature is too high
        if temp > 26.0 {
            temp_pv.post_double_with_alarm(temp, 2, 0, "Very high temperature")?;
        } else if temp > 24.0 {
            temp_pv.post_double_with_alarm(temp, 1, 0, "High temperature")?;
        } else {
            temp_pv.post_double(temp)?;
        }
        
        // Update sine wave
        let sine_val = (elapsed * 2.0).sin();
        sine_pv.post_double(sine_val)?;
        
        // Update status every 5 seconds
        if counter % 5 == 0 {
            let status = format!("Running ({}s elapsed)", elapsed as u32);
            status_pv.post_string(&status)?;
        }
        
        // Print status every 10 seconds
        if counter % 10 == 0 {
            println!("📊 Server status at {}s:", elapsed as u32);
            println!("   Counter: {}", counter);
            println!("   Temperature: {:.1}°C", temp);
            println!("   Sine wave: {:.3}", sine_val);
        }
        
        // Sleep for 1 second
        std::thread::sleep(Duration::from_secs(1));
    }
    
    // Update status before shutdown
    status_pv.post_string("Shutting down")?;
    std::thread::sleep(Duration::from_millis(100)); // Give clients a moment to see the status change
    
    println!();
    println!("🛑 Stopping server...");
    server.stop()?;
    println!("✅ Server stopped successfully");
    
    println!();
    println!("📊 Final statistics:");
    println!("   Runtime: {:.1} seconds", start_time.elapsed().as_secs_f64());
    println!("   Updates performed: {}", counter);
    
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