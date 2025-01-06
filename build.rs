use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Debug: Notify that build.rs has started
    println!("TZ: Running build.rs");
    // Define the paths to PVXS headers and libraries
    let pvxs_lib_dir = "third_party/lib";
    let pvxs_dll_name = "pvxs.dll"; // Windows-specific
    let com_dll_name = "Com.dll";   // Windows-specific
    let event_core_dll_name = "event_core.dll"; // Windows-specific

    // Link PVXS library
    println!("cargo:rustc-link-search=native={}", pvxs_lib_dir);
    println!("cargo:rustc-link-lib=dylib=pvxs"); // Dynamic linking with pvxs.dll on Windows

    println!("TZ:Linking set up for PVXS library");

    // Configure cxx-build to compile the C++ bridge
    cxx_build::bridge("src/bindings.rs")
        .file("src-cxx/pvxs_wrapper.cpp")
        .include("include")          // Path to pvxs_wrapper.h
        .include("third_party/pvxs/include") // Path to original PVXS headers
        .include("third_party/epics/include") // Path to EPICS headers
        .flag_if_supported("-std=c++17") // Use C++17 standard
        .flag_if_supported("/std:c++17") // MSVC compatibility
        .compile("pvxs_rs");   // Output library name

    // Copy pvxs.dll, Com.dll and event_core.dll to the target directory (Windows only)
    if cfg!(target_os = "windows") {
        copy_dll_to_target(pvxs_lib_dir, pvxs_dll_name);
        copy_dll_to_target(pvxs_lib_dir, com_dll_name);
        copy_dll_to_target(pvxs_lib_dir, event_core_dll_name);
    }else {
        // On Linux, we need to link with the shared libraries
        //println!("cargo:rustc-link-lib=dylib=Com");
        //println!("cargo:rustc-link-lib=dylib=event_core");
        println!("Linux not yet implemented");
    }

    // Ensure Cargo rebuilds if any pf the DLLs change
    println!("cargo:rerun-if-changed={}", Path::new(pvxs_lib_dir).join(pvxs_dll_name).display());
    println!("cargo:rerun-if-changed={}", Path::new(pvxs_lib_dir).join(com_dll_name).display());
    println!("cargo:rerun-if-changed={}", Path::new(pvxs_lib_dir).join(event_core_dll_name).display());
    println!("cargo:rerun-if-changed=src-cxx/pvxs_wrapper.cpp");
    println!("cargo:rerun-if-changed=include/pvxs_wrapper.h");
    println!("cargo:rerun-if-changed=src/bindings.rs");
}

/// Copies the specified DLL to the `target/debug` or `target/release` directory.
fn copy_dll_to_target(lib_dir: &str, dll_name: &str) {
    // Get the output directory where the Rust binary will be located
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("OUT_DIR: {}", out_dir);
    let target_dir = PathBuf::from(out_dir)
        .ancestors()
        .nth(3) // Navigate back to the top-level `target/<profile>` directory
        .unwrap()
        .to_path_buf();

    // Define source and destination paths
    let dll_src = Path::new(lib_dir).join(dll_name);
    let dll_dst = target_dir.join(dll_name);

    // Copy the DLL
    fs::copy(&dll_src, &dll_dst).expect(&format!("Failed to copy '{}' to the target directory",dll_name));
    println!(
        "Successfully Copied {} to {}",
        dll_src.display(),
        dll_dst.display()
    );
}
