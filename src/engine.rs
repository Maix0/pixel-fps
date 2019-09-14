//! A library to sweeten the interaction with sdl2
#[deny(
    missing_docs,
    missing_debug_implementations,
//  missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
// use rayon::prelude::*;
pub use sdl2;

/// The core of this library
/// Engine handle the backend of drawing and deals with sdl2
// #[derive(Send)]

pub struct Engine<'a> {
    /// The name the engine
    pub name: String,
    /// The size of the Window, formated (width,height,pixel_size)
    pub size: (u32, u32, u32),
    //pub main: (dyn Fn(f64) -> Result<(),&'a String>),
    /// The window , where the drawing methods are
    pub screen: Window,
    event_pump: sdl2::EventPump,
    timer: std::time::SystemTime, // DEFAULTED TO 0
    frame_count: u64,             // DEFAULTED TO 0
    frame_timer: f64,             // DEFAULTED TO 0
    /// the time elapsed between the start of this frame and the old one
    pub elapsed: f64, // DEFAULTED TO 0
    key_pressed: std::collections::HashSet<sdl2::keyboard::Keycode>,
    key_hold: std::collections::HashSet<sdl2::keyboard::Keycode>,
    key_release: std::collections::HashSet<sdl2::keyboard::Keycode>,
    /// The user code run by the Engine
    pub main: &'a (dyn Fn(&mut Engine) -> Result<(), String>),
}

impl<'a> std::fmt::Debug for Engine<'a> {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        return write!(fmt, "Engine {{ pub name: {:?}, pub size: {:?}, pub screen: {:?}, event_pump: sdl2::EventPump, timer: {:?}, frame_count: {:?}, frame_timer: {:?}, pub elapsed: {:?}, key_pressed: {:?}, key_hold: {:?}, key_release: {:?}, main: &(dyn Fn(&mut Engine) -> Result<(),String>) }}",
            self.name,
            self.size,
            self.screen,
            self.timer,
            self.frame_count,
            self.frame_timer,
            self.elapsed,
            self.key_pressed,
            self.key_hold,
            self.key_release
        );
    }
}

impl<'a> Engine<'a> {
    /// Create a new game Engine
    pub fn new(
        name: &str,
        size: (u32, u32, u32),
        main: &'a (dyn Fn(&mut Engine) -> Result<(), String>),
    ) -> Result<Engine<'a>, String> {
        let sdl_ctx = sdl2::init()?;
        let video_subsystem = sdl_ctx.video()?;
        let screen = initialize(video_subsystem, size.0, size.1, size.2, String::from(name))
            .map_err(|err| err.to_string())?;
        let event_pump = sdl_ctx.event_pump()?;
        Ok(Engine {
            name: String::from(name),
            size: size,
            main: main,
            screen: screen,
            event_pump: event_pump,
            timer: std::time::SystemTime::now(),
            frame_count: 0,
            frame_timer: 0.0,
            elapsed: 0.0,
            key_pressed: std::collections::HashSet::new(),
            key_hold: std::collections::HashSet::new(),
            key_release: std::collections::HashSet::new(),
        })
    }
    /// The core of the engine, where crucial value are calculated, like elapsed or the key's
    /// vector
    pub fn new_frame(&mut self) -> Result<bool, String> {
        // Calcaulting elapsed_time
        self.elapsed = (std::time::SystemTime::now()
            .duration_since(self.timer)
            .map_err(|err| err.to_string())?)
        .as_secs_f64();
        // End
        self.timer = std::time::SystemTime::now();
        self.frame_timer += self.elapsed;
        self.frame_count += 1;
        if self.frame_timer > 1.0 {
            self.frame_timer -= 1.0;
            self.screen
                .update_title(&self.name.as_ref(), self.frame_count)?;
            self.frame_count = 0
        }

        for key in &self.key_pressed {
            self.key_hold.insert(*key);
        }
        self.key_pressed.clear();
        self.key_release.clear();

        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} /*| sdl2::event::Event::KeyDown {keycode: Some(sdl2::keyboard::Keycode::Escape), .. }*/ => return Ok(false),
                sdl2::event::Event::KeyDown { keycode: Some(keycode), repeat, ..} => {
                    if !repeat {
                        self.key_pressed.insert(keycode);
                    }
                },
                sdl2::event::Event::KeyUp {keycode: Some(keycode), ..} => {
                    self.key_release.insert(keycode);
                    self.key_hold.remove(&keycode);
                    self.key_pressed.remove(&keycode);
                }
                _ => {},
            }
        }
        Ok(true)
    }
    /// Start the engine, lanch the "main" function
    pub fn start(&mut self) -> Result<(), String> {
        (self.main)(self).map_err(|err| err.to_string())?;
        Ok(())
    }
    /// Return the state of "key" via KeyState enum
    pub fn get_key_state(&self, key: sdl2::keyboard::Keycode) -> KeyState {
        if self.key_pressed.contains(&key) {
            return KeyState::Pressed;
        }
        if self.key_hold.contains(&key) {
            return KeyState::Held;
        }
        if self.key_release.contains(&key) {
            return KeyState::Released;
        }
        return KeyState::None;
    }
    ///Return a clone of the vector conataining pressed keys
    pub fn get_pressed(&self) -> std::collections::HashSet<sdl2::keyboard::Keycode> {
        self.key_pressed.clone()
    }
    ///Return a clone of the vector conataining held keys
    pub fn get_held(&self) -> std::collections::HashSet<sdl2::keyboard::Keycode> {
        self.key_hold.clone()
    }
    ///Return a clone of the vector conataining released keys
    pub fn get_released(&self) -> std::collections::HashSet<sdl2::keyboard::Keycode> {
        self.key_release.clone()
    }
    /// Return true if "key" is pressed this frame, else, return false
    pub fn is_pressed(&self, key: sdl2::keyboard::Keycode) -> bool {
        self.key_pressed.contains(&key)
    }
    /// Return true if "key" is held this frame, else, return false
    pub fn is_held(&self, key: sdl2::keyboard::Keycode) -> bool {
        self.key_hold.contains(&key)
    }
    /// Return true if "key" is released this frame, else , return false
    pub fn is_released(&self, key: sdl2::keyboard::Keycode) -> bool {
        self.key_release.contains(&key)
    }
}
/// All the keystate possible
#[derive(Eq, PartialEq, Debug)]
pub enum KeyState {
    ///The key is not pressed nor released nor held
    None,
    ///The key is held
    Held,
    ///The key is pressed
    Pressed,
    ///The key is released
    Released,
}

