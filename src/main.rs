//extern crate pixel_engine as engine;
extern crate rayon;
pub mod engine;
use engine::sdl2::keyboard::Keycode;
// use rayon::prelude::*;
use std::io::BufRead;
struct Player {
    angle: f64,
    x: f64,
    y: f64,
    fov: f64,
    speed: f64,
    depth: f64,
}

unsafe impl Send for engine::Engine<'_> {}
unsafe impl Sync for engine::Engine<'_> {}

const MMF: i32 = 1; // Minimap factor

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

struct Map {
    map: String,
    w: i64,
    h: i64,
}

impl Map {
    /*fn new() -> Map {
        let mut map = String::from("");
        map.push_str("################");
        map.push_str("#..............#");
        map.push_str("#..............#");
        map.push_str("#..............#");
        map.push_str("#..............#");
        map.push_str("#........#.....#");
        map.push_str("#........#.....#");
        map.push_str("#........#.....#");
        map.push_str("#....#####.....#");
        map.push_str("#..............#");
        map.push_str("#..............#");
        map.push_str("#..............#");
        map.push_str("#........#######");
        map.push_str("#........#.....#");
        map.push_str("#..............#");
        map.push_str("################");
        Map { map, w: 16, h: 16 }
    }
    fn get(&self, index: i64) -> Option<char> {
        self.map.chars().nth(index as usize)
    }*/
    fn get_2d(&self, x: i64, y: i64) -> Option<char> {
        self.map.chars().nth((y * self.w + x) as usize)
    }
    fn load_from_file(file_path: &std::path::Path) -> Result<Map, String> {
        let mut map = Map {
            w: 0,
            h: 0,
            map: String::from(""),
        };
        let file = std::fs::File::open(file_path).map_err(|err| err.to_string())?;
        let reader = std::io::BufReader::new(file);
        for (index, line) in reader.lines().enumerate() {
            let line = line.map_err(|err| err.to_string())?; // Ignore errors.
            map.h = index as i64 + 1;
            if index == 0 {
                map.w = line.len() as i64;
            }
            if map.w != line.len() as i64 {
                return Err(String::from("map not with same width!"));
            }
            map.map.push_str(&line)
        }
        Ok(map)
    }
}

