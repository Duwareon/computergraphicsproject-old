use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = u8::MAX as u32;
const HEIGHT: u32 = u8::MAX as u32;

fn put_pixel(x: u32, y: u32, col: [u8; 4], frame: &mut [u8]) {
    let pixelindex: u64 = (y as u64) * (WIDTH as u64) + (x as u64);
    for pixel in frame.chunks_exact_mut(4).nth(pixelindex as usize) {
        pixel.copy_from_slice(&col);
    }
}

fn draw(frame: &mut [u8], time: Instant) {
    let now = Instant::now();
    let elapsed = now.duration_since(time).subsec_millis();

    let f = |i: f32| -> f32 { i.sin() }; // + (elapsed as f32 / 160.0)).sin() };
    let f8 = |i: u8| -> u8 { ((f((i as f32) / 10.0) + 1.0) * 20.0) as u8 };

    frame.iter_mut().for_each(|m| *m = 0);

    for i in 0..WIDTH {
        //println!("f({}) = {}", i, f(i as f32));
        put_pixel(i, (0x80 - f8(i as u8)) as u32, [0xffu8; 4], frame);
    }

    //println!("{:?}", time.elapsed());
    /*for i in 0..WIDTH {
        for j in 0..HEIGHT {
            let r: u8 = i as u8;
            let g: u8 = j as u8;
            let b: u8 = 0xa0;
            let a: u8 = ((elapsed / 10) % 255) as u8;
            put_pixel(i, j, [r, g, b, a], frame);
        }
        //println!("c{}", i)
    }*/
    //println!("\njob done");
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

            window.request_redraw();
        }
    });
}
