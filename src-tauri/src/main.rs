#[cfg(not(target_os = "macos"))]
compile_error!("Sentence Binder supports macOS-only.");

fn main() {
    sentence_binder_lib::run()
}
