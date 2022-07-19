use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

fn put_pixel(x: u32, y: u32, col: [u8; 4], frame: &mut [u8]) {
    let pixelindex: u64 = (y as u64)*(WIDTH as u64) + (x as u64);
    for pixel in frame.chunks_exact_mut(4).nth(pixelindex as usize) {
	pixel.copy_from_slice(&col);
    }
}

fn draw(frame: &mut [u8]) {
    for i in 0..WIDTH {
	for j in 0..HEIGHT{
	    put_pixel(i, j, [0x80u8+(((i as f32).cos()*100.0) as u8), 0x80u8+(((j as f32).sin()*100.0) as u8), 0x0u8, 0xffu8], frame);
	}
	println!("c{}", i)
    }
    println!("\njob done");
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

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
	    draw(pixels.get_frame());
	    
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
	}
    });
}
