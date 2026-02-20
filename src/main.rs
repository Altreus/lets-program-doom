extern crate sdl2;

use std::f64::consts::TAU;
use sdl2::rect::{Point, FPoint};
use sdl2::render::WindowCanvas;
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

    fn clone(p3: &Point3) -> Point3 {
        Point3{ x: p3.x, y: p3.y, z: p3.z }
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
    // The XY of the point is actually the XZ of the world. The Y value of
    // the world is simply the floor value, and the upper line is defined by
    // the height.
    pub line: [Point;2],
    // floor is an absolute unit
    pub floor: i32,
    // height is relative to floor
    pub height: i32
}

impl Wall {
    fn transformed(&self, player: &Player ) -> Wall {
        return Wall {
            line: [
                point2_transformed( &self.line[0], player),
                point2_transformed( &self.line[1], player),
            ],
            floor: self.floor - player.location.y,
            height: self.height,
        };
    }
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
    if p.movement.movement.is_none() && p.movement.rotation.is_none() { return }

    // Let's Program Doom used 4 degrees, which is 1/90 of a turn
    match &p.movement.rotation {
        None => {},
        Some(Rotation::Left) => { p.angle -= TAU/90.0; if p.angle < 0.0 { p.angle += TAU } }
        Some(Rotation::Right) => { p.angle += TAU/90.0; if p.angle > TAU { p.angle -= TAU } }
    }

    // We're trying to move by 10 units in the direction of the player
    // (or perpendicular to it) but something's wrong
    let dx = (p.angle.sin() * 10.0) as i32;
    let dz = (p.angle.cos() * 10.0) as i32;

