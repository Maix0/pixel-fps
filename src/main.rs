extern crate pixel_engine as engine;
extern crate ron;
extern crate serde;
use engine::Keycode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
struct Sprite(engine::Sprite);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct World {
    map: Map,
    objs: Vec<Objects>,
    tiles: std::collections::HashMap<char, Tile>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Map {
    map: String,
    w: u64,
    h: u64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Objects {
    #[serde(skip)]
    sprite: Option<Sprite>,
    sprite_path: String,
    x: f64,
    y: f64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Tile {
    #[serde(skip)]
    sprite: Option<Sprite>,
    sprite_path: String,
    chr: char,
}
impl Tile {
    fn load(&mut self) -> Result<(), String> {
        self.sprite = Some(Sprite {
            0: engine::Sprite::load_from_file(std::path::Path::new(&self.sprite_path))?,
        });
        Ok(())
    }
}

struct WorldConstructor {
    map: Vec<String>,
    objects: Vec<Objects>,
    tiles: std::collections::HashMap<char, Tile>,
}
impl WorldConstructor {
    fn new() -> WorldConstructor {
        WorldConstructor {
            map: Vec::new(),
            objects: Vec::new(),
            tiles: std::collections::HashMap::new(),
        }
    }
    fn load_file(path: String) -> Result<WorldConstructor, String> {
        use std::io::prelude::*;
        if std::path::Path::new(&path).exists() {
            let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
            let mut data = String::new();
            file.read_to_string(&mut data).map_err(|e| e.to_string())?;
            let world = ron::de::from_str::<'_, World>(&data).map_err(|e| e.to_string())?;
            Ok(WorldConstructor::from_world(world))
        } else {
            Ok(WorldConstructor::new())
        }
    }
    fn from_world(world: World) -> WorldConstructor {
        //let mut temp_map = world.map.split();
        let mut cmap: Vec<String> = Vec::new();
        for chr in world.map.map.split("") {
            if cmap.len() > 0 {
                let last_index = cmap.len() - 1;
                if cmap[last_index].len() < world.map.w as usize {
                    cmap[last_index].push_str(chr)
                } else {
                    cmap.push(chr.to_owned())
                }
            } else {
                cmap.push(chr.to_owned());
            }
        }
        cmap.pop();
        WorldConstructor {
            map: cmap,
            tiles: world.tiles,
            objects: world.objs,
        }
    }
    fn to_world(&mut self) -> World {
        let mut w = 0;
        let h = self.map.len();
        let mut map: Vec<String> = self.map.clone();
        for r in &map {
            if r.len() > w {
                w = r.len();
            }
        }
        let mut index = 0;
        for row in &self.map {
            if index > h {
                break;
            }
            let mut r = row.clone();
            if r.len() < w {
                while r.len() < w {
                    r.push('.');
                }
            }
            map[index] = r.to_owned();
            index += 1;
        }
        World {
            map: Map {
                map: map.join(""), /* STRING */
                w: w as u64,       /* u64 */
                h: h as u64,       /* u64 */
            },
            tiles: self.tiles.clone(),
            objs: self.objects.clone(),
        }
    }
    fn map_set_y(&mut self, len: usize) {
        if len > self.map.len() {
            while len > self.map.len() {
                self.map.push(String::new());
            }
        }
    }
    fn map_set_x(&mut self, len: usize) {
        for row in &mut self.map {
            if len > row.len() {
                while len >= row.len() {
                    row.push_str(".");
                }
            }
        }
    }
    fn map_set(&mut self, x: usize, y: usize, chr: char) {
        self.map[y] = change_char(self.map[y].clone(), chr, x);
    }
}

fn sprite_frame(game: &mut engine::Engine, spr: &Option<Sprite>) -> Result<(), String> {
    //return Ok(());

    if let Some(spr) = spr {
        for x in 0..257 {
            for y in 0..257 {
                game.screen.draw(
                    x + 5,
                    y + 305,
                    spr.0.get_sample(x as f64 / 256_f64, y as f64 / 256_f64),
                )?
            }
        }
    } else {
        game.screen
            .draw_line(6, 306, 6 + 254, 306 + 254, engine::Color::WHITE)?;
        game.screen
            .draw_line(6, 306 + 254, 6 + 254, 306, engine::Color::WHITE)?;
    }
    Ok(())
}

fn game_logic(game: &mut engine::Engine) -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    //let mut c_world = WorldConstructor::new();
    let mut c_world: WorldConstructor;
    if args.len() > 1 {
        c_world = WorldConstructor::load_file(args[1].clone()).map_err(|e| e.to_string())?;
    } else {
        panic!("Filename required");
    }
    let mut typing = false;
    let mut typed_string = String::new();
    let mut finished_string = String::new();
    std::fs::write(
        &args[1],
        ron::ser::to_string(&c_world.to_world()).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    game.screen.clear(engine::Color::BLACK);
    game.screen.update();
    for tile in &mut c_world.tiles {
        tile.1.load()?;
    }
    let mut selected_tile: Option<&Tile> = None;
    let mut selected_tile_index = 0;

    let mut add_tile_t: Tile = Tile {
        sprite: None,
        sprite_path: String::new(),
        chr: '\u{0000}',
    };
    let mut add_tile = false;
    let mut add_tile_chr_buf: String = String::new();
    let mut add_tile_field = 0;

    // MAP EDIT VARS
    let mut c_tile_x: f64 = 0.0;
    let mut c_tile_y: f64 = 0.0;
    // END

    //println!("{:?}", c_world.map);
    'running: loop {
        if !game.new_frame()? {
            break 'running;
        }
        if game.is_pressed(Keycode::Escape) || game.is_held(Keycode::Escape) {
            break 'running;
        }
        if typing {
            let keys = game.get_pressed();
            for key in &keys {
                if *key == engine::Keycode::Return || *key == engine::Keycode::KpEnter {
                    println!("end of typing !");
                    typing = false;
                    finished_string = typed_string.clone();
                    typed_string = String::new();
                } else if *key == engine::Keycode::Backspace {
                    typed_string.pop();
                } else {
                    typed_string += &normalize(*key);
                }
            }
        }
        if !typing && !add_tile && game.is_pressed(Keycode::P) {
            selected_tile_index += 1;
            if selected_tile_index > c_world.tiles.len() {
                selected_tile_index = 0;
            }
        }
        if !typing && !add_tile && game.is_pressed(Keycode::M) {
            selected_tile_index -= 1;
            if selected_tile_index > c_world.tiles.len() {
                selected_tile_index = 0;
            }
        }

        if !typing && finished_string.len() > 0 {
            println!("finished === {}", finished_string);
            finished_string = String::new();
        }

        game.screen.clear(engine::Color::BLACK);
        game.screen.draw_rect(5, 5, 590, 295, engine::Color::RED)?;
        game.screen
            .draw_rect(5, 305, 257, 256, engine::Color::BLUE)?;
        game.screen
            .draw_rect(5, 566, 257, 29, engine::Color::WHITE)?;
        game.screen
            .draw_rect(266, 305, 329, 290, engine::Color::GREEN)?;
        //game.screen
        //    .draw_string(0, 0, typed_string.clone(), engine::Color::WHITE, 1)?;

        // HANDLE MAP VIEW + EDIT

        let offset_x = 8;
        let offset_y = 8;
        let mut index_x;
        let mut index_y = 0;
        if !typing {
            if game.is_pressed(Keycode::Left) || game.is_held(Keycode::Left) {
                if c_tile_x != 0.0 {
                    c_tile_x -= 20.0 * game.elapsed;
                }
                if c_tile_x < 0.0 {
                    c_tile_x = 0.0;
                }
            }
            if game.is_pressed(Keycode::Right) || game.is_held(Keycode::Right) {
                c_tile_x += 20.0 * game.elapsed;
                if c_tile_x > 72.0 {
                    c_tile_x = 72.0;
                }
            }
            if game.is_pressed(Keycode::Up) || game.is_held(Keycode::Up) {
                if c_tile_y != 0.0 {
                    c_tile_y -= 20.0 * game.elapsed;
                }
                if c_tile_y < 0.0 {
                    c_tile_y = 0.0;
                }
            }
            if game.is_pressed(Keycode::Down) || game.is_held(Keycode::Down) {
                c_tile_y += 20.0 * game.elapsed;
                if c_tile_y > 35.0 {
                    c_tile_y = 35.0;
                }
            }
            if game.is_pressed(Keycode::Return) || game.is_held(Keycode::Return) {
                let selected_char = match &selected_tile {
                    Some(t) => t.chr,
                    None => '.',
                };
                c_world.map_set_y(c_tile_y as usize + 1);
                c_world.map_set_x(c_tile_x as usize);
                c_world.map_set(c_tile_x as usize, c_tile_y as usize, selected_char);
            }
        }
        game.screen.fill_rect(
            (offset_x + 8 * c_tile_x as usize) as i32,
            (offset_y + 8 * c_tile_y as usize) as i32,
            8,
            8,
            engine::Color::GREY,
        )?;
        for row in &mut c_world.map {
            index_x = 0;
            for chr in row.chars() {
                if index_y == c_tile_y as usize && index_x == c_tile_x as usize {
                    game.screen.draw_string(
                        (offset_x + 8 * index_x) as u32,
                        (offset_y + 8 * index_y) as u32,
                        format!("{}", chr),
                        engine::Color::BLACK,
                        1,
                    )?;
                } else {
                    game.screen.draw_string(
                        (offset_x + 8 * index_x) as u32,
                        (offset_y + 8 * index_y) as u32,
                        format!("{}", chr),
                        engine::Color::WHITE,
                        1,
                    )?;
                }
                index_x += 1;
            }
            index_y += 1;
        }

        // END
        if game.is_pressed(Keycode::S) {
            std::fs::write(
                &args[1],
                ron::ser::to_string(&c_world.to_world()).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }

        // HANDLE SPRITE LIST
        /* 267,306,328,289 */
        selected_tile = c_world.tiles.values().nth(selected_tile_index as usize);
        let mut d_offset = 0;
        for (chr, spr) in &c_world.tiles {
            match selected_tile.clone() {
                Some(_) => {
                    if *chr == selected_tile.clone().unwrap().chr {
                        game.screen.fill_rect(
                            267,
                            306 + d_offset,
                            596 - 267 - 2,
                            8,
                            engine::Color::VERY_DARK_GREY,
                        )?;
                        game.screen.draw_string(
                            267,
                            306 + d_offset as u32,
                            format!("'{}': {}", chr, str_normalize(spr.sprite_path.clone())),
                            engine::Color::WHITE,
                            1,
                        )?;
                    } else {
                        game.screen.draw_string(
                            267,
                            306 + d_offset as u32,
                            format!("'{}': {}", chr, str_normalize(spr.sprite_path.clone())),
                            engine::Color::WHITE,
                            1,
                        )?;
                    }
                }
                _ => {
                    game.screen.draw_string(
                        267,
                        306 + d_offset as u32,
                        format!("'{}': {}", chr, str_normalize(spr.sprite_path.clone())),
                        engine::Color::WHITE,
                        1,
                    )?;
                }
            }
            d_offset += 8;
        }

        // END OF SPRITE LIST

        // HANDLE SPRITE DATA
        match &mut selected_tile {
            Some(tile) => {
                sprite_frame(game, &tile.sprite)?; // Handle Sprite Preview

                game.screen.draw_string(
                    6,
                    567,
                    format!(
                        "Current Tile: {} \nSpr Path: {}",
                        tile.chr, tile.sprite_path
                    ),
                    engine::Color::GREY,
                    1,
                )?;
            }
            None => {
                sprite_frame(game, &None)?;
                game.screen.draw_string(
                    6,
                    566 + 8,
                    format!("No Tile Selected"),
                    engine::Color::GREY,
                    2,
                )?;
            }
        };
        /*if let Some(tile) = selected_tile {
        } else {
        }*/
        // END OF SPRITE DATA
        // START OF ADD TILE

        // FIXME: THIS WHOLE BLOCK !
        if false && !add_tile && game.is_pressed(Keycode::A) {
            //#[allow(unused_assignments)]
            //add_tile = true;
            typing = true;
            add_tile_field = 0;
            add_tile_t = Tile {
                sprite: None,
                sprite_path: String::new(),
                chr: '\u{0000}',
            }
        }
        add_tile = false; // FORCE TO NOT GO TO THE ADD TILE MENU BC IT DON'T WORK I DON'T KNOW WHY !!!
        if add_tile {
            if add_tile_field == 0 {
                /*
                    current_string = &mut add_tile_path;
                    *current_string = format!("{}", typed_string.clone());
                */
                add_tile_t.sprite_path = copy_string(format!("{}", typed_string));
            } else if add_tile_field == 1 {
                while typed_string.len() > 4 {
                    typed_string.pop();
                }
                //println!("{}", add_tile_t.sprite_path);
                /*
                current_string = &mut add_tile_chr_buf;
                *current_string = format!("{}", typed_string.clone());

                */
                add_tile_chr_buf = copy_string(typed_string.clone());
            }

            game.screen
                .fill_rect(50, 150, 500, 100, engine::Color::BLACK)?;
            game.screen
                .draw_rect(50, 150, 500, 100, engine::Color::MAGENTA)?;
            game.screen.draw_string(
                51,
                151,
                String::from("Sprite Path"),
                engine::Color::WHITE,
                2,
            )?;
            game.screen
                .draw_rect(51, 167, 498, 10, engine::Color::RED)?;
            game.screen.draw_string(
                52,
                168,
                format!("{}", &add_tile_t.sprite_path),
                engine::Color::WHITE,
                1,
            )?;
            game.screen.draw_string(
                51,
                177,
                String::from("Sprite Char"),
                engine::Color::WHITE,
                2,
            )?;
            game.screen
                .draw_rect(51, 167 + 26, 16 * 4 + 2, 18, engine::Color::YELLOW)?;
            game.screen
                .draw_rect(51 + 16 * 4 + 2 + 5, 167 + 26, 18, 18, engine::Color::YELLOW)?;
            game.screen.draw_string(
                52,
                168 + 26,
                format!("{}", add_tile_chr_buf.clone()),
                engine::Color::WHITE,
                2,
            )?;
            game.screen.draw_string(
                52 + 16 * 4 + 2 + 5,
                168 + 26,
                format!("{}", add_tile_t.chr),
                engine::Color::WHITE,
                2,
            )?;
            match u32::from_str_radix(&add_tile_chr_buf, 16) {
                Ok(int) => {
                    add_tile_t.chr = match std::char::from_u32(int) {
                        Some(chr) => chr,
                        None => std::char::from_u32(0xFFFD).unwrap(),
                    };
                }
                _ => {}
            };
            if !typing && add_tile_field < 3 {
                typing = true;
                add_tile_field += 1;
                if add_tile_field > 2 {
                    typing = false;
                }
            }
            game.screen.draw_string(
                0,
                0,
                format!("{}", add_tile_field),
                engine::Color::WHITE,
                1,
            )?;
        }

        if !typing && add_tile && game.is_pressed(Keycode::D) {
            add_tile = false;
        }
        // END OF ADD TILE
        game.screen.update();
        // WRITE YOUR CODE HRE
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let mut game: engine::Engine<'static> =
        engine::Engine::new("FPS Map Editor", (600, 600, 1), &game_logic)
            .map_err(|err| err.to_string())?;
    game.start()?;
    Ok(())
}

fn str_normalize(source: String) -> String {
    let mut res: String = String::new();
    for chr in source.chars() {
        if res.len() < 10 {
            res.push(chr);
        }
    }
    res
}

fn normalize(source: engine::Keycode) -> String {
    use engine::Keycode::*;
    match source {
        A | B | C | D | E | F | G | H | I | J | K | L | M | N | O | P | Q | R | S | T | U | V
        | W | X | Y | Z | Underscore | Minus | Slash | Backslash | Num0 | Num1 | Num2 | Num3
        | Num4 | Num5 | Num6 | Num7 | Num8 | Num9 | Period | Semicolon | Exclaim | Colon
        | Comma | Asterisk | Dollar | RightParen | Equals | Less => source.name(),
        Space => String::from(" "),
        Kp0 | Kp1 | Kp2 | Kp3 | Kp4 | Kp5 | Kp6 | Kp7 | Kp8 | Kp9 | KpPeriod | KpDivide
        | KpPlus | KpMinus | KpMultiply => source.name().split_at(7).1.to_owned(),
        _ => {
            return if source.name().len() == 1 {
                //println!("char used _ => {{}} (Keycode::{:?})", source);
                source.name()
            } else {
                //println!("unknow char for : Keycode::{:?}", source);
                String::from("")
            };
        }
    }
}

fn copy_string(source: String) -> String {
    let mut res = String::new();
    for chr in source.chars() {
        res.push((chr).clone());
    }
    res
}

fn change_char(source: String, chr: char, index: usize) -> String {
    let mut res = String::new();
    let mut c_index = 0_usize;
    for c in source.chars() {
        if c_index == index {
            res.push(chr);
        } else {
            res.push(c);
        }
        c_index += 1;
    }
    res
}
