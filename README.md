# pvxs-rs
EPICS PVXS reimplementation in Rust.

## Migration guide from pvxs-sys

Switching an existing project from pvxs-sys to pvxs-rs is usually a dependency change plus a small environment cleanup:

1. Update the dependency in Cargo.toml:
   ```toml
   [dependencies]
   pvxs = "0.1"
   ```
   Remove any `pvxs-sys` dependency and related `cxx` or native build glue.
2. Remove the old EPICS build assumptions from your environment:
   - delete `EPICS_BASE`-style configuration if it is only used for pvxs-sys
   - remove any C++ toolchain requirement from your build script or CI pipeline
3. Build and test normally with Cargo:
   ```bash
   cargo check
   cargo test
   ```

The crate aims to keep the same public API surface as pvxs-sys, so most application code can remain unchanged once the dependency swap is complete.