    match &p.movement.movement {
        None => {}
        Some(x) => { match x {
            // Cargo-culted from the video, in which he seems very confident it
            // works like this
            MovementDirection::Forward  => { p.location.x += dx; p.location.z += dz },
            MovementDirection::Backward => { p.location.x -= dx; p.location.z -= dz },
            MovementDirection::Right    => { p.location.x += dz; p.location.z -= dx },
            MovementDirection::Left     => { p.location.x -= dz; p.location.z += dx },
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

fn draw_stuff(canvas: &mut WindowCanvas, player: &Player) {
    canvas.set_draw_color(Color::BLUE);
    canvas.clear();

    // In the video he defines a wall as 4 points in 3D, and then constructs
    // them programmatically from something or other. Let's not do that,
    // because he then later just draws one point, and then two, and then a
    // line in between them.
    // Walls in Doom are coded as a line with height, so let's start with a
    // line and later add the height.

    let wall = Wall {
        line: [ Point::new(-40, 50), Point::new(40, 50) ],
        floor: 0,
        height: 40
    };

    draw_wall( canvas, wall.transformed(player), Color::YELLOW );

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

// Eventually the wall will know its colour, which will of course be a texture
// The function takes a transformed wall, so the player is at 0,0,0.
fn draw_wall(canvas: &mut WindowCanvas, wall: Wall, c: Color) {
    // In the video the drawWall function iterates over the SCREEN values of
    // the wall that he computes in the draw function. Maybe he tidies this up
    // later but why would you write code you need to tidy up? The function is
    // called drawWall so it should accept wall coordinates, which is what this
    // function does.
    //
    // We convert the floor line (which is what's in the struct) into 2 screen
    // coordinates, and a second virtual line adjusted in the Y by the wall's
    // height into 2 more screen coordinates. Then we iterate over the X values
    // between the floor points and draw up to the corresponding top point.

    let line_in_view = match truncate_z( &wall.line ) {
        Some(x) => x,
        None    => return
    };

    let bot_point_1 = Point3::new( line_in_view[0].x, wall.floor, line_in_view[0].y);
    let bot_point_2 = Point3::new( line_in_view[1].x, wall.floor, line_in_view[1].y);
    let screen_bot_1 = point3_to_point2( &bot_point_1 );
    let screen_bot_2 = point3_to_point2( &bot_point_2 );

    // Another way to do this would be to compute the Y offset of the wall
    // height inside the loop at each X value, since we don't ever actually
    // use the X values of these two points.
    let top_point_1 = Point3::new( wall.line[0].x, wall.floor + wall.height, wall.line[0].y);
    let top_point_2 = Point3::new( wall.line[1].x, wall.floor + wall.height, wall.line[1].y);
    let screen_top_1 = point3_to_point2( &top_point_1 );
    let screen_top_2 = point3_to_point2( &top_point_2 );


    // This is where we use the X values of the bottom points but not the
    // top points, because when we're drawing the wall we just draw vertical
    // lines from each X point
    let dx = screen_bot_1.x - screen_bot_2.x;
    let dy_bot = screen_bot_1.y - screen_bot_2.y;
    let dy_top = screen_top_1.y - screen_top_2.y; // top dx is the same

    // Actually we only use screen_top_2 once as well, to get a delta Y, so
    // maybe we can get away with just creating two Y values

    for xval in screen_bot_1.x .. screen_bot_2.x {
        // xval - wall.line[0].x / dx normalises xval so needs to be a float
        // multiply that by dy and we get a normalised y value proportional to x
        // make that an int and add the actual y value of the first point
        let yval_bot = ((dy_bot * ( xval - screen_bot_1.x ) ) as f64 / dx as f64 ) as i32 + screen_bot_1.y;
        let yval_top = ((dy_top * ( xval - screen_bot_1.x ) ) as f64 / dx as f64 ) as i32 + screen_top_1.y;

        // Remember +Y is down so top < bot
        for yval in yval_top .. yval_bot {
            draw_pixel( canvas, Point::new(xval, yval), c );
        }
    }
}

fn point2_transformed(p2: &Point, player: &Player) -> Point {
    // We translate the point by using its XY coordinates as being the ground
    // plane of the world, and thus the Y point of the coordinate is the Z
    // axis in the world. So we use the player's Z to transform the Y
    let mut translated_point = FPoint::new(
        (p2.x - player.location.x) as _,
        (p2.y - player.location.z) as _
    );

    let newx = translated_point.x as f32 * player.angle.cos() as f32
             - translated_point.y as f32 * player.angle.sin() as f32;
    let newy = translated_point.y as f32 * player.angle.cos() as f32
             + translated_point.x as f32 * player.angle.sin() as f32;
    translated_point.x = newx; translated_point.y = newy;

    // FIXME: We can't just return None here and make this an Option function
    // because we still need to interpolate between two points. For now we
    // fudge the Y value so there's no DIV0 error and come back to clipping
    // later
    if translated_point.y == 0.0 { translated_point.y = 0.1 }

    return Point::new( translated_point.x as _, translated_point.y as _ );
}

fn point3_to_point2(p3: &Point3) -> Point {
    let pixel = Point::new(
        ((p3.x as f64 / p3.z as f64 * FIELD_OF_VIEW as f64) + (SCREEN_WIDTH as f64 / 2.0)) as i32,
        ((-p3.y as f64 / p3.z as f64 * FIELD_OF_VIEW as f64) + (SCREEN_HEIGHT as f64 / 2.0)) as i32,
    );

    return pixel;
}

fn truncate_z(line: &[Point;2]) -> Option<[Point;2]> {
    let mut new_line = [line[0].clone(), line[1].clone()];
    // Remember that the Y coordinate of the line is the Z coordinate of the
    // world; and this line comes to us pre-translated, so +Z in the world is
    // +Y in the line

    // Completely behind us
    if line[0].y < 1 && line[1].y < 1 { return None }

    // The proportion of the line that's behind us based on the Y coordinate
    // can be directly applied to the X coordinate range to chop that bit off.
    // We'll use 1 instead of 0 to avoid div0.
    // If the line is entirely in front of us, just use it.
    if line[0].y > 1 && line[1].y > 1 { return Some(new_line) }

    // If line[1] is actually behind us (because we turned 180°) then swap
    // new_line around, work on it, then swap it back.
    if line[1].y < 1 {
        new_line = [new_line[1], new_line[0]];
    }
    let line_length = new_line[1].y - new_line[0].y;
    let player_proportion = line_length as f32 / (1 - new_line[0].y) as f32;

    new_line[0].y = 1;
    new_line[0].x = (new_line[0].x as f32 * player_proportion) as _;

    if line[1].y < 1 {
        new_line = [new_line[1], new_line[0]];
    }

    return Some(new_line);
}

fn draw_pixel(canvas: &mut WindowCanvas, p: Point, c: Color) {
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
