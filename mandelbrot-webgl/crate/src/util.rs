use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    Document, HtmlCanvasElement, HtmlDivElement, Performance, WebGlRenderingContext, Window,
};

pub fn window() -> Window {
    web_sys::window().expect("Failed to get window global object")
}

pub fn performance() -> Performance {
    window()
        .performance()
        .expect("Failed to get performance object")
}

pub fn request_animation_frame(f: &Closure<FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("request_animation_frame failed");
}

pub fn document() -> Document {
    window()
        .document()
        .expect("should have a document on window")
}

pub fn canvas() -> Result<HtmlCanvasElement, JsValue> {
    document()
        .get_element_by_id("canvas")
        .expect("should have #canvas on the page")
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| "should be a HtmlCanvasElement".into())
}

pub fn fps_div() -> Result<HtmlDivElement, JsValue> {
    document()
        .get_element_by_id("fps")
        .expect("should have #fps on the page")
        .dyn_into::<HtmlDivElement>()
        .map_err(|_| "should be a HtmlDivElement".into())
}

pub fn gl() -> Result<WebGlRenderingContext, JsValue> {
    canvas()?
        .get_context("webgl")?
        .ok_or("canvas missing webgl context")?
        .dyn_into::<WebGlRenderingContext>()
        .map_err(|_| "should be a WebGlRenderingContext".into())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (util::log(&format_args!($($t)*).to_string()))
}
