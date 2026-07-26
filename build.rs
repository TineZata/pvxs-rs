// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
//
// pvxs-rs is pure Rust -- no C++, no DLLs, no EPICS_BASE required.
// This build script is intentionally minimal.
fn main() {
    // Nothing to do: no C++ bridge, no native libraries to link.
    // TODO(network): when the pvAccess network layer is added, this will
    // configure any OS-specific socket options if needed.
    println!("cargo:rerun-if-changed=build.rs");
}
