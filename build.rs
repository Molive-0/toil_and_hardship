use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=shaders/shader.vert");
    println!("cargo:rerun-if-changed=shaders/shader.frag");
    let out;
    if cfg!(windows) {
        out = Command::new("glslc.exe")
            .args(&["shaders/shader.vert", "-Os", "-o", "shaders/vert.spv"])
            .output()
            .expect("failed to execute vert process");
    } else {
        out = Command::new("glslc")
            .args(&["shaders/shader.vert", "-Os", "-o", "shaders/vert.spv"])
            .output()
            .expect("failed to execute vert process");
    }
    println!("{}", String::from_utf8(out.stderr).unwrap());
    println!("{}", String::from_utf8(out.stdout).unwrap());
    let out;
    if cfg!(windows) {
        out = Command::new("glslc.exe")
            .args(&["shaders/shader.frag", "-Os", "-o", "shaders/frag.spv"])
            .output()
            .expect("failed to execute frag process");
    } else {
        out = Command::new("glslc")
            .args(&["shaders/shader.frag", "-Os", "-o", "shaders/frag.spv"])
            .output()
            .expect("failed to execute frag process");
    }
    println!("{}", String::from_utf8(out.stderr).unwrap());
    println!("{}", String::from_utf8(out.stdout).unwrap());
    println!(r"cargo:rustc-link-search=.");
}
