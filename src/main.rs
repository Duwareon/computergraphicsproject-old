use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn put_pixel(x: u32, y: u32, col: [u8; 4], frame: &mut [u8]) {
    let pixelindex: u64 = (y as u64) * (WIDTH as u64) + (x as u64);
    for pixel in frame.chunks_exact_mut(4).nth(pixelindex as usize) {
        pixel.copy_from_slice(&col);
    }
}

/*fn map(f: impl Fn(i32) -> i32, i: i32, xoff: i32, yoff: i32, a: i32, b: i32) -> u32 {
    let x = f((i - xoff) / a) * b;
    let y = || -> i32 { yoff - x.max(0 - HEIGHT as i32 / 2).min(HEIGHT as i32 / 2) };
    return y() as u32;
}

fn heart(i: i32) -> i32 {
    let j = i as f32;
    return (j.abs().powf(2.0 / 3.0)
        + (8.0 - j.abs().powf(2.0)).abs().powf(1.0 / 2.0) * (16.0 * 3.1415926 * j).sin())
        as i32;
}*/

fn interpolate(i0: u32, d0: f32, i1: u32, d1: f32) -> Vec<f32> {
    if !(i0 == i1) {
        let mut values = Vec::new();
        let a = (d1 - d0) / (i1 - i0) as f32;
        let mut d = d0;
        for i in i0..i1 {
            values.push(d);
            d = d + a;
        }
        return values;
    } else {
        return vec![d0];
    }
}

fn draw_line(p0: [i32; 2], p1: [i32; 2], col: [u8; 4], frame: &mut [u8]) {
    let mut x0 = p0[0];
    let mut y0 = p0[1];
    let mut x1 = p1[0];
    let mut y1 = p1[1];

    println!("C");
    if (x1 - x0).abs() > (y1 - y0).abs() {
        println!("A");
        // Make sure x0 < x1
        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }
        let ys = interpolate(x0 as u32, y0 as f32, x1 as u32, y1 as f32);
        for x in x0..x1 {
            put_pixel(x as u32, ys[(x - x0) as usize] as u32, col, frame);
        }
    } else {
        println!("B");
        // Make sure y0 < y1
        if y0 > y1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }
        let xs = interpolate(y0 as u32, x0 as f32, y1 as u32, x1 as f32);
        for y in y0..y1 {
            put_pixel(xs[(y - y0) as usize] as u32, y as u32, col, frame);
        }
    }
}

fn draw_wire_triangle(p0: [i32; 2], p1: [i32; 2], p2: [i32; 2], col: [u8; 4], frame: &mut [u8]) {
    draw_line(p0, p1, col, frame);
    draw_line(p1, p2, col, frame);
    draw_line(p2, p0, col, frame);
}

fn clear(frame: &mut [u8]) {
    frame.iter_mut().for_each(|m| *m = 0);
}

fn draw(frame: &mut [u8], time: Instant) {
    //let now = Instant::now();
    //let elapsed = now.duration_since(time).subsec_millis();
    clear(frame);
    draw_wire_triangle([100, 100], [200, 100], [50, 400], [0xff; 4], frame);
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("pogging my pants")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    window.set_min_inner_size(Some(LogicalSize::new(WIDTH as f64, HEIGHT as f64)));
    window.set_max_inner_size(Some(LogicalSize::new(WIDTH as f64, HEIGHT as f64)));

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let time: Instant = Instant::now();

    //let mut frametime: Instant = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            /*println!(
                "{}",
                Instant::now().duration_since(frametime).subsec_millis()
            );
            frametime = Instant::now();*/
            draw(pixels.get_frame(), time);

            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            //window.request_redraw();
        }
    });
}
