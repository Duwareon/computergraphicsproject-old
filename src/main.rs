extern crate bitfont;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::mem::swap;
use std::time::{Duration, Instant};
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

fn draw_text(p: [u32; 2], text: &str, col: [u8; 4], frame: &mut [u8]) {
    let texts = text.split("\n");
    let mut xd = 0;
    let mut yd = 0;
    for text in texts {
        let bitvec = bitfont::bitmap_bool(text).unwrap();
        for row in bitvec {
            for charac in row {
                if charac {
                    put_pixel(p[0] + xd, p[1] + yd, col, frame);
                }
                xd += 1;
            }
            yd += 1;
            xd = 0;
        }
        yd += 1;
    }
}

fn interpolate(i0: u32, d0: f32, i1: u32, d1: f32) -> Vec<f32> {
    if i0 == i1 {
        return vec![d0];
    }
    let mut values = Vec::new();
    let a = (d1 - d0) / (i1 - i0) as f32;
    let mut d = d0;
    for _ in i0..i1 {
        values.push(d);
        d = d + a;
    }
    return values;
}

fn draw_line(p0: [i32; 2], p1: [i32; 2], col: [u8; 4], frame: &mut [u8]) {
    let mut x0 = p0[0];
    let mut y0 = p0[1];
    let mut x1 = p1[0];
    let mut y1 = p1[1];

    if (x1 - x0).abs() > (y1 - y0).abs() {
        // Make sure x0 < x1
        if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }
        let ys = interpolate(x0 as u32, y0 as f32, x1 as u32, y1 as f32);
        for x in x0..x1 {
            put_pixel(x as u32, ys[(x - x0) as usize] as u32, col, frame);
        }
    } else {
        // Make sure y0 < y1
        if y0 > y1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
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

fn draw_filled_triangle(p0: [i32; 2], p1: [i32; 2], p2: [i32; 2], col: [u8; 4], frame: &mut [u8]) {
    let mut p0 = p0;
    let mut p1 = p1;
    let mut p2 = p2;
    if p1[1] < p0[1] {
        swap(&mut p1, &mut p0)
    }
    if p2[1] < p0[1] {
        swap(&mut p2, &mut p0)
    }
    if p2[1] < p1[1] {
        swap(&mut p2, &mut p1)
    }

    let mut x012 = interpolate(p0[1] as u32, p0[0] as f32, p1[1] as u32, p1[0] as f32);
    x012.pop();
    let mut x12 = interpolate(p1[1] as u32, p1[0] as f32, p2[1] as u32, p2[0] as f32);
    let x02 = interpolate(p0[1] as u32, p0[0] as f32, p2[1] as u32, p2[0] as f32);
    //x01.pop();

    x012.append(&mut x12);

    let m = (x012.len() as f32 / 2.0).floor() as usize;

    let x_left: Vec<f32>;
    let x_right: Vec<f32>;

    if x02[m] < x012[m] {
        x_left = x02;
        x_right = x012;
    } else {
        x_left = x012;
        x_right = x02;
    }

    for y in p0[1]..p2[1] - 1 {
        for x in
            (x_left[(y - p0[1]) as usize] as i32)..(x_right[(y - p0[1]) as usize].floor() as i32)
        {
            put_pixel(x as u32, y as u32, col, frame);
        }
    }
}

fn clear(col: [u8; 4], frame: &mut [u8]) {
    for (_, pixel) in frame.chunks_exact_mut(4).enumerate() {
        pixel.copy_from_slice(col.as_slice());
    }
}

fn draw_test(frame: &mut [u8], time: Duration, timesincelastframe: Duration) {
    draw_filled_triangle(
        [100, 125],
        [200, 100],
        [150, 400],
        [0xff, 0x60, 0x4f, 0xff],
        frame,
    );
    draw_filled_triangle(
        [125, 50],
        [20, 70],
        [120, 440],
        [0x00, 0x80, 0x8f, 0xff],
        frame,
    );
    draw_filled_triangle(
        [200, 225],
        [300, 200],
        [250, 300],
        [0x00, 0x70, 0x00, 0xff],
        frame,
    );
    draw_wire_triangle(
        [200, 225],
        [300, 200],
        [250, 300],
        [0xff, 0xff, 0xff, 0xff],
        frame,
    );

    draw_wire_triangle(
        [400, 400],
        [450, 80],
        [500, 420],
        [0xa0, 0xb0, 0x00, 0xff],
        frame,
    );

    draw_line([410, 450], [490, 70], [0x40, 0x17, 0xc0, 0xff], frame);

    draw_text(
        [200, 270],
        "poggers\npogchamp",
        [0xff, 0x00, 0xff, 0xff],
        frame,
    );
}

fn draw(frame: &mut [u8], time: Duration, timesincelastframe: Duration) {
    clear([0x00u8; 4], frame);
    draw_test(frame, time, timesincelastframe);

    // Draw debug text for frame time
    draw_text(
        [50, 50],
        &((timesincelastframe.as_nanos() / 10000).to_string()),
        [0xff; 4],
        frame,
    );
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

    let mut frametime: Instant = Instant::now();
    let starttime: Instant = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let now = Instant::now();
            draw(
                pixels.get_frame(),
                now.duration_since(starttime),
                now.duration_since(frametime),
            );
            frametime = Instant::now();

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

            window.request_redraw();
        }
    });
}
