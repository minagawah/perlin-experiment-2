use std::sync::Arc;
use tokio::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

use crate::proxy::Proxy;

#[wasm_bindgen]
pub struct App {
    proxy: Arc<Mutex<Proxy>>,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new(params: &JsValue) -> Result<App, JsValue> {
        Ok(App {
            proxy: Arc::new(Mutex::new(Proxy::new(params))),
        })
    }

    #[wasm_bindgen]
    pub fn start(&mut self) {
        let proxy = Arc::clone(&self.proxy);
        spawn_local(async move {
            let mut proxy = proxy.lock().await;
            proxy.run().await;
            drop(proxy); // release the lock before the await point
        });
    }
}
