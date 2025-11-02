# PVXS-RS API Update Summary

## Overview
Updated pvxs-rs to fully support the latest epics-pvxs-sys development branch API, including enum types, array types, and monitor callbacks.

## Changes Made

### 1. Enum Support (types.rs)
Added methods to Value type for reading enum fields:
- `get_enum(field)` - Get i16 enum index from any field
- `get_enum_array(field)` - Get Vec<i16> enum array from any field  
- `as_enum()` - Convenience method for "value" field
- `as_enum_array()` - Convenience method for "value" field as array

**Note**: Enums are represented as i16 indices. On the server side, enum PVs can be backed by int32 PVs since epics-pvxs-sys doesn't have a dedicated `create_pv_enum` method yet.

### 2. Array Support (types.rs)
Added methods to Value type for reading array fields:
- `get_double_array(field)` - Get Vec<f64> from any field
- `get_int_array(field)` - Get Vec<i32> from any field
- `get_string_array(field)` - Get Vec<String> from any field
- `as_double_array()` - Convenience method for "value" field
- `as_int_array()` - Convenience method for "value" field
- `as_string_array()` - Convenience method for "value" field

**Note**: Arrays are read-only via get_field_*_array methods. The underlying SharedPV only supports scalar post operations (post_double, post_int32, post_string).

### 3. Monitor & MonitorBuilder Support (client.rs)
Added high-level wrappers for monitoring PV changes:

#### Monitor struct
- `start()` - Begin receiving updates
- `stop()` - Stop receiving updates  
- `pop()` - Retrieve next update from queue (Result<Option<Value>>)
- `is_connected()` - Check connection status
- `has_update()` - Check if updates available
- `name()` - Get monitored PV name

#### MonitorBuilder struct
- `connection_events(bool)` - Enable/disable connection event notifications
- `disconnection_events(bool)` - Enable/disable disconnection event notifications
- `event(extern "C" fn())` - Register callback for queue state changes
- `exec()` - Execute builder and create Monitor

#### Client methods
- `monitor(pv_name)` - Create simple monitor
- `monitor_builder(pv_name)` - Create builder for advanced configuration

**Callback Pattern**: Callbacks fire when the monitor queue goes from empty to not-empty. You must drain the queue with `pop()` to reset the state and enable future callbacks.

### 4. New Example (monitor_example.rs)
Created comprehensive example demonstrating:
- Server with changing PVs (counter incrementing, sine wave)
- Simple monitor interface
- MonitorBuilder with callbacks
- Proper queue draining pattern
- Connection status checking

Run with:
```bash
cargo run --example monitor_example --features="client,server"
```

## API Usage Examples

### Enum Access
```rust
let value = client.get("MY:ENUM:PV", 5.0)?;
let enum_index = value.as_enum()?; // Returns i16 index
println!("Enum state: {}", enum_index);
```

### Array Access  
```rust
let value = client.get("MY:DOUBLE:ARRAY", 5.0)?;
let array = value.as_double_array()?; // Returns Vec<f64>
println!("Array: {:?}", array);

let int_array = value.as_int_array()?; // Vec<i32>
let string_array = value.as_string_array()?; // Vec<String>
```

### Simple Monitor
```rust
let mut monitor = client.monitor("MY:PV")?;
monitor.start();

while let Ok(Some(value)) = monitor.pop() {
    println!("Update: {}", value);
}

monitor.stop();
```

### Monitor with Callback
```rust
extern "C" fn my_callback() {
    println!("New data available!");
}

let mut monitor = client.monitor_builder("MY:PV")?
    .connection_events(false)
    .event(my_callback)
    .exec()?;

monitor.start();

// Drain queue after callback fires
while let Ok(Some(value)) = monitor.pop() {
    println!("Value: {}", value);
}
```

## Implementation Notes

### From Test Analysis
The implementation was guided by the epics-pvxs-sys unit tests:

1. **test_pvxs_local_enum_array_fetch_post.rs**: Showed enum values are i16 indices accessed via get_field_enum/get_field_enum_array. Server-side uses int32 PVs due to lack of create_pv_enum.

2. **test_pvxs_local_double_array_fetch_post.rs**: Demonstrated get_field_double_array pattern for reading arrays. PVs created with scalar methods (create_pv_double) can be read as arrays if server implements array support.

3. **test_pvxs_monitor_builder.rs**: Showed MonitorBuilder pattern with mask_connected/mask_disconnected, event callback registration, pop() for queue draining, and proper callback semantics (empty→not-empty transitions).

### API Limitations
Current limitations based on upstream epics-pvxs-sys:

1. **No put_string**: Context.put_string not implemented upstream, only put_double and put_int32 available
2. **No enum PV creation**: Server.create_pv_enum doesn't exist, use int32 PVs as backing store
3. **No array posting**: SharedPV doesn't support post_double_array etc., only scalar post operations
4. **No string PV from_env**: SharedPV.open_string works but may have limitations with from_env servers

## Compilation Status
✅ All changes compile successfully:
```bash
cargo build --features="client,server"
cargo build --example monitor_example --features="client,server"
```

## Files Modified
- `src/types.rs` - Added enum and array accessor methods to Value
- `src/client.rs` - Added Monitor, MonitorBuilder wrappers and Client methods
- `src/lib.rs` - Export Monitor and MonitorBuilder
- `examples/monitor_example.rs` - Created comprehensive monitor example

## Testing Recommendations
When testing the new features:

1. **Enum Testing**: Test enum PVs created via int32 backing, verify get_field_enum returns correct i16 values
2. **Array Testing**: Verify get_field_*_array methods work with both scalar and array PVs
3. **Monitor Testing**: Test callback firing with proper queue draining pattern
4. **Connection Events**: Test mask_connected/mask_disconnected with server start/stop cycles

## Next Steps (Future Enhancements)
Potential future additions when upstream supports them:

1. Add Context.put_string when available
2. Add Server.create_pv_enum if/when implemented
3. Add array put operations if SharedPV gains post_*_array methods
4. Consider async monitor interface with tokio streams
5. Add monitor filtering/throttling options if supported upstream
