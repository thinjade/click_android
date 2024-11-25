fn main() {
    slint_build::compile("ui.slint").unwrap();
    shadow_rs::new().unwrap();
}
