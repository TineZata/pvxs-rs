use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Debug: Notify that build.rs has started
    eprintln!("TZ: Running build.rs");

    // Define the paths to PVXS headers and libraries
    let pvxs_lib_dir = "third_party/lib";
    let pvxs_dll_name = "pvxs.dll"; // Windows-specific
    let com_dll_name = "Com.dll";   // Windows-specific
    let event_core_dll_name = "event_core.dll"; // Windows-specific

    if cfg!(target_os = "windows") {
        // Copy DLLs for Windows
        copy_dll_to_target(pvxs_lib_dir, pvxs_dll_name);
        copy_dll_to_target(pvxs_lib_dir, com_dll_name);
        copy_dll_to_target(pvxs_lib_dir, event_core_dll_name);
    } else if cfg!(target_os = "linux") {
        // Add Linux-specific linking or shared library handling
        println!("cargo:rustc-link-search=native={}", pvxs_lib_dir);
        println!("cargo:rustc-link-lib=dylib=pvxs");
        println!("cargo:rustc-link-lib=dylib=Com");
        println!("cargo:rustc-link-lib=dylib=event_core");
    } else {
        panic!("Unsupported platform");
    }
}

/// Copies the specified DLL to the `target/debug` or `target/release` directory.
fn copy_dll_to_target(lib_dir: &str, dll_name: &str) {
    // Get the output directory where the Rust binary will be located
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not found");
    println!("OUT_DIR: {}", out_dir);

    // Navigate back to the top-level `target/<profile>` directory
    let target_dir = PathBuf::from(out_dir)
        .ancestors()
        .nth(3)
        .expect("Failed to find target directory")
        .to_path_buf();

    // Define source and destination paths
    let dll_src = Path::new(lib_dir).join(dll_name);
    let dll_dst = target_dir.join(dll_name);

    // Debug: Print the paths being used
    println!("Source DLL: {}", dll_src.display());
    println!("Destination DLL: {}", dll_dst.display());

    // Verify source file exists
    if !dll_src.exists() {
        panic!("Source DLL does not exist: {}", dll_src.display());
    }

    // Copy the DLL
    fs::copy(&dll_src, &dll_dst)
        .unwrap_or_else(|err| panic!("Failed to copy '{}' to '{}': {}", dll_src.display(), dll_dst.display(), err));
    println!(
        "Successfully copied {} to {}",
        dll_src.display(),
        dll_dst.display()
    );
}
