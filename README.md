# pvxs-rs - High-level Rust bindings for EPICS PVXS

High-level, idiomatic Rust bindings for [EPICS PVXS](https://github.com/epics-base/pvxs) (PVAccess) with separate Client and Server APIs.

Built on top of [`epics-pvxs-sys`](https://github.com/TineZata/epics-pvxs-sys), this crate provides a safe, ergonomic interface for EPICS Process Variable operations.

## Features

- 🔌 **Client API**: Connect to EPICS IOCs, get/put/monitor Process Variables
- 🖥️ **Server API**: Create PVXS servers and provide PVs to the network
- ⚡ **Async Support**: Optional tokio-based async operations
- 🛡️ **Type Safety**: Strong typing with comprehensive error handling
- 🚀 **High Performance**: Zero-copy operations where possible
- 📊 **Rich Value Types**: Support for scalars, arrays, structures with timestamps and alarms
- 🔧 **Builder Patterns**: Fluent APIs for configuration

## Quick Start

### Client Usage

```rust
use pvxs::Client;

fn main() -> Result<(), pvxs::Error> {
    // Create a client using EPICS environment variables
    let client = Client::new()?;
    
    // Get a PV value
    let value = client.get("MY:PV:NAME", 5.0)?;
    println!("Value: {}", value.as_double()?);
    println!("Alarm: {}", value.alarm_info());
    
    // Put a new value (generic - supports f64, i32, &str, String)
    client.put("MY:PV:NAME", 42.5, 5.0)?;
    
    // Legacy method also available
    client.put_double("MY:PV:NAME", 42.5, 5.0)?;
    
    Ok(())
}
```

### Server Usage

```rust
use pvxs::{Server, server::PvValue};

fn main() -> Result<(), pvxs::Error> {
    // Create a server
    let server = Server::new()?;
    
    // Add some PVs
    server.add_pv("DEMO:COUNTER", PvValue::Int32(0))?;
    server.add_pv("DEMO:TEMPERATURE", PvValue::Double(20.0))?;
    server.add_pv("DEMO:STATUS", PvValue::String("Running".to_string()))?;
    
    // Start the server
    server.start()?;
    println!("Server running with {} PVs", server.list_pvs().len());
    
    // Update values
    server.update_pv("DEMO:COUNTER", PvValue::Int32(1))?;
    
    Ok(())
}
```

### Async Client Usage

```rust
use pvxs::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;
    
    // Concurrent operations on multiple PVs
    let pv_names = ["PV1", "PV2", "PV3"];
    let futures = pv_names.iter().map(|name| async {
        tokio::task::spawn_blocking({
            let name = name.to_string();
            move || client.get(&name, 3.0)
        }).await
    });
    
    let results = futures::future::join_all(futures).await;
    
    for (name, result) in pv_names.iter().zip(results) {
        match result? {
            Ok(value) => println!("{}: {}", name, value),
            Err(e) => println!("{}: Error - {}", name, e),
        }
    }
    
    Ok(())
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
# Use the latest development version (synchronous operations by default)
pvxs = { git = "https://github.com/TineZata/pvxs-rs", branch = "main" }

# For async support (⚠️ experimental - see limitations below)
pvxs = { git = "https://github.com/TineZata/pvxs-rs", branch = "main", features = ["async"] }

# For all features
pvxs = { git = "https://github.com/TineZata/pvxs-rs", branch = "main", features = ["full"] }
```

### Prerequisites

Before using this crate, you need:

1. **EPICS Base** (≥3.15.1) - [Download](https://epics-controls.org/resources-and-support/base/)
2. **PVXS Library** (≥1.0.0) - [Download](https://github.com/epics-base/pvxs) 
3. **epics-pvxs-sys** - The low-level bindings (automatically included)

Set these environment variables:

```bash
export EPICS_BASE=/path/to/epics/base
export EPICS_HOST_ARCH=linux-x86_64  # or windows-x64, darwin-x86, etc.
export EPICS_PVXS=/path/to/pvxs      # Optional, defaults to EPICS_BASE/pvxs
```

For detailed installation instructions, see the [epics-pvxs-sys documentation](https://github.com/TineZata/epics-pvxs-sys).

## Architecture

This crate is organized into separate modules for different use cases:

```
pvxs/
├── client/     # Client API for connecting to PVs
├── server/     # Server API for providing PVs
├── types/      # Common value types and utilities  
└── error/      # Error handling
```

### Client API

The client API provides both synchronous and asynchronous interfaces:

```rust
use pvxs::client::{Client, ClientBuilder};

// Simple client
let client = Client::new()?;

// Configured client
let client = ClientBuilder::new()
    .addr_list("192.168.1.100:5076")
    .auto_beacon_addr_list(false)
    .build()?;

// Operations
let value = client.get("PV:NAME", 5.0)?;
client.put_double("PV:DOUBLE", 42.0, 5.0)?;
client.put_string("PV:STRING", "hello", 5.0)?;

// Convenience methods
let value = client.quick_get("PV:NAME")?;  // 5s timeout
client.quick_put_double("PV:NAME", 42.0)?;
```

### Server API

The server API allows you to provide PVs to the network:

```rust
use pvxs::{Server, server::PvValue, types::AlarmSeverity};

let server = Server::new()?;

// Add PVs with different types
server.add_pv("TEMP:01", PvValue::Double(25.0))?;
server.add_pv("STATUS", PvValue::String("OK".to_string()))?;
server.add_pv("COUNTER", PvValue::Int32(0))?;
server.add_pv("WAVEFORM", PvValue::DoubleArray(vec![1.0, 2.0, 3.0]))?;

// Set alarms
server.set_alarm("TEMP:01", AlarmSeverity::Minor, "High temperature")?;

// Update values
server.update_pv("COUNTER", PvValue::Int32(1))?;

// Server lifecycle
server.start()?;
println!("Server stats: {}", server.stats());
server.stop()?;
```

### Value Types and Metadata

Rich value types with timestamp and alarm information:

```rust
use pvxs::types::{Value, AlarmSeverity, Timestamp};

let value = client.get("TEMP:SENSOR", 5.0)?;

// Access the main value
let temperature = value.as_double()?;
println!("Temperature: {:.1}°C", temperature);

// Access specific fields
let setpoint = value.get_double("setpoint")?;
let units = value.get_string("units")?;

// Alarm information
let alarm = value.alarm_info();
if alarm.has_alarm() {
    println!("⚠️  Alarm: {}", alarm);
}

// Timestamp
if let Some(timestamp) = value.timestamp() {
    println!("Updated: {:.3}s ago", timestamp.as_f64());
}
```

## Examples

The crate includes comprehensive examples:

### Client Examples

```bash
# Get a PV value with detailed information
cargo run --example client_get -- MY:PV:NAME

# Put a value to a PV
cargo run --example client_put -- MY:PV:NAME 42.5

# Monitor a PV for changes
cargo run --example client_monitor -- MY:PV:NAME 30

# Async operations on multiple PVs
cargo run --features async --example async_client -- PV1 PV2 PV3
```

### Server Example

```bash
# Run a demo server providing various PV types
cargo run --example server_basic

# Connect from another terminal:
# pvget DEMO:COUNTER
# pvget DEMO:TEMPERATURE  
# pvmonitor DEMO:SINE_WAVE
```

## Features

- `client` (default) - Client API for connecting to PVs
- `server` - Server API for providing PVs  
- `async` - Async/await support with tokio (⚠️ experimental)
- `serde` - Serialization support for values
- `full` - All features enabled

### ⚠️ Current Limitations

- **Async Support**: The `async` feature is experimental due to C++ thread safety constraints in the underlying epics-pvxs library. The C++ types are not `Send`/`Sync`, making true async operations challenging.
- **Limited PUT Operations**: The generic `put()` method supports `f64`, `i32`, `&str`, and `String` types, but currently only `f64` is fully functional. Other types will return a "not implemented" error until the underlying `epics-pvxs-sys` library adds support for them.
- **Server API**: The server implementation is a placeholder and not yet fully functional.

## Error Handling

Comprehensive error types with context:

```rust
use pvxs::Error;

match client.get("NONEXISTENT:PV", 5.0) {
    Ok(value) => println!("Value: {}", value),
    Err(Error::PvNotFound { pv_name }) => {
        eprintln!("PV '{}' not found", pv_name);
    }
    Err(Error::Timeout { timeout }) => {
        eprintln!("Timeout after {}s", timeout);
    }
    Err(Error::ConnectionError { message }) => {
        eprintln!("Connection failed: {}", message);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Performance Considerations

- Operations are synchronous by default for simplicity
- Use `features = ["async"]` for concurrent operations
- Values use zero-copy where possible with the underlying PVXS library
- Connection pooling and caching handled by PVXS library

## Compatibility

- **Rust**: 1.70+
- **EPICS Base**: 3.15.1+
- **PVXS**: 1.0.0+
- **Platforms**: Linux, Windows, macOS

## Development

### Building

```bash
# Standard build
cargo build

# With all features
cargo build --features full

# Run tests (requires EPICS environment)
cargo test

# Run examples
cargo run --example client_get -- TEST:PV
```

### Environment Setup

For development, set up your EPICS environment:

```bash
# Linux/macOS
export EPICS_BASE=/opt/epics/base
export EPICS_HOST_ARCH=linux-x86_64
export PATH=$EPICS_BASE/bin/$EPICS_HOST_ARCH:$PATH

# Test with a soft IOC
softIoc -d test.db
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure examples work
5. Submit a pull request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Related Projects

- [`epics-pvxs-sys`](https://github.com/TineZata/epics-pvxs-sys) - Low-level FFI bindings
- [EPICS PVXS](https://github.com/epics-base/pvxs) - The underlying C++ library
- [EPICS Base](https://epics-controls.org/) - The EPICS control system toolkit

## Acknowledgments

- The EPICS collaboration for the PVXS library
- The Rust community for excellent FFI tooling
- Contributors to epics-pvxs-sys for the foundational bindings