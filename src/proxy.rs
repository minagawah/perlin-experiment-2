use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;

use crate::canvas::Canvas;
use crate::utils::{get_canvas, request_animation_frame_future, timer};

const REFRESH_RATE: i32 = 60;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub bgcolor: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct Proxy {
    pub canvas: Rc<RefCell<Canvas>>,
}

#[allow(clippy::await_holding_refcell_ref)]
impl Proxy {
    pub fn new(params: &JsValue) -> Self {
        let config: Config =
            serde_wasm_bindgen::from_value(params.clone()).unwrap();

        let bgcolor: String = config.bgcolor.clone();
        let color: String = config.color;

        let element = get_canvas("#perlin-experiment").unwrap();
        let canvas =
            Rc::new(RefCell::new(Canvas::new(element, bgcolor, color)));

        canvas.borrow_mut().register_listeners();
        canvas.borrow_mut().update_size();

        Proxy { canvas }
    }

    pub async fn run(&mut self) {
        loop {
            timer(REFRESH_RATE).await.unwrap();
            self.canvas.borrow_mut().update();
            self.canvas.borrow_mut().draw();
            request_animation_frame_future().await;
        }
    }
}
