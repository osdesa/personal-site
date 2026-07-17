#[cfg(target_arch = "wasm32")]
use personal_site::app::App;

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // The website executable runs in the browser. Keeping a native entry point
    // allows the complete package to be checked without implying a server.
}
