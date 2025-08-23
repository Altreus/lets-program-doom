extern crate sdl2;

use std::f64::consts::TAU;
use sdl2::rect::Point;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 120;
const PIXEL_SCALE: u32 = 4;
const FRAME_RATE: u32 = 20;
const MS_PER_FRAME: u32 = 1000 / FRAME_RATE;
const FIELD_OF_VIEW : i32 = 90;

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

#[derive(Debug)]
enum Rotation {
    Left, Right
}

#[derive(Default, Debug)]
struct Point3 {
    pub x: i32,
    pub y: i32,
    pub z: i32
}

impl Point3 {
    fn new(x: i32, y: i32, z: i32) -> Point3 {
        Point3{ x,y,z }
    }
}

#[derive(Default, Debug)]
struct FPoint3 {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl FPoint3 {
    fn new(x: f64, y: f64, z: f64) -> FPoint3 {
        FPoint3{ x,y,z }
    }
}

#[derive(Default)]
struct PlayerMovement {
    pub movement: Option<MovementDirection>,
    pub rotation: Option<Rotation>
}

#[derive(Default)]
struct Player {
    pub location: Point3,
    pub angle: f64,
    pub movement: PlayerMovement,
}

// In Doom, all points in the map are integers, so we don't need to use
// FPoint3 to define a wall; only to avoid losing precision while rendering it
struct Wall {
    pub line: [Point3;2],
    pub height: i32
}

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    let window = video_subsystem
        .window("Game",
            SCREEN_WIDTH * PIXEL_SCALE,
            SCREEN_HEIGHT * PIXEL_SCALE)
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
    let mut player : Player = Default::default();
    player.location.y = 20;

    let mut event_pump = sdl.event_pump().unwrap();
    let mut lastframe = Instant::now();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'main,
                Event::KeyDown { keycode: Some(k), .. } => on_key_down(&mut player.movement, k),
                Event::KeyUp   { keycode: Some(k), .. } => on_key_up  (&mut player.movement, k),
                _ => {}
            }
        }

        // Draw frame_rate times per second
        if lastframe.elapsed().as_millis() >= MS_PER_FRAME as u128 {
            move_player(&mut player);
            draw_stuff(&mut canvas, &player);

            canvas.present();
            lastframe = Instant::now();
        }
    }
}

fn move_player(p: &mut Player) {
    // Let's Program Doom used 4 degrees, which is 1/90 of a turn
    match &p.movement.rotation {
        None => {},
        Some(Rotation::Left) => { p.angle -= TAU/90.0; if p.angle < 0.0 { p.angle += TAU } }
        Some(Rotation::Right) => { p.angle += TAU/90.0; if p.angle > TAU { p.angle -= TAU } }
    }

    let dx = (p.angle.sin() * 10.0) as i32;
    let dz = (p.angle.cos() * 10.0) as i32;
    println!("Player delta xz: {:?} {:?}", dx, dz);

    match &p.movement.movement {
        None => {}
        Some(x) => { match x {
            // Cargo-culted from the video, in which he seems very confident it
            // works like this
            MovementDirection::Forward  => { p.location.x += dx; p.location.z += dz },
            MovementDirection::Backward => { p.location.x -= dx; p.location.z -= dz },
            MovementDirection::Right    => { p.location.x += dz; p.location.z += dx },
            MovementDirection::Left     => { p.location.x -= dz; p.location.z -= dx },
            _ => {},
            // Can't be bothered figuring this out yet. It's going to involve
            // a point on a circle between the two directions and therefore
            // more sines and cosines
            //MovementDirection::ForwardRight,
            //MovementDirection::BackwardRight,
            //MovementDirection::BackwardLeft,
            //MovementDirection::ForwardLeft
        }}
    }
}

fn draw_stuff(canvas: &mut sdl2::render::WindowCanvas, player: &Player) {
    canvas.set_draw_color(Color::BLUE);
    canvas.clear();

    // In the video he defines a wall as 4 points in 3D, and then constructs
    // them programmatically from something or other. Let's not do that,
    // because he then later just draws one point, and then two, and then a
    // line in between them.
    // Walls in Doom are coded as a line with height, so let's start with a
    // line and later add the height.

    let wall = Wall {
        line: [ Point3::new(0, 0, 50), Point3::new(50, 0, 50) ],
        height: 40
    };

    for point in wall.line {
        println!("Point is {:?}", point);
        let mut point_to_draw = FPoint3::new(
            (point.x - player.location.x) as _,
            (point.y - player.location.y) as _,
            (point.z - player.location.z) as _
        );
        println!("After translation {:?}", point_to_draw);

        let newx = point_to_draw.x * player.angle.cos() - point_to_draw.z * player.angle.sin();
        let newz = point_to_draw.z * player.angle.cos() + point_to_draw.x * player.angle.sin();
        point_to_draw.x = newx; point_to_draw.z = newz;
        println!("After rotation {:?}", point_to_draw);

        if point_to_draw.z <= 0.0 { return }

        // Invert the Y coordinate. World coordinates have +Y is up, but
        // screen coordinates have +Y is down. That'll flip the world, so
        // flip it back
        let pixel = Point::new(
            ((point_to_draw.x / point_to_draw.z * FIELD_OF_VIEW as f64) + (SCREEN_WIDTH as f64 / 2.0)) as i32,
            ((-point_to_draw.y / point_to_draw.z * FIELD_OF_VIEW as f64) + (SCREEN_HEIGHT as f64 / 2.0)) as i32,
        );
        println!("After projection {:?}", pixel);
        draw_pixel( canvas, pixel, Color::YELLOW );
    }
    // for x in 0 .. (SCREEN_WIDTH / 2) - 1 {
    //     for y in 0 .. (SCREEN_HEIGHT / 2) - 1 {
    //         let x_pc : f32 = x as f32 / (SCREEN_WIDTH as f32 / 2.0);
    //         let y_pc : f32 = y as f32 / (SCREEN_HEIGHT as f32 / 2.0);
    //
    //         draw_pixel(canvas, Point::new(x as _,y as _), Color::RGB(
    //             (255.0 * x_pc) as _, (255.0 * y_pc) as _, (255.0 * x_pc * y_pc ) as _ )
    //         );
    //     }
    // }
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

        Keycode::LEFT => match p.rotation {
            Some(Rotation::Left) => p.rotation = None,
            _ => {}
        },
        Keycode::RIGHT => match p.rotation {
            Some(Rotation::Right) => p.rotation = None,
            _ => {}
        },
        _ => {}
    }
}
