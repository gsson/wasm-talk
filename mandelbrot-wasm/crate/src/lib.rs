use std::mem;
use std::os::raw::c_void;
use std::slice;

const ESCAPE: f64 = 4.0;
const BYTES_PER_PIXEL: usize = 4;

#[no_mangle]
pub unsafe extern "C" fn createImageBuffer(width: usize, height: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(width * height * BYTES_PER_PIXEL);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr as *mut c_void
}

#[no_mangle]
pub unsafe fn mandelbrot(
    pointer: *mut u8,
    width: i32,
    height: i32,
    max_iterations: usize,
    pos_x: f64,
    pos_y: f64,
    zoom: f64,
) {
    let elements = (width * height * 4) as usize;
    let slice = slice::from_raw_parts_mut(pointer, elements);
    mandelbrot_impl(slice, width, height, max_iterations, pos_x, pos_y, zoom)
}

fn mandelbrot_impl(
    slice: &mut [u8],
    width: i32,
    height: i32,
    max_iterations: usize,
    pos_x: f64,
    pos_y: f64,
    zoom: f64,
) {
    let f_width = f64::from(width);
    let f_height = f64::from(height);
    let aspect = f_height / f_width;
    let half_zoom = zoom / 2.0;
    let zoomed_height = half_zoom * f_height;
    let zoomed_width = half_zoom * f_width * aspect;
    let half_width = width / 2;
    let half_height = height / 2;
    let mut slice_index = 0;
    let max_iterations_squared = max_iterations * max_iterations;

    for y in -half_height..half_height {
        let y0 = f64::from(y) / zoomed_height + pos_y;

        for x in -half_width..half_width {
            let x0 = f64::from(x) / zoomed_width + pos_x;
            let iterations = iterate(x0, y0, max_iterations);

            let c = (iterations * iterations * 255 / max_iterations_squared) as u8;
            slice[slice_index] = c;
            slice[slice_index + 1] = c;
            slice[slice_index + 2] = c;
            slice[slice_index + 3] = 255;

            slice_index += 4;
        }
    }
}

fn iterate(x0: f64, y0: f64, max_iterations: usize) -> usize {
    let mut real = 0f64;
    let mut imaginary = 0f64;

    for i in 0..max_iterations {
        let real_squared = real * real;
        let imaginary_squared = imaginary * imaginary;

        if real_squared + imaginary_squared > ESCAPE {
            return i;
        }

        let real_temp = real_squared - imaginary_squared + x0;
        imaginary = 2.0 * real * imaginary + y0;
        real = real_temp;
    }
    max_iterations
}
