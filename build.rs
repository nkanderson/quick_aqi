// Taken from embassy stm32f3 examples:
// https://github.com/embassy-rs/embassy/blob/main/examples/stm32f3/build.rs
fn main() {
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
