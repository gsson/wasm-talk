extern crate wasm_bindgen;
extern crate web_sys;
extern crate wee_alloc;

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Float32Array, Math};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    KeyboardEvent, MouseEvent, UiEvent, WebGlProgram, WebGlRenderingContext, WebGlShader,
    WebGlUniformLocation, WheelEvent,
};

use crate::stats::FrameStats;
use crate::util::{canvas, document, fps_div, gl, performance, request_animation_frame, window};

mod stats;
mod util;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const TRIANGLES: [f32; 12] = [
    1.0, 1.0,
    -1.0, 1.0,
    -1.0, -1.0,
    -1.0, -1.0,
    1.0, -1.0,
    1.0, 1.0
];

fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error compiling shader")))
    }
}

fn create_program(gl: &WebGlRenderingContext) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    let vertex_shader = compile_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        include_str!("vertices.vert"),
    )?;

    let frag_shader = compile_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        include_str!("mandelbrot64.frag"),
    )?;

    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &frag_shader);

    gl.link_program(&program);

    gl.delete_shader(Some(&vertex_shader));
    gl.delete_shader(Some(&frag_shader));

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program object".into()))
    }
}

fn split(v: f64) -> (f32, f32) {
    let high = Math::fround(v);
    let error = v - f64::from(high);
    (high as f32, error as f32)
}

trait WebGlRenderingContextAdaptation {
    fn require_uniform_location(&self, program: &WebGlProgram, name: &str) -> WebGlUniformLocation;
    fn uniform2f_from_f64(&self, location: &WebGlUniformLocation, value: f64);
    fn buffer_data_from_f64_slice(&self, target: u32, data: &[f32], usage: u32);
}

impl WebGlRenderingContextAdaptation for WebGlRenderingContext {
    fn require_uniform_location(&self, program: &WebGlProgram, name: &str) -> WebGlUniformLocation {
        self.get_uniform_location(program, name)
            .unwrap_or_else(|| panic!("missing uniform '{:}'", name))
    }

    fn uniform2f_from_f64(&self, location: &WebGlUniformLocation, value: f64) {
        let (h, l) = split(value);
        self.uniform2f(Some(&location), h, l);
    }

    fn buffer_data_from_f64_slice(&self, target: u32, data: &[f32], usage: u32) {
        unsafe {
            self.buffer_data_with_array_buffer_view(target, &Float32Array::view(data), usage);
        }
    }
}

#[derive(Copy, Clone)]
enum Animate {
    Stopped,
    ZoomTowards(f64, f64),
    ZoomOut(f64, f64),
    Move { t0: f64, p0: Position, p1: Position },
}

#[derive(Copy, Clone)]
struct Position {
    x: f64,
    y: f64,
    zoom: f64,
}

#[derive(Copy, Clone)]
struct MandelbrotState {
    position: Position,
    animate: Animate,
    time: f64,
}

#[derive(Copy, Clone)]
struct ScreenState {
    client_width: i32,
    client_height: i32,
}

fn clamp_zoom(zoom: f64) -> f64 {
    if zoom > 314_419_530_579_829.0 {
        314_419_530_579_829.0
    } else if zoom < 100.0 {
        100.0
    } else {
        zoom
    }
}

fn install_resize_handler(screen_ref: &Rc<RefCell<ScreenState>>) {
    let screen_ref = screen_ref.clone();
    let closure = Closure::wrap(Box::new(move |_: UiEvent| {
        let canvas_ = canvas().unwrap();
        let client_width = canvas_.client_width();
        let client_height = canvas_.client_height();

        screen_ref.replace(ScreenState {
            client_width,
            client_height,
        });
    }) as Box<dyn FnMut(_)>);

    window().set_onresize(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}

fn install_contextmenu_handler() {
    let closure = Closure::wrap(Box::new(move |e: MouseEvent| {
        e.prevent_default();
    }) as Box<dyn FnMut(_)>);

    window().set_oncontextmenu(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}

fn install_keyboard_handler(state_ref: &Rc<RefCell<MandelbrotState>>) {
    let state_ref = state_ref.clone();

    let closure = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        if e.key() == " " {
            let state = { *state_ref.borrow() };
            let p0 = state.position;
            let p1 = Position {
                x: 0.5,
                y: 0.0,
                zoom: 150.0,
            };
            let t0 = state.time;

            state_ref.replace(MandelbrotState {
                position: p0,
                animate: Animate::Move { t0, p0, p1 },
                time: t0,
            });
        } else if e.key() == "f" {
            if document().fullscreen_element().is_some() {
                document().exit_fullscreen();
            } else {
                canvas().unwrap().request_fullscreen().unwrap();
            }
        }
    }) as Box<dyn FnMut(_)>);

    window().set_onkeyup(Some(closure.as_ref().unchecked_ref()));

    closure.forget();
}