fn game_logic(game: &mut engine::Engine) -> Result<(), String> {
    #![allow(unused_assignments)]
    // DECLARE YOUR VARIABLE HERE
    let mut player = Player::new();
    let map = Map::load_from_file(&std::path::Path::new("maps/dev.map"))?;

    // RAYCAST VARS

    let mut vx = 0_f64;
    let mut vy = 0_f64;
    let mut d = 0_f64;
    let mut dot = 0_f64;
    let mut b = 0_f64;
    // END OF RAYCAST VARS

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
            if map.get_2d(player.x as i64, player.y as i64) == Some('#') {
                player.x -= player.angle.sin() * player.speed * game.elapsed;
                player.y -= player.angle.cos() * player.speed * game.elapsed;
            }
        }
        if game.is_held(Keycode::S) {
            // MOCE BACKWARD
            player.x -= player.angle.sin() * player.speed * game.elapsed;
            player.y -= player.angle.cos() * player.speed * game.elapsed;
            if map.get_2d(player.x as i64, player.y as i64) == Some('#') {
                player.x += player.angle.sin() * player.speed * game.elapsed;
                player.y += player.angle.cos() * player.speed * game.elapsed;
            }
        }
        if game.is_pressed(Keycode::Escape) {
            break 'running;
        }

        for x in 0..=(game.size.0) {
            rayon::scope(|s| {
                s.spawn(|_| {
                    //ray_angle =
                    //    (player.angle - player.fov / 2.0_f64) + ((x / game.size.0) as f64) * player.fov;
                    let ray_angle = (player.angle - player.fov / 2.0_f64)
                        + (x as f64 / game.size.0 as f64) * player.fov;
                    let mut wall_distance = 0_f64;
                    let stepsize = 0.1_f64;
                    let mut hit_wall = false;
                    let mut boundry = false;
                    let eye_x = ray_angle.sin();
                    let eye_y = ray_angle.cos();
                    let mut boundry_vec: Vec<(f64, f64)> = Vec::new();
                    boundry_vec.clear();
                    while !hit_wall && wall_distance < player.depth {
                        wall_distance += stepsize;
                        let test_x = (player.x + eye_x * wall_distance).floor() as i64;
                        let test_y = (player.y + eye_y * wall_distance).floor() as i64;

                        if test_x < 0 || test_x >= map.w || test_y < 0 || test_y >= map.h {
                            hit_wall = true;
                            wall_distance = player.depth;
                        } else {
                            if map.get_2d(test_x, test_y) == Some('#') {
                                hit_wall = true;
                                boundry_vec.clear();
                                // /*
                                for tx in 0..2 {
                                    for ty in 0..2 {
                                        vy = (test_y as f64) + ty as f64 - player.y;
                                        vx = (test_x as f64) + tx as f64 - player.x;
                                        d = (vx * vx + vy * vy).sqrt();
                                        dot = (eye_x * vx / d) + (eye_y * vy / d);
                                        boundry_vec.push((d, dot));
                                    }
                                }
                                boundry_vec.sort_by(|left: &(f64, f64), right: &(f64, f64)| {
                                    left.0.partial_cmp(&right.0).unwrap()
                                });

                                let bound = 0.01_f64;
                                if boundry_vec[0].1.acos() < bound {
                                    //println!("{:?}", boundry_vec[0].1.acos());
                                    boundry = true;
                                }
                                if boundry_vec[1].1.acos() < bound {
                                    //println!("{:?}", boundry_vec[1].1.acos());
                                    boundry = true;
                                }
                                if boundry_vec[2].1.acos() < bound {
                                    //println!("{:?}", boundry_vec[2].1.acos());
                                    boundry = true;
                                }
                                // */
                            }
                        }
                    }
                    let ceiling = ((game.size.1 as f64 / 2.0) as f64
                        - game.size.1 as f64 / wall_distance)
                        as i64;
                    let floor = (game.size.1 as i64 - ceiling) as i64;

                    let mut col = engine::Color::BLACK;
                    if wall_distance <= player.depth / 4.0_f64 {
                        col = engine::Color::WHITE;
                    } else if wall_distance < player.depth / 3.0_f64 {
                        //println!("grey");
                        col = engine::Color::GREY;
                    } else if wall_distance < player.depth / 2.0_f64 {
                        //println!("dark");
                        col = engine::Color::DARK_GREY;
                    } else if wall_distance < player.depth {
                        //println!("darker");
                        col = engine::Color::VERY_DARK_GREY;
                    } else {
                        //println!("black");
                        col = engine::Color::BLACK;
                    }

                    if boundry {
                        //println!("boundry");
                        if wall_distance <= player.depth / 4.0_f64 {
                            col = engine::Color::GREY;
                        } else if wall_distance < player.depth / 3.0_f64 {
                            col = engine::Color::DARK_GREY;
                        } else if wall_distance < player.depth / 2.0_f64 {
                            col = engine::Color::VERY_DARK_GREY;
                        } else if wall_distance < player.depth {
                            col = engine::Color::new(32, 32, 32);
                        } else {
                            col = engine::Color::BLACK;
                        }
                    }

                    for y in 0..=((game).size.1) {
                        if y as i64 <= ceiling {
                            // CEILING
                            game.screen
                                .draw(x as i32, y as i32, engine::Color::BLACK)
                                .expect("Error while drawing to screen");
                        } else if y as i64 > ceiling && y as i64 <= floor {
                            // WALL
                            game.screen
                                .draw(x as i32, y as i32, col)
                                .expect("Error while drawing to screen");
                        } else {
                            // FLOOR
                            b = 1.0_f64
                                - ((y as f64) - (&game).size.1 as f64 / 2.0_f64)
                                    / (((&game).size.1 as f64 / 2.0_f64) as f64);

                            if b < 0.25 {
                                col = engine::Color::BLUE;
                            } else if b < 0.5 {
                                col = engine::Color::DARK_BLUE;
                            } else if b < 0.75 {
                                col = engine::Color::VERY_DARK_BLUE;
                            } else if b < 0.9 {
                                col = engine::Color::new(0, 0, 32);
                            } else {
                                col = engine::Color::BLACK;
                            }

                            //col = engine::Color::new(0, 0, (255_f64 - (255_f64 * b)) as u8);
                            game.screen
                                .draw(x as i32, y as i32, col)
                                .expect("Error while drawing to screen");
                        }
                    }
                });
            });
            for ny in 0..=(map.h - 1) {
                for nx in 0..=(map.w - 1) {
                    //println!("{:?}, {:?} : {:?}", nx, ny, map.get(ny * map.w + nx));
                    match map.get_2d(nx, ny) {
                        Some('#') => {
                            game.screen.fill_rect(
                                (nx as i32 * MMF) + MMF,
                                (ny as i32 * MMF) + MMF,
                                MMF,
                                MMF,
                                engine::Color::RED,
                            )?;
                        }
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
                                engine::Color::MAGENTA,
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
        }
        // END OF CODE
        game.screen.update();
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let mut game: engine::Engine<'static> =
        engine::Engine::new("Pixel FPS", (60, 60, 10), &game_logic)
            .map_err(|err| err.to_string())?;
    game.start()?;
    Ok(())
}
