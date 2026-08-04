# pvxs-rs

pvxs-rs is a pure Rust implementation of EPICS PVXS-style APIs for the EPICS PVXS, designed to keep application code familiar while removing native build friction: no C++ toolchain, no EPICS_BASE setup, and no PVXS DLL runtime packaging. (Historical note: `pvxs-sys` remains usable and was the earlier Rust PVXS wrapper around `pvxs.dll`, `ca.dll`, and `com.dll`.)

## Install

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
pvxs = "0.1"
```

Build and test normally with Cargo:

```bash
cargo check
cargo test
```

## Usage

### Client GET

```rust
use pvxs::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
	let mut ctx = Context::from_env()?;
	let value = ctx.get("TEST:DOUBLE", 5.0)?;
	println!("{}", value.get_field_double("value")?);
	Ok(())
}
```

### Client PUT

```rust
use pvxs::{Context, PvxsError};

fn main() -> Result<(), PvxsError> {
	let mut ctx = Context::from_env()?;
	ctx.put_double("TEST:DOUBLE", 42.0, 5.0)?;
	ctx.put_string("TEST:STRING", "hello", 5.0)?;
	Ok(())
}
```

### Monitor

Use `MonitorBuilder` and `start()` for subscriptions, then call `pop()` to read updates.

```rust
use pvxs::{Context, MonitorEvent, PvxsError};

fn main() -> Result<(), PvxsError> {
	let mut ctx = Context::from_env()?;
	let mut monitor = ctx
		.monitor_builder("TEST:COUNTER")?
		.connect_exception(true)
		.disconnect_exception(true)
		.exec()?;

	monitor.start()?;

	for _ in 0..10 {
		match monitor.pop() {
			Ok(Some(value)) => {
				println!("update={}", value.get_field_double("value")?);
			}
			Ok(None) => {
				std::thread::sleep(std::time::Duration::from_millis(50));
			}
			Err(MonitorEvent::Connected(msg)) => println!("connected: {msg}"),
			Err(MonitorEvent::Disconnected(msg)) => {
				println!("disconnected: {msg}");
				break;
			}
			Err(MonitorEvent::Finished(msg)) => {
				println!("finished: {msg}");
				break;
			}
			Err(e) => {
				println!("monitor error: {e}");
				break;
			}
		}
	}

	monitor.stop()?;
	Ok(())
}
```

### Server

Use `Server` to host PVs in memory.

```rust
use pvxs::{NTEnumMetadataBuilder, NTScalarMetadataBuilder, PvxsError, Server};

fn main() -> Result<(), PvxsError> {
	let server = Server::start_isolated()?;

	// create_pv_*
	server.create_pv_double("sensor:temp", 22.5, NTScalarMetadataBuilder::new())?;
	server.create_pv_string("device:name", "ioc-01", NTScalarMetadataBuilder::new())?;
	server.create_pv_int32_array("wave:counts", vec![1, 2, 3], NTScalarMetadataBuilder::new())?;
	server.create_pv_enum(
		"device:mode",
		vec!["Off", "On", "Auto"],
		1,
		NTEnumMetadataBuilder::new(),
	)?;

	// post_*
	server.post_double("sensor:temp", 23.0)?;
	server.post_string("device:name", "ioc-02")?;
	server.post_int32_array("wave:counts", vec![10, 20, 30])?;
	server.post_enum("device:mode", 2)?;

	// fetch_*
	let f_temp = server.fetch_double("sensor:temp")?;
	let f_name = server.fetch_string("device:name")?;
	let f_counts = server.fetch_int32_array("wave:counts")?;
	let f_mode = server.fetch_enum("device:mode")?;

	println!("temp={} severity={:?}", f_temp.value, f_temp.alarm_severity);
	println!("name={}", f_name.value);
	println!("counts={:?}", f_counts.value);
	println!("mode index={} choices={:?}", f_mode.value, f_mode.value_choices);

	server.stop_drop()?;
	Ok(())
}
```

### Server Metadata (Control + Alarm)

```rust
use pvxs::{
	AlarmMetadata, AlarmSeverity, NTScalarMetadataBuilder, PvxsError, Server,
	ControlMetadata,
};

fn main() -> Result<(), PvxsError> {
	let server = Server::start_isolated()?;

	let metadata = NTScalarMetadataBuilder::new()
		.control(ControlMetadata {
			limit_low: 0.0,
			limit_high: 100.0,
			min_step: 0.1,
		})
		.alarm_metadata(AlarmMetadata {
			active: true,
			low_alarm_limit: 5.0,
			low_warning_limit: 10.0,
			high_warning_limit: 90.0,
			high_alarm_limit: 95.0,
			low_alarm_severity: AlarmSeverity::Major,
			low_warning_severity: AlarmSeverity::Minor,
			high_warning_severity: AlarmSeverity::Minor,
			high_alarm_severity: AlarmSeverity::Major,
			hysteresis: 0,
		});

	server.create_pv_double("sensor:temp", 25.0, metadata)?;
	server.post_double("sensor:temp", 96.0)?;

	let fetched = server.fetch_double("sensor:temp")?;
	println!(
		"value={} severity={:?} status={:?}",
		fetched.value, fetched.alarm_severity, fetched.alarm_status
	);

	server.stop_drop()?;
	Ok(())
}
```

### Enum Metadata

```rust
use pvxs::{NTEnumMetadataBuilder, PvxsError, Server};

fn main() -> Result<(), PvxsError> {
	let server = Server::start_isolated()?;
	server.create_pv_enum(
		"device:mode",
		vec!["Off", "On", "Auto"],
		1,
		NTEnumMetadataBuilder::new(),
	)?;

	server.post_enum("device:mode", 2)?;
	let fetched = server.fetch_enum("device:mode")?;
	let label = fetched
		.value_choices
		.get(fetched.value as usize)
		.map(String::as_str)
		.unwrap_or("<invalid>");
	println!("index={} label={}", fetched.value, label);

	server.stop_drop()?;
	Ok(())
}
```

## Tests

The tests in `tests/` are the best reference for real usage, especially local server workflows, remote GET/PUT flows, monitors, and alarm behavior.

## Inspiration

Kudos to [`epics-rs`](https://github.com/epics-rs/epics-rs) and its author for proving out the broader idea that EPICS infrastructure can be implemented cleanly in pure Rust.
