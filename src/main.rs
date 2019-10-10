extern crate pixel_engine as engine;
use engine::sdl2::keyboard::Keycode;
use std::f64::consts::PI;
//use std::io::BufRead;
mod maps;
struct Player {
    angle: f64,
    x: f64,
    y: f64,
    fov: f64,
    speed: f64,
    depth: f64,
}

const MMF: i32 = 4; // Minimap factor

impl Player {
    fn new() -> Player {
        Player {
            angle: 0_f64,
            x: 2_f64,
            y: 2_f64,
            fov: 3.14159_f64 / 4.0_f64,
            depth: 16.0_f64,
            speed: 5.0_f64,
        }
    }
}

fn game_logic(game: &mut engine::Engine) -> Result<(), String> {
    // #![allow(unused_assignments)]
    // DECLARE YOUR VARIABLE HERE
    let wall = engine::Sprite::load_from_file(&std::path::Path::new("brick.png"))?;
    let mut player = Player::new();
    let mut map = maps::WorldConstructor::load_file(String::from("maps/dev.map"))?.to_world();
    map.load_all()?;
    let mut current_tile: char = '#';
    // END OF DECLARATION
    'running: loop {
        game.screen.clear(engine::Color::BLACK);
        if !game.new_frame()? {
            break 'running;
        }
        // WRITE YOUR CODE HERE
        if game.is_held(Keycode::Q) {
            // TURN TO THE LEFT
            player.angle -= (player.speed * 0.75_f64) * game.elapsed;
        }
        if game.is_held(Keycode::D) {
            // TURN TO THE RIGHT
            player.angle += (player.speed * 0.75_f64) * game.elapsed;
        }
        if game.is_held(Keycode::Z) {
            // MOVE FORWARD
            player.x += player.angle.sin() * player.speed * game.elapsed;
            player.y += player.angle.cos() * player.speed * game.elapsed;
            if map.get_2d(player.x as i64, player.y as i64) != Some('.') {
                player.x -= player.angle.sin() * player.speed * game.elapsed;
                player.y -= player.angle.cos() * player.speed * game.elapsed;
            }
        }
        if game.is_held(Keycode::S) {
            // MOCE BACKWARD
            player.x -= player.angle.sin() * player.speed * game.elapsed;
            player.y -= player.angle.cos() * player.speed * game.elapsed;
            if map.get_2d(player.x as i64, player.y as i64) != Some('.') {
                player.x += player.angle.sin() * player.speed * game.elapsed;
                player.y += player.angle.cos() * player.speed * game.elapsed;
            }
        }
        if game.is_held(Keycode::A) {
            // MOVE LEFT
            player.x -= player.angle.cos() * player.speed * game.elapsed;
            player.y += player.angle.sin() * player.speed * game.elapsed;
            if map.get_2d(player.x as i64, player.y as i64) != Some('.') {
                player.x += player.angle.cos() * player.speed * game.elapsed;
                player.y -= player.angle.sin() * player.speed * game.elapsed;
            }
        }
        if game.is_held(Keycode::E) {
            // MOVE RIGHT
            player.x += player.angle.cos() * player.speed * game.elapsed;
            player.y -= player.angle.sin() * player.speed * game.elapsed;
            if map.get_2d(player.x as i64, player.y as i64) != Some('.') {
                player.x -= player.angle.cos() * player.speed * game.elapsed;
                player.y += player.angle.sin() * player.speed * game.elapsed;
            }
        }
        if game.is_pressed(Keycode::Escape) {
            break 'running;
        }

        for x in 0..=(game.size.0) {
            let ray_angle = (player.angle - player.fov / 2.0_f64)
                + (x as f64 / game.size.0 as f64) * player.fov;
            let mut wall_distance = 0_f64;
            let stepsize = 0.1_f64;
            let mut hit_wall = false;
            let eye_x = (&ray_angle).sin();
            let eye_y = (&ray_angle).cos();
            let mut sample_x = -0.1_f64;
            while !hit_wall && wall_distance < player.depth {
                wall_distance += stepsize;

                // CORDINATES OF CURRENT TESTED CELL AS i64
                let test_x = (player.x + eye_x * wall_distance).floor() as i64;
                let test_y = (player.y + eye_y * wall_distance).floor() as i64;

                if test_x < 0
                    || test_x >= map.map.w as i64
                    || test_y < 0
                    || test_y >= map.map.h as i64
                {
                    hit_wall = true;
                    wall_distance = player.depth;
                    sample_x = -1.0;
                } else {
                    if map.get_2d(test_x, test_y) != Some('.') {
                        hit_wall = true;
                        current_tile = map.get_2d(test_x, test_y).unwrap();
                        // MIDDLE OF WALL AS f64
                        let mid_x = test_x as f64 + 0.5_f64;
                        let mid_y = test_y as f64 + 0.5_f64;

                        let test_point_x = player.x + eye_x * wall_distance;
                        let test_point_y = player.y + eye_y * wall_distance;

                        let test_angle =
                            (test_point_y as f64 - mid_y).atan2(test_point_x as f64 - mid_x);

                        if test_angle >= -PI * 0.25_f64 && test_angle < PI * 0.25_f64 {
                            sample_x = test_point_y - (test_y as f64);
                        } else if test_angle >= PI * 0.25_f64 && test_angle < PI * 0.75_f64 {
                            sample_x = test_point_x - (test_x as f64);
                        } else if test_angle < -PI * 0.25_f64 && test_angle >= -PI * 0.75_f64 {
                            sample_x = test_point_x - (test_x as f64);
                        } else if test_angle >= PI * 0.75_f64 || test_angle < -PI * 0.75_f64 {
                            sample_x = test_point_y - (test_y as f64);
                        } else {
                            sample_x = -1.0_f64
                        }
                    }
                }
            }
            let ceiling =
                ((game.size.1 as f64 / 2.0) as f64 - game.size.1 as f64 / wall_distance) as i64;
            let floor = (game.size.1 as i64 - ceiling) as i64;

            for y in 0..=((game).size.1) {
                if y as i64 <= ceiling {
                    // CEILING
                    game.screen
                        .draw(x as i32, y as i32, engine::Color::BLACK)
                        .expect("Error while drawing to screen");
                } else if y as i64 > ceiling && y as i64 <= floor {
                    // WALL
                    let sample_y =
                        ((y as f64) - (ceiling as f64)) / ((floor as f64) - (ceiling as f64));
                    let color = if let Some(tile) = map.tiles.get(&current_tile)
                    //== Some(tile)
                    {
                        tile.sprite.as_ref().unwrap().get_sample(sample_x, sample_y)
                    } else {
                        engine::Color::GREEN
                    };
                    game.screen
                        .draw(x as i32, y as i32, color)
                        .expect("Error while drawing to screen");
                /*match wall.get_sample(sample_x, sample_y) {
                    engine::Color::WHITE => {
                        println!("WHITE PIXEL DRAWN TO SCREEN!");
                    }
                    _ => {}
                };*/
                } else {
                    // FLOOR
                    game.screen
                        .draw(x as i32, y as i32, engine::Color::DARK_GREEN)
                        .expect("Error while drawing to screen");
                }
            }
        }
        for ny in 0..=(map.map.h - 1) {
            for nx in 0..=(map.map.w - 1) {
                match map.get_2d(nx as i64, ny as i64) {
                    Some('.') => {
                        game.screen.fill_rect(
                            (nx as i32 * MMF) + MMF,
                            (ny as i32 * MMF) + MMF,
                            MMF,
                            MMF,
                            engine::Color::BLACK,
                        )?;
                    }
                    _ => {
                        game.screen.fill_rect(
                            (nx as i32 * MMF) + MMF,
                            (ny as i32 * MMF) + MMF,
                            MMF,
                            MMF,
                            engine::Color::RED,
                        )?;
                    }
                }
                game.screen.fill_rect(
                    (player.x as i32 * MMF) + MMF,
                    (player.y as i32 * MMF) + MMF,
                    MMF,
                    MMF,
                    engine::Color::GREEN,
                )?;
            }
        }
        // END OF CODE
        game.screen.update();
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let fac = 4;
    let mut game: engine::Engine<'static> =
        engine::Engine::new("Pixel FPS", (120 * fac, 60 * fac, 10 / fac), &game_logic)
            .map_err(|err| err.to_string())?;
    game.start()?;
    Ok(())
}
