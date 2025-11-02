//! Monitor example demonstrating PV monitoring with callbacks
//!
//! This example shows how to:
//! - Create a server with a PV that changes over time
//! - Monitor the PV using both simple and builder interfaces
//! - Use callbacks to be notified of updates
//! - Pop values from the monitor queue
//!
//! Run with: cargo run --example monitor_example --features="client,server"

use pvxs::{Client, Server};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

// Global callback counter for demonstration
static CALLBACK_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Callback function that gets invoked when queue goes empty -> not-empty
extern "C" fn monitor_callback() {
    let count = CALLBACK_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    println!("📢 Callback #{}: New data available!", count);
}

#[cfg(all(feature = "client", feature = "server"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🚀 PVXS Monitor Example\n");

    // ========================================
    // Part 1: Create Server with Changing PV
    // ========================================
    println!("📡 Creating server...");
    let mut server = Server::new()?;
    
    let mut counter_pv = server.add_int32_pv("DEMO:COUNTER", 0)?;
    let mut sine_pv = server.add_double_pv("DEMO:SINE", 0.0)?;
    
    println!("✅ Server created with PVs:");
    println!("   - DEMO:COUNTER (int32, incrementing)");
    println!("   - DEMO:SINE (double, sine wave)");
    
    server.start()?;
    println!("✅ Server started on TCP port: {}", server.tcp_port());
    
    // Give server time to fully initialize
    thread::sleep(Duration::from_millis(500));

    // ========================================
    // Part 2: Create Client and Monitors
    // ========================================
    println!("\n🔌 Creating client...");
    let mut client = Client::new()?;
    println!("✅ Client created");

    // Simple monitor interface
    println!("\n📊 Creating simple monitor for DEMO:COUNTER...");
    let mut simple_monitor = client.monitor("DEMO:COUNTER")?;
    simple_monitor.start();
    println!("✅ Simple monitor started");

    // Builder monitor with callback
    println!("\n🔧 Creating monitor with builder for DEMO:SINE...");
    let mut builder_monitor = client.monitor_builder("DEMO:SINE")?
        .connection_events(false)      // Ignore connection events
        .disconnection_events(false)   // Ignore disconnection events
        .event(monitor_callback)       // Register callback
        .exec()?;
    
    builder_monitor.start();
    println!("✅ Builder monitor started with callback");

    // Give monitors time to connect
    thread::sleep(Duration::from_millis(1000));
    
    println!("\n📈 Monitor status:");
    println!("   Simple monitor connected: {}", simple_monitor.is_connected());
    println!("   Builder monitor connected: {}", builder_monitor.is_connected());

    // ========================================
    // Part 3: Update PVs and Receive Updates
    // ========================================
    println!("\n⚡ Starting PV updates...\n");
    
    let updates = 10;
    for i in 1..=updates {
        // Update counter PV
        counter_pv.post_int32(i)?;
        
        // Update sine PV
        let angle = (i as f64) * std::f64::consts::PI / 4.0;
        let sine_value = angle.sin();
        sine_pv.post_double(sine_value)?;
        
        println!("📤 Update #{}: counter={}, sine={:.4}", i, i, sine_value);
        
        // Give time for monitors to receive updates
        thread::sleep(Duration::from_millis(200));
        
        // Pop updates from simple monitor
        println!("   Simple monitor:");
        let mut pop_count = 0;
        while let Ok(Some(value)) = simple_monitor.pop() {
            pop_count += 1;
            if let Ok(val) = value.as_int() {
                println!("      📥 Popped counter value: {}", val);
            }
            // Only pop a few to avoid flooding output
            if pop_count >= 2 { break; }
        }
        if pop_count == 0 {
            println!("      (no updates in queue)");
        }
        
        // Pop updates from builder monitor
        println!("   Builder monitor:");
        pop_count = 0;
        while let Ok(Some(value)) = builder_monitor.pop() {
            pop_count += 1;
            if let Ok(val) = value.as_double() {
                println!("      📥 Popped sine value: {:.4}", val);
            }
            if pop_count >= 2 { break; }
        }
        if pop_count == 0 {
            println!("      (no updates in queue)");
        }
        
        println!();
        thread::sleep(Duration::from_millis(300));
    }

    // ========================================
    // Part 4: Drain remaining queue and stop
    // ========================================
    println!("🧹 Draining remaining monitor queues...\n");
    
    // Drain simple monitor
    let mut simple_count = 0;
    while let Ok(Some(value)) = simple_monitor.pop() {
        simple_count += 1;
        if let Ok(val) = value.as_int() {
            println!("   Simple monitor final value #{}: {}", simple_count, val);
        }
    }
    if simple_count > 0 {
        println!("   ✅ Drained {} values from simple monitor", simple_count);
    }
    
    // Drain builder monitor
    let mut builder_count = 0;
    while let Ok(Some(value)) = builder_monitor.pop() {
        builder_count += 1;
        if let Ok(val) = value.as_double() {
            println!("   Builder monitor final value #{}: {:.4}", builder_count, val);
        }
    }
    if builder_count > 0 {
        println!("   ✅ Drained {} values from builder monitor", builder_count);
    }

    println!("\n📊 Callback Summary:");
    let total_callbacks = CALLBACK_COUNTER.load(Ordering::SeqCst);
    println!("   Total callbacks fired: {}", total_callbacks);
    if total_callbacks > 0 {
        println!("   ✅ Callbacks working correctly!");
    } else {
        println!("   ⚠️  No callbacks fired (may be expected in some environments)");
    }

    // Stop monitors
    println!("\n🛑 Stopping monitors...");
    simple_monitor.stop();
    builder_monitor.stop();
    println!("✅ Monitors stopped");

    // Stop server
    println!("🛑 Stopping server...");
    server.stop()?;
    println!("✅ Server stopped");

    println!("\n✨ Monitor example completed successfully!");
    
    Ok(())
}

#[cfg(not(all(feature = "client", feature = "server")))]
fn main() {
    println!("This example requires both 'client' and 'server' features.");
    println!("Run with: cargo run --example monitor_example --features=\"client,server\"");
}