// DRAW STUFF

use sdl2::rect::Rect;

// ==========START=COLOR==========
#[derive(Debug, Copy, Clone)]
/// A RGBA color interface
pub struct Color {
    /// Red Channel
    pub r: u8,
    /// Green Channel
    pub g: u8,
    /// Blue Channel
    pub b: u8,
    /// Alpha Channel
    pub a: u8,
}

impl Color {
    /// Color
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    /// Color
    pub const GREY: Color = Color {
        r: 192,
        g: 192,
        b: 192,
        a: 255,
    };
    /// Color
    pub const DARK_GREY: Color = Color {
        r: 128,
        g: 128,
        b: 128,
        a: 255,
    };
    /// Color
    pub const VERY_DARK_GREY: Color = Color {
        r: 64,
        g: 64,
        b: 64,
        a: 255,
    };
    /// Color
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Color
    pub const DARK_RED: Color = Color {
        r: 128,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Color
    pub const VERY_DARK_RED: Color = Color {
        r: 64,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Color
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };
    /// Color
    pub const DARK_YELLOW: Color = Color {
        r: 128,
        g: 128,
        b: 0,
        a: 255,
    };
    /// Color
    pub const VERY_DARK_YELLOW: Color = Color {
        r: 64,
        g: 64,
        b: 0,
        a: 255,
    };
    /// Color
    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    /// Color
    pub const DARK_GREEN: Color = Color {
        r: 0,
        g: 128,
        b: 0,
        a: 255,
    };
    /// Color
    pub const VERY_DARK_GREEN: Color = Color {
        r: 0,
        g: 64,
        b: 0,
        a: 255,
    };
    /// Color
    pub const CYAN: Color = Color {
        r: 0,
        g: 255,
        b: 255,
        a: 255,
    };
    /// Color
    pub const DARK_CYAN: Color = Color {
        r: 0,
        g: 128,
        b: 128,
        a: 255,
    };
    /// Color
    pub const VERY_DARK_CYAN: Color = Color {
        r: 0,
        g: 64,
        b: 64,
        a: 255,
    };
    /// Color
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    /// Color
    pub const DARK_BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 128,
        a: 255,
    };
    /// Color
    pub const VERY_DARK_BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 64,
        a: 255,
    };
    /// Color
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
        a: 255,
    };
    /// Color
    pub const DARK_MAGENTA: Color = Color {
        r: 128,
        g: 0,
        b: 128,
        a: 255,
    };
    /// Color
    pub const VERY_DARK_MAGENTA: Color = Color {
        r: 64,
        g: 0,
        b: 64,
        a: 255,
    };
    /// Color
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Create a new Color with the alpha set at 255
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }
    /// Create a new Color with the alpha as an additinal argument
    pub fn new_alpha(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
    /// Friendly way to transphorm Color to sdl2::pixels::Color
    pub fn to_pixel_color(&self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGBA(self.r, self.g, self.b, self.a)
    }
}
// ==========END===COLOR==========

