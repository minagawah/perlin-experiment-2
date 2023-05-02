pub mod app;
pub mod canvas;
pub mod proxy;
pub mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

// `wee_alloc` is a tiny allocator designed for WebAssembly
// that has a (pre-compression) code-size footprint of only
// a single kilobyte. When the `wee_alloc` feature is enabled,
// this uses `wee_alloc` as the global allocator. If you don't
// want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc =
    wee_alloc::WeeAlloc::INIT;

pub fn exit(message: &str) {
    let v = JsValue::from_str(message);
    web_sys::console::log_1(&("panic".into()));
    web_sys::console::exception_1(&v);
    std::process::abort();
}

#[wasm_bindgen(start)]
pub fn start() {
    console_log::init()
        .expect("console_log::init failed");
    console_error_panic_hook::set_once();

    #[cfg(debug_assertions)]
    web_sys::console::log_1(&JsValue::from_str(
        "debug",
    ));

    #[cfg(not(debug_assertions))]
    web_sys::console::log_1(&JsValue::from_str(
        "release",
    ));
}
