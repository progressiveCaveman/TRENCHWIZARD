use std::cmp;

use engine::{colors::{Color, self}, map::{XY, Map}};
use rltk::Point;
use shipyard::World;

use crate::{
    assets::{Image, Assets}, WIDTH, game::Game,
};

use self::console::{Console, ConsoleMode};

pub mod console;
pub mod menu_config;

pub const MAX_ZOOM: i32 = 16;
const UI_GLYPH_SIZE: i32 = 12;
const DEBUG_OUTLINES: bool = false;

pub struct Screen { // todo include frame here, set in window loop, prop to consoles?, remove from functions
    pub size: XY,
    pub input_blocking: bool,
    pub consoles: Vec<Console>,
    pub mouse_pos: XY,
}

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub pos: XY,
    pub ch: usize,
    pub fg: Color,
    pub bg: Color,
    pub gsize: i32,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RangedTargetResult {
    Cancel,
    NoResponse,
    NewTarget{ target: XY },
    Selected,
}

impl Screen {
    pub fn new(size: XY) -> Self {
        Self {
            size,
            input_blocking: false,
            consoles: Vec::new(),
            mouse_pos: (0, 0),
        }
    }

    pub fn setup_consoles(&mut self) {
        // TODO why are these -1 necessary? 

        // info console
        let xinfo = 0;
        let yinfo = 0;
        let winfo = 30 * UI_GLYPH_SIZE - 4; //todo this has to be 4. Gonna cause issues down the line but idk
        let hinfo = 10 * UI_GLYPH_SIZE;
        self.consoles.push(Console::new((winfo, hinfo), (xinfo, yinfo), ConsoleMode::Info));

        // context console
        let xcontext = 0;
        let ycontext = hinfo + 1;
        let wcontext = winfo;
        let hcontext = self.size.1 - ycontext - UI_GLYPH_SIZE;
        self.consoles.push(Console::new((wcontext, hcontext), (xcontext, ycontext), ConsoleMode::Context));

        // log console
        let xlog = winfo + 1;
        let ylog = 0;
        let wlog = self.size.0 - xlog - UI_GLYPH_SIZE;
        let hlog = hinfo;
        self.consoles.push(Console::new((wlog, hlog), (xlog, ylog), ConsoleMode::Log));

        // world console
        let xworld = winfo + 1;
        let yworld = hinfo + 1;
        let wworld = self.size.0 - xworld;
        let hworld = self.size.1 - yworld;
        self.consoles.push(Console::new((wworld, hworld), (xworld, yworld), ConsoleMode::WorldMap));

        // menu console
        let wmenu = UI_GLYPH_SIZE * 30;
        let hmenu = UI_GLYPH_SIZE * 20;
        let xmenu = self.size.0/2 - wmenu/2;
        let ymenu = self.size.1/2 - hmenu/2;
        self.consoles.push(Console::new((wmenu, hmenu), (xmenu, ymenu), ConsoleMode::MainMenu));

        // inventory console
        let wmenu = UI_GLYPH_SIZE * 40;
        let hmenu = UI_GLYPH_SIZE * 30;
        let xmenu = self.size.0 - wmenu - UI_GLYPH_SIZE;
        let ymenu = hinfo;
        self.consoles.push(Console::new((wmenu, hmenu), (xmenu, ymenu), ConsoleMode::Inventory));

        // item info console
        let wmenu = UI_GLYPH_SIZE * 30;
        let hmenu = UI_GLYPH_SIZE * 10;
        let xmenu = self.size.0/2 - wmenu/2;
        let ymenu = hinfo;
        self.consoles.push(Console::new((wmenu, hmenu), (xmenu, ymenu), ConsoleMode::ItemInfo));
    }

    pub fn autozoom_world_map(&mut self, map: &Map) {
        for con in self.consoles.iter_mut() {
            if con.mode == ConsoleMode::WorldMap {
                con.zoom_to_fit(map);
            }
        }
    }

    pub fn get_map_console(&self) -> Option<&Console> { 
        for c in self.consoles.iter() {
            if c.mode == ConsoleMode::WorldMap {
                return Some(&c);
            }
        }

        None
    }

    pub fn ranged_target(&mut self, frame: &mut [u8], assets: &Assets, world: &mut World, range: i32, clicked: bool, target: XY) -> (RangedTargetResult, Option<Point>) {
        let map_mouse_pos = self.get_mouse_game_pos();
        for c in self.consoles.iter_mut() {
            if c.mode == ConsoleMode::WorldMap {
                return c.ranged_target(frame, assets, world, map_mouse_pos, range, clicked, target);
            }
        }

        unreachable!()
    }

    pub fn increment_zoom(&mut self) {
        for con in self.consoles.iter_mut() {
            if con.mode == ConsoleMode::WorldMap {
                con.zoom_in();
            }
        }
    }

    pub fn decrement_zoom(&mut self) {
        for con in self.consoles.iter_mut() {
            if con.mode == ConsoleMode::WorldMap {
                con.zoom_out();
            }
        }
    }