// =========START=WINDOW==========
/// Base struct of the library
/// contains the function to draw onto the window's canvas
pub struct Window {
    /// The canvas's Window, not realy useful
    pub canv: sdl2::render::Canvas<sdl2::video::Window>,
    /// The Window's height, if ever useful
    pub height: u32,
    /// The Window's width, if ever useful
    pub width: u32,
    /// The Window pixel ratio, "1 pixel = px_size pixels" on the physical screen
    pub px_size: u32,
    /// The Window name
    pub name: String,
}

impl std::fmt::Debug for Window {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        return write!(fmt, "Window {{ pub self.canv: sdl2::render::Canvas<sdl2::video::Window>, pub height: {:?}, pub width: {:?}, pub px_size: {:?}, pub name: {:?} }}",
            self.height,
            self.width,
            self.px_size,
            self.name
            );
    }
}

impl Window {
    /// Create a new window from arguments given
    pub fn new(
        canvas: sdl2::render::Canvas<sdl2::video::Window>,
        w: u32,
        h: u32,
        pixel_size: u32,
        name: String,
    ) -> Window {
        Window {
            canv: canvas,
            width: w,
            height: h,
            px_size: pixel_size,
            name: name,
        }
    }
    fn ptsp(&self, num: i32) -> i32 {
        // fn point_to_screen_point()
        ((num/* - 1*/) * self.px_size as i32)
    }
    /// Fill the whole screen with color given
    pub fn clear(&mut self, col: Color) {
        self.canv.set_draw_color(col.to_pixel_color());
        self.canv.clear();
    }
    /// Draw a pixel at x,y
    pub fn draw(&mut self, x: i32, y: i32, col: Color) -> Result<(), String> {
        self.canv.set_draw_color(col.to_pixel_color());
        self.canv
            .fill_rect(Rect::new(
                self.ptsp(x),
                self.ptsp(y),
                self.px_size,
                self.px_size,
            ))
            .map_err(|err| err.to_string())?;
        Ok(())
    }
    /// Draw a filled rectangle starting a x,y with size w,h
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, col: Color) -> Result<(), String> {
        self.canv.set_draw_color(col.to_pixel_color());
        self.canv
            .fill_rect(Rect::new(
                self.ptsp(x),
                self.ptsp(y),
                self.ptsp(w) as u32,
                self.ptsp(h) as u32,
            ))
            .map_err(|err| err.to_string())?;
        Ok(())
    }
    /// Update the window's canvas to show change
    pub fn update(&mut self) {
        self.canv.present();
    }
    /// Update the title of the window with format "{name} - {fps}fps"
    pub fn update_title(&mut self, title: &str, frame_count: u64) -> Result<(), String> {
        self.canv
            .window_mut()
            .set_title(format!("{} - {}fps", title, frame_count).as_ref())
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}
// =========END===WINDOW==========
/// Create a window with the name ,size and pixel size given
pub fn initialize(
    video_subsystem: sdl2::VideoSubsystem,
    width: u32,
    height: u32,
    pixsize: u32,
    name: String,
) -> Result<Window, String> {
    let window = video_subsystem
        .window(name.as_ref(), width * pixsize, height * pixsize)
        .opengl()
        .build()
        .map_err(|err| err.to_string())?;
    let canvas: sdl2::render::Canvas<sdl2::video::Window> = window
        .into_canvas()
        .build()
        .map_err(|err| err.to_string())?;
    Ok(Window::new(canvas, width, height, pixsize, name))
}