fn install_wheel_handler(state_ref: &Rc<RefCell<MandelbrotState>>) {
    let state_ref = state_ref.clone();
    let closure = Closure::wrap(Box::new(move |e: WheelEvent| {
        e.prevent_default();
        let state: MandelbrotState = { *state_ref.borrow() };
        if e.ctrl_key() {
            let mut position = state.position;
            position.zoom = clamp_zoom(position.zoom - e.delta_y() * position.zoom / 200.0);

            state_ref.replace(MandelbrotState {
                position,
                animate: state.animate,
                time: state.time,
            });
        } else {
            let position = Position {
                x: state.position.x + e.delta_x() / state.position.zoom,
                y: state.position.y - e.delta_y() / state.position.zoom,
                zoom: state.position.zoom,
            };

            state_ref.replace(MandelbrotState {
                position,
                animate: state.animate,
                time: state.time,
            });
        }
    }) as Box<dyn FnMut(_)>);

    window().set_onwheel(Some(closure.as_ref().unchecked_ref()));

    closure.forget();
}

fn install_mouse_handler(
    screen_ref: &Rc<RefCell<ScreenState>>,
    state_ref: &Rc<RefCell<MandelbrotState>>,
) {
    let state_ref = state_ref.clone();
    let screen_ref = screen_ref.clone();

    let closure = Closure::wrap(Box::new(move |e: MouseEvent| {
        let screen_state = { *screen_ref.borrow() };
        let state = { *state_ref.borrow() };

        let x = f64::from(e.offset_x() - screen_state.client_width / 2) / state.position.zoom;
        let y = f64::from(screen_state.client_height / 2 - e.offset_y()) / state.position.zoom;

        e.prevent_default();
        match e.type_().as_ref() {
            "mousedown" => {
                let animate = if e.button() == 0 {
                    Animate::ZoomTowards(x, y)
                } else {
                    Animate::ZoomOut(x, y)
                };
                state_ref.replace(MandelbrotState {
                    position: state.position,
                    animate,
                    time: state.time,
                });
            }
            "mouseup" => {
                state_ref.replace(MandelbrotState {
                    position: state.position,
                    animate: Animate::Stopped,
                    time: state.time,
                });
            }
            "mousemove" => match state.animate {
                Animate::Stopped => {}
                Animate::Move { .. } => {}
                Animate::ZoomTowards(_, _) => {
                    state_ref.replace(MandelbrotState {
                        position: state.position,
                        animate: Animate::ZoomTowards(x, y),
                        time: state.time,
                    });
                }
                Animate::ZoomOut(_, _) => {
                    state_ref.replace(MandelbrotState {
                        position: state.position,
                        animate: Animate::ZoomOut(x, y),
                        time: state.time,
                    });
                }
            },
            _ => (),
        }
    }) as Box<dyn FnMut(_)>);

    window().set_onmousedown(Some(closure.as_ref().unchecked_ref()));
    window().set_onmouseup(Some(closure.as_ref().unchecked_ref()));
    window().set_onmousemove(Some(closure.as_ref().unchecked_ref()));

    closure.forget();
}

fn pow_in(t: f64, p: f64) -> f64 {
    Math::pow(t, p)
}

fn pow_out(t: f64, p: f64) -> f64 {
    1.0 - Math::pow(1.0 - t, p)
}

