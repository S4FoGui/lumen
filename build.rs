// Build script for Glaido
// whisper-rs handles its own whisper.cpp compilation via its build script,
// so we only need minimal setup here.

fn main() {
    // Tell cargo to rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    // whisper-rs crate handles whisper.cpp compilation internally.
    // If WHISPER_NO_METAL and WHISPER_NO_CUDA are not set,
    // it will try to link against GPU backends if available.
    // For Linux CPU-only builds, we ensure these are set:
    if std::env::var("WHISPER_NO_CUDA").is_err() {
        println!("cargo:rustc-env=WHISPER_NO_CUDA=1");
    }
}
