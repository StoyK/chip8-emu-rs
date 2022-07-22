use chip8_core::*;
use sdl2::{event::Event, pixels::Color, rect::Rect, render::Canvas, video::Window};
use std::{env, fs::File, io::Read};

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    // Setup SDL2
    let sdl_context = sdl2::init().expect("Unable to init SDL");
    let video_subsystem = sdl_context.video().expect("Unable to init video");
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .expect("Unable to build window");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("Unable to build canvas");

    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Unable to create event pump");

    let mut chip8 = Chip8::new();

    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut game = Vec::new();
    rom.read_to_end(&mut game).expect("Unable to read file");
    chip8.load(&game);

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. } => {
                    break 'gameloop;
                }
                _ => {}
            }
        }

        chip8.tick();
        draw_screen(&chip8, &mut canvas);
    }
}

fn draw_screen(chip8: &Chip8, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = chip8.get_display();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i % SCREEN_HEIGHT) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}