    pub fn pan_map(&mut self, offset: (i32, i32)) {
        self.consoles[1].map_pos = {
            let mp = self.consoles[1].map_pos;
            (
                cmp::max(
                    0, 
                    mp.0 + offset.0,
                ),
                cmp::max(
                    0, 
                    mp.1 + offset.1,
                ),
            )
        };
    }

    pub fn get_mouse_game_pos(&self) -> XY {
        return self.screen_pos_to_map(self.mouse_pos);
    }

    pub fn screen_pos_to_map(&self, screen_pos: XY) -> XY {
        if let Some(console) = self.get_map_console() {
            return (
                (screen_pos.0 - console.pos.0) / console.gsize + console.map_pos.0,
                (screen_pos.1 - console.pos.1) / console.gsize + console.map_pos.1,
            );
        }

        (0,0)
    }

    pub fn map_pos_to_screen(&self, map_pos: XY) -> XY {
        if let Some(console) = self.get_map_console() {
            return (
                (map_pos.0 - console.map_pos.0) * console.gsize + console.pos.0,
                (map_pos.1 - console.map_pos.1) * console.gsize + console.pos.1,
            );
        }

        (0,0)
    }

    pub fn draw(&self, frame: &mut [u8], game: &Game) {
        // clear screen
        for (_, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba = [0, 0, 0, 0];
            pixel.copy_from_slice(&rgba);
        }

        for c in self.consoles.iter() {
            if !c.hidden {
                c.render(frame, game);
                if DEBUG_OUTLINES {
                    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                        let xscreen = i as i32 % WIDTH;
                        let yscreen = i as i32 / WIDTH;
        
                        if c.in_bounds((xscreen, yscreen)) &&
                            (xscreen == c.pos.0 || 
                            xscreen == c.pos.0 + c.size.0 ||
                            yscreen == c.pos.1 ||
                            yscreen == c.pos.1 + c.size.1 )
                        {
                            pixel.copy_from_slice(&colors::COLOR_PURPLE);               
                        }
                    }
                }
            }
        }
    }

    pub fn draw_image(&self, image: &Image, frame: &mut [u8], pos: XY) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let image_buf = &image.0;
            let size = image.1;

            let xscreen = i as i32 % WIDTH;
            let yscreen = i as i32 / WIDTH;

            let xrange = pos.0 .. pos.0 + self.size.0;
            let yrange = pos.1 .. pos.1 + self.size.1;

            if xrange.contains(&xscreen) && yrange.contains(&yscreen) {
                let ximg = xscreen - pos.0;
                let yimg = yscreen - pos.1;

                let idx = yimg * size.1 + ximg;
                let rgba = image_buf[idx as usize];

                pixel.copy_from_slice(&rgba);
            }
        }
    }

    // pub fn highlight_map_coord(&mut self, frame: &mut [u8], game: &Game, map_pos: XY, mut color: Color) {
    //     // let xmap = self.map_offset.0 + (xscreen - self.pos.0) / self.gsize;
        
    //     let mapcon = self.get_map_console().unwrap();
    //     // set alpha
    //     color[3] = 128;

    //     let pos = (
    //         mapcon.pos.0 + (map_pos.0 - mapcon.map_pos.0) * mapcon.gsize, 
    //         mapcon.pos.1 + (map_pos.1 - mapcon.map_pos.1) * mapcon.gsize, 
    //     );

    //     let glyph = Glyph {
    //         pos,
    //         ch: to_cp437(' '),
    //         fg: colors::COLOR_CLEAR,
    //         bg: color,
    //     };

    //     self.print_cp437(&game.assets, frame, glyph, UI_GLYPH_SIZE);
    // }

    // /// Draw a line to the pixel buffer using Bresenham's algorithm.
    // pub(crate) fn line(screen: &mut [u8], p1: &Point, p2: &Point, color: [u8; 4]) {
    //     let p1 = (p1.x as i64, p1.y as i64);
    //     let p2 = (p2.x as i64, p2.y as i64);

    //     for (x, y) in Bresenham::new(p1, p2) {
    //         let x = min(x as usize, WIDTH - 1);
    //         let y = min(y as usize, HEIGHT - 1);
    //         let i = x * 4 + y * WIDTH * 4;

    //         screen[i..i + 4].copy_from_slice(&color);
    //     }
    // }

    // /// Draw a rectangle to the pixel buffer using two points in opposite corners.
    // pub(crate) fn rect(screen: &mut [u8], p1: &Point, p2: &Point, color: [u8; 4]) {
    //     let p2 = Point::new(p2.x - 1, p2.y - 1);
    //     let p3 = Point::new(p1.x, p2.y);
    //     let p4 = Point::new(p2.x, p1.y);

    //     line(screen, p1, &p3, color);
    //     line(screen, &p3, &p2, color);
    //     line(screen, &p2, &p4, color);
    //     line(screen, &p4, p1, color);
    // }

    
}
