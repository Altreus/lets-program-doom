extern crate sdl2;

use sdl2::rect::Point;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 120;
const PIXEL_SCALE: u32 = 4;
const FRAME_RATE: u32 = 20;

// TODO: This could be an analogue angle but we're not doing that for
// this toy project.
#[derive(Debug)]
enum MovementDirection {
    Forward,
    ForwardRight,
    Right,
    BackwardRight,
    Backward,
    BackwardLeft,
    Left,
    ForwardLeft
}

enum Rotation {
    Left, Right
}

#[derive(Default)]
struct PlayerMovement {
    pub movement: Option<MovementDirection>,
    pub rotation: Option<Rotation>
}

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    let window = video_subsystem
        .window("Game",
            (SCREEN_WIDTH * PIXEL_SCALE),
            (SCREEN_HEIGHT * PIXEL_SCALE))
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(SCREEN_WIDTH, SCREEN_HEIGHT).expect("Failed to set logical size");
    canvas.set_scale(PIXEL_SCALE as _, PIXEL_SCALE as _).expect("Failed to set scale");
    canvas.set_integer_scale(true).expect("Failed to set integer scale");
    canvas.set_draw_color(Color::WHITE);
    canvas.clear();
    canvas.present();

    // TODO: Expand this to a game context object
    let mut player_movement : PlayerMovement = Default::default();

    let mut event_pump = sdl.event_pump().unwrap();
    let mut lastframe = Instant::now();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'main,
                Event::KeyDown { keycode: Some(k), .. } => on_key_down(&mut player_movement, k),
                Event::KeyUp   { keycode: Some(k), .. } => on_key_up  (&mut player_movement, k),
                _ => {}
            }
        }

        move_player(&player_movement);
        // Draw frame_rate times per second
        if lastframe.elapsed().as_millis() >= 1000 / FRAME_RATE as u128 {
            draw_stuff(&mut canvas);

            canvas.present();
            lastframe = Instant::now();
        }
    }
}

fn move_player(p: &PlayerMovement) {
    match &p.movement {
        None => {}
        Some(x) => { println!("{:?}", x) }
    }
}

fn draw_stuff(canvas: &mut sdl2::render::WindowCanvas) {
    canvas.set_draw_color(Color::WHITE);
    canvas.clear();
}

fn draw_pixel(canvas: &mut sdl2::render::WindowCanvas, p: Point, c: Color) {
    canvas.set_draw_color(c);
    canvas.draw_point(p).expect("Error drawing pixel :(");
}

fn on_key_down(p: &mut PlayerMovement, k: Keycode) {
    // For each movement key, combine it with the existing movement.
    // If it contradicts, do nothing; I can't be bothered cancelling out. This
    // means we'll ignore a key that is held down if the contradictory key is
    // released but whatever
    match k {
        Keycode::W => {
            match p.movement {
                Some(MovementDirection::Left) => p.movement = Some(MovementDirection::ForwardLeft),
                Some(MovementDirection::Right) => p.movement = Some(MovementDirection::ForwardRight),
                None => p.movement = Some(MovementDirection::Forward),
                _ => {},
            }
        },
        Keycode::S => {
            match p.movement {
                Some(MovementDirection::Left) => p.movement = Some(MovementDirection::BackwardLeft),
                Some(MovementDirection::Right) => p.movement = Some(MovementDirection::BackwardRight),
                None => p.movement = Some(MovementDirection::Backward),
                _ => {},
            }
        },
        Keycode::A => {
            match p.movement {
                Some(MovementDirection::Forward) => p.movement = Some(MovementDirection::ForwardLeft),
                Some(MovementDirection::Backward) => p.movement = Some(MovementDirection::BackwardLeft),
                None => p.movement = Some(MovementDirection::Left),
                _ => {},
            }
        },
        Keycode::D => {
            match p.movement {
                Some(MovementDirection::Forward) => p.movement = Some(MovementDirection::ForwardRight),
                Some(MovementDirection::Backward) => p.movement = Some(MovementDirection::BackwardRight),
                None => p.movement = Some(MovementDirection::Right),
                _ => {},
            }
        },

        // Here we ignore the input if we're already rotating.
        Keycode::LEFT if !p.rotation.is_some() => p.rotation = Some(Rotation::Left),
        Keycode::RIGHT if !p.rotation.is_some() => p.rotation = Some(Rotation::Right),
        _ => {}
    }
}

fn on_key_up(p: &mut PlayerMovement, k: Keycode) {
    // For each movement key, if that key's job is taking place, untakeplace it
    // If that results in no keys, set None.
    // This really just swaps the operands to => above
    // If the contradictory key is still being pressed, oh well
    match k {
        Keycode::W => {
            match p.movement {
                Some(MovementDirection::ForwardLeft) => p.movement = Some(MovementDirection::Left),
                Some(MovementDirection::ForwardRight) => p.movement = Some(MovementDirection::Right),
                Some(MovementDirection::Forward) => p.movement = None,
                _ => {},
            }
        },
        Keycode::S => {
            match p.movement {
                Some(MovementDirection::BackwardLeft) => p.movement = Some(MovementDirection::Left),
                Some(MovementDirection::BackwardRight) => p.movement = Some(MovementDirection::Right),
                Some(MovementDirection::Backward) => p.movement = None,
                _ => {},
            }
        },
        Keycode::A => {
            match p.movement {
                Some(MovementDirection::ForwardLeft) => p.movement = Some(MovementDirection::Forward),
                Some(MovementDirection::BackwardLeft) => p.movement = Some(MovementDirection::Backward),
                Some(MovementDirection::Left) => p.movement = None,
                _ => {},
            }
        },
        Keycode::D => {
            match p.movement {
                Some(MovementDirection::ForwardRight) => p.movement = Some(MovementDirection::Forward),
                Some(MovementDirection::BackwardRight) => p.movement = Some(MovementDirection::Backward),
                Some(MovementDirection::Right) => p.movement = None,
                _ => {},
            }
        },

        Keycode::LEFT if !p.rotation.is_some() => p.rotation = Some(Rotation::Left),
        Keycode::RIGHT if !p.rotation.is_some() => p.rotation = Some(Rotation::Right),
        _ => {}
    }
}
