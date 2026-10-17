fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target == "macos" {
        println!("cargo:rerun-if-changed=native/macos.m");
        cc::Build::new()
            .file("native/macos.m")
            .flag("-fobjc-arc")
            .compile("floatleaf_native");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=ApplicationServices");
    } else if target == "windows" {
        println!("cargo:rerun-if-changed=native/windows.c");
        cc::Build::new()
            .file("native/windows.c")
            .compile("floatleaf_native");
        for lib in ["user32", "comctl32", "wtsapi32"] {
            println!("cargo:rustc-link-lib={lib}");
        }
    }
    tauri_build::build()
}
