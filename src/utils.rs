use futures::future::LocalBoxFuture;
use futures::FutureExt;
use num::{Float, NumCast};
use std::f64::consts::PI;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_timer::Instant;
use web_sys::{DomRect, HtmlCanvasElement};

pub fn exit(message: &str) {
    let v = JsValue::from_str(message);
    web_sys::console::log_1(&("panic".into()));
    web_sys::console::exception_1(&v);
    std::process::abort();
}

pub fn debounce<F>(
    mut func: F,
    duration: Duration,
) -> impl FnMut()
where
    F: FnMut(),
{
    let mut last_call_time = Instant::now();

    move || {
        let now = Instant::now();
        let elapsed = now
            .duration_since(last_call_time)
            .as_millis() as f64;
        let should_call =
            elapsed >= duration.as_millis() as f64;

        if should_call {
            func();
            last_call_time = now;
        }
    }
}

pub async fn timer(msec: i32) -> Result<(), JsValue> {
    let promise = js_sys::Promise::new(
        &mut |resolve, _| {
            get_window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve, msec,
            )
            .unwrap();
        },
    );
    let future =
        wasm_bindgen_futures::JsFuture::from(promise);
    future.await?;
    Ok(())
}

pub fn get_window() -> Result<web_sys::Window, String>
{
    web_sys::window()
        .ok_or_else(|| "No window".into())
}

pub fn get_document(
) -> Result<web_sys::Document, String> {
    get_window()?
        .document()
        .ok_or_else(|| "No document".into())
}

pub fn device_pixel_ratio() -> f64 {
    get_window()
        .map_or(1_f64, |w| w.device_pixel_ratio())
}

pub fn get_window_size() -> (f64, f64) {
    match get_window() {
        Ok(win) => (
            win.inner_width()
                .map_or(0_f64, f64_from_js),
            win.inner_height()
                .map_or(0_f64, f64_from_js),
        ),
        Err(_) => (0_f64, 0_f64),
    }
}

pub fn request_animation_frame(
    f: &Closure<dyn FnMut()>,
) {
    get_window()
        .unwrap()
        .request_animation_frame(
            f.as_ref().unchecked_ref(),
        )
        .expect(
            "Failed to start request_animation_frame",
        );
}

pub fn request_animation_frame_future(
) -> LocalBoxFuture<'static, ()> {
    let f = callback_future::CallbackFuture::new(
        |complete| {
            get_window()
                .expect("Should have window")
                .request_animation_frame(
                    Closure::once_into_js(
                        move || complete(()),
                    )
                    .as_ref()
                    .unchecked_ref(),
                )
                .expect(
                    "should register \
                     `requestAnimationFrame` OK",
                );
        },
    );
    f.boxed_local()
}

pub fn get_wrapper_element(
    name: &str,
) -> Result<web_sys::HtmlElement, String> {
    let elem: web_sys::Element = get_document()?
        .get_element_by_id(name)
        .ok_or(format!("No element: {}", name))?;

    Ok(elem
        .dyn_into::<web_sys::HtmlElement>()
        .map_err(|e| e.to_string())?)
}

pub fn get_canvas(
    id: &str,
) -> Result<web_sys::HtmlCanvasElement, String> {
    let canvas = get_document()?
        .query_selector(id)
        .map_err(|_| format!("No element: {}", id))?
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| {
            "Failed to get canvas".to_string()
        })?;
    Ok(canvas)
}

pub fn get_ctx(
    canvas: &web_sys::HtmlCanvasElement,
) -> Result<web_sys::CanvasRenderingContext2d, String>
{
    let ctx = canvas
        .get_context("2d")
        .map_err(|_| "Failed get 2D Context".to_string())?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    Ok(ctx)
}

pub fn get_canvas_size(
    el: &HtmlCanvasElement,
) -> (f64, f64) {
    let rect: DomRect = el.get_bounding_client_rect();
    (
        rect.right() - rect.left(),
        rect.bottom() - rect.top(),
    )
}

pub fn rad_to_deg(rad: f64) -> f64 {
    rad * (180.0 / PI)
}
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * (PI / 180.0)
}

pub fn fixed_decimals<F: Float>(
    value: F,
    digits: F,
) -> F {
    (value * digits).round() / digits
}

pub fn lazy_round<F: Float>(value: F) -> F {
    fixed_decimals(
        value,
        NumCast::from(100.00).unwrap(),
    )
}

/// Get the norm for `val` between `min` and `max`.
/// Ex. norm(75, 0, 100) ---> 0.75
pub fn norm(val: f64, min: f64, max: f64) -> f64 {
    (val - min) / (max - min)
}

/// Apply `norm` (the linear interpolate value) to the range
/// between `min` and `max` (usually between `0` and `1`).
/// Ex. lerp(0.5, 0, 100) ---> 50
pub fn lerp(norm: f64, min: f64, max: f64) -> f64 {
    min + (max - min) * norm
}

pub fn f64_from_js(js: JsValue) -> f64 {
    js.as_f64().unwrap_or_default()
}

pub fn f64_cmp(
    a: &f64,
    b: &f64,
) -> std::cmp::Ordering {
    a.partial_cmp(b).unwrap()
}

pub fn ease_in_out_quad(v: f64) -> f64 {
    if v < 0.5 {
        v * v * 2.0
    } else {
        v * (4.0 - v * 2.0) - 1.0
    }
}

pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn rgb_to_hex(rgb_color: &RgbColor) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        rgb_color.r, rgb_color.g, rgb_color.b
    )
}

pub fn hex_to_rgb(hex_color: &str) -> RgbColor {
    let hex_value = hex::decode(
        hex_color.trim_start_matches('#'),
    )
    .expect("Invalid hex code");
    match hex_value.as_slice() {
        [r, g, b] => RgbColor {
            r: *r,
            g: *g,
            b: *b,
        },
        _ => panic!("Invalid hex code"),
    }
}

pub fn color_change_intensity_rgb(
    rbg: &RgbColor,
    intensity: f64,
) -> RgbColor {
    let r = ((rbg.r as f64 * intensity) as i16)
        .max(0)
        .min(255) as u8;
    let g = ((rbg.g as f64 * intensity) as i16)
        .max(0)
        .min(255) as u8;
    let b = ((rbg.b as f64 * intensity) as i16)
        .max(0)
        .min(255) as u8;
    RgbColor { r, g, b }
}

pub fn color_change_intensity_hex(
    hex_color: &str,
    intensity: f64,
) -> String {
    let rgb = hex_to_rgb(hex_color);
    let new_rgb =
        color_change_intensity_rgb(&rgb, intensity);
    rgb_to_hex(&new_rgb)
}
