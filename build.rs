use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // --- C Source Paths ---
    let can2040_src_dir = manifest_dir.join("can2040/src");
    let header_path = can2040_src_dir.join("can2040.h");
    let source_path = can2040_src_dir.join("can2040.c");

    // --- YOUR FULL LIST OF SDK AND GENERATED INCLUDE PATHS ---
    let pico_sdk_dir = manifest_dir.join("pico-sdk");
    let pico_sdk_include_paths = [
        manifest_dir.join("generated_headers"),
        pico_sdk_dir.join("src/rp2040/pico_platform/include"),
        pico_sdk_dir.join("bazel/include"),
        pico_sdk_dir.join("src/common/pico_base_headers/include"),
        pico_sdk_dir.join("src/common/pico_base/include"),
        pico_sdk_dir.join("src/rp2040/hardware_regs/include"),
        pico_sdk_dir.join("src/rp2_common/hardware_irq/include"),
        pico_sdk_dir.join("src/rp2_common/hardware_resets/include"),
        pico_sdk_dir.join("src/rp2_common/hardware_clocks/include"),
        pico_sdk_dir.join("src/rp2_common/hardware_pio/include"),
        pico_sdk_dir.join("src/rp2_common/pico_platform/include"),
        pico_sdk_dir.join("src/boards/include"),
        pico_sdk_dir.join("src/rp2040/hardware_structs/include"),
        pico_sdk_dir.join("src/rp2_common/hardware_dma/include"),
        pico_sdk_dir.join("src/rp2_common/hardware_base/include"),
        pico_sdk_dir.join("src/rp2_common/pico_platform_compiler/include"),
        pico_sdk_dir.join("src/rp2_common/pico_platform_sections/include"),
        pico_sdk_dir.join("src/rp2_common/pico_platform_panic/include"),
        pico_sdk_dir.join("src/rp2_common/pico_platform_common/include"),
    ];

    // --- 1. Build the C source ---
    let mut build = cc::Build::new();

    build.compiler("arm-none-eabi-gcc");
    build
        .file(&source_path)
        .includes(&pico_sdk_include_paths)
        .include(&can2040_src_dir)
        .flag("-Wno-unused-parameter")
        .flag("-Wno-sign-compare")
        .flag("-Wno-missing-field-initializers")
        .define("PICO_RP2040_CHECK_BSS_PC", "0")
        // THIS IS THE FIX. We are now explicitly telling the preprocessor
        // that we are building for an RP2040. This is what the official
        // CMake build system does, and it resolves the header conflict.
        .define("PICO_RP2040", "1");

    build.compile("can2040");

    // --- 2. Generate Rust bindings ---
    // (This part remains the same)
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .clang_arg("--target=thumbv6m-none-eabi")
        .clang_args(pico_sdk_include_paths.iter().map(|p| format!("-I{}", p.display())))
        .use_core()
        .ctypes_prefix("::core::ffi")
        .header(header_path.to_str().unwrap())
        .derive_debug(true)
        .blocklist_function("__wfe")
        .blocklist_function("__sev")
        .blocklist_function("__nop")
        .blocklist_function("__dmb")
        .blocklist_function("__clz")
        .generate()
        .expect("Failed to generate bindings");

    let bindings_path = out_dir.join("can2040_bindings.rs");
    bindings.write_to_file(&bindings_path).expect("Failed to write bindings");

    println!("cargo:rerun-if-changed={}", header_path.display());
    println!("cargo:rerun-if-changed={}", source_path.display());
    for path in &pico_sdk_include_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