fn interpolate(t: f64, t0: f64, p0: Position, p1: Position) -> Option<Position> {
    let p = Math::log(Math::abs(p1.zoom - p0.zoom) + 1.0);
    let t = (t - t0) / (p * 150.0);
    if t < 0.0 || t > 1.0 {
        None
    } else {
        let (d, zd) = if p1.zoom < p0.zoom {
            (pow_in(t, p / 2.0), pow_out(t, p))
        } else {
            (pow_out(t, p / 2.0), pow_in(t, p))
        };
        Some(Position {
            x: p0.x + (p1.x - p0.x) * d,
            y: p0.y + (p1.y - p0.y) * d,
            zoom: p0.zoom + (p1.zoom - p0.zoom) * zd,
        })
    }
}

fn render_stuff() -> Result<(), JsValue> {
    let canvas_ = canvas()?;

    let client_width = canvas_.client_width();
    let client_height = canvas_.client_height();

    let width = canvas_.width() as f32;
    let height = canvas_.height() as f32;

    let performance = performance();
    let gl = gl()?;
    let fps_div = fps_div()?;

    let mut stats = FrameStats::new(performance.now());

    let screen_ref = Rc::new(RefCell::new(ScreenState {
        client_width,
        client_height,
    }));

    let state_ref = Rc::new(RefCell::new(MandelbrotState {
        position: Position {
            x: 0.5,
            y: 0.0,
            zoom: 150.0,
        },
        animate: Animate::Stopped,
        time: performance.now(),
    }));

    install_resize_handler(&screen_ref);
    install_keyboard_handler(&state_ref);
    install_contextmenu_handler();
    install_wheel_handler(&state_ref);
    install_mouse_handler(&screen_ref, &state_ref);

    let program = create_program(&gl)?;

    let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
    gl.buffer_data_from_f64_slice(
        WebGlRenderingContext::ARRAY_BUFFER,
        &TRIANGLES,
        WebGlRenderingContext::STATIC_DRAW,
    );

    gl.vertex_attrib_pointer_with_i32(0, 2, WebGlRenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);

    gl.use_program(Some(&program));

    let uni_size = gl.require_uniform_location(&program, "size");
    let uni_offset_x = gl.require_uniform_location(&program, "offset_x");
    let uni_offset_y = gl.require_uniform_location(&program, "offset_y");
    let uni_zoom = gl.require_uniform_location(&program, "zoom");
    let uni_one = gl.require_uniform_location(&program, "ONE");

    gl.uniform2f(Some(&uni_size), width / 2.0, height / 2.0);
    gl.uniform1f(Some(&uni_one), 1.0);

    let f: Rc<RefCell<Option<_>>> = Rc::new(RefCell::new(None));
    let g: Rc<RefCell<Option<_>>> = f.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        let timestamp = performance.now();
        stats.frame(timestamp, |frame, fps| {
            fps_div.set_inner_text(&format!("{:} frames, {:.1} fps", frame, fps));
        });
        let mut state: MandelbrotState = { *state_ref.borrow() };

        match state.animate {
            Animate::Stopped => (),
            Animate::ZoomTowards(x, y) => {
                let new_zoom = clamp_zoom(state.position.zoom * 1.02);
                let dx = 0.02 * x;
                let dy = 0.02 * y;
                state.position.x -= dx;
                state.position.y -= dy;
                state.position.zoom = new_zoom;
            }
            Animate::ZoomOut(x, y) => {
                let new_zoom = clamp_zoom(state.position.zoom / 1.02);
                let dx = 0.02 * x;
                let dy = 0.02 * y;
                state.position.x += dx;
                state.position.y += dy;
                state.position.zoom = new_zoom;
            }
            Animate::Move { t0, p0, p1 } => {
                if let Some(p) = interpolate(timestamp, t0, p0, p1) {
                    state.position = p;
                } else {
                    state.position = p1;
                    state.animate = Animate::Stopped;
                }
            }
        }

        gl.uniform2f_from_f64(&uni_offset_x, state.position.x);
        gl.uniform2f_from_f64(&uni_offset_y, state.position.y);
        gl.uniform2f_from_f64(&uni_zoom, state.position.zoom);

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (TRIANGLES.len() / 2) as i32,
        );

        state_ref.replace(MandelbrotState {
            position: state.position,
            animate: state.animate,
            time: timestamp,
        });

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    render_stuff()
}
