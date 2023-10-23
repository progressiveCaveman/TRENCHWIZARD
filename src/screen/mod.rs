use std::{iter::zip, cmp};

use engine::{colors::{Color, self}, map::{XY, Map}};

use crate::{
    assets::{
        cp437_converter::string_to_cp437,
        Assets, sprites::Drawable, Image,
    },
    Game, HEIGHT, WIDTH,
};

use self::console::{Console, ConsoleMode};

pub mod console;
pub mod menu_config;

pub const MAX_ZOOM: i32 = 16;
const UI_GLYPH_SIZE: i32 = 12;
const DEBUG_OUTLINES: bool = false;

pub struct Screen {
    pub size: XY,
    pub input_blocking: bool,
    consoles: Vec<Console>,
}

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub pos: XY,
    pub ch: usize,
    pub fg: Color,
    pub bg: Color,
}

impl Screen {
    pub fn new(size: XY) -> Self {
        Self {
            size,
            input_blocking: false,
            consoles: Vec::new(),
        }
    }

    pub fn setup_consoles(&mut self) {

        // info console
        let x = 0;
        let y = 0;
        let w = self.size.0 / 4;
        let h = 10 * UI_GLYPH_SIZE - 1;
        self.consoles.push(Console::new((w, h), (x, y), ConsoleMode::Info));

        // log console
        let x = w;
        let y = 0;
        let w = self.size.0 - w - 1;
        let h = h;
        self.consoles.push(Console::new((w, h), (x, y), ConsoleMode::Log));

        // world console
        let x = 0;
        let y = h;
        let w = self.size.0 - 1;
        let h = self.size.1 - h - 1;
        self.consoles.push(Console::new((w, h), (x, y), ConsoleMode::WorldMap));

        // menu console
        let w = UI_GLYPH_SIZE * 30;
        let h = UI_GLYPH_SIZE * 20;
        let x = self.size.0/2 - w/2;
        let y = self.size.1/2 - h/2;
        self.consoles.push(Console::new((w, h), (x, y), ConsoleMode::MainMenu));
    }

    pub fn autozoomn_world_map(&mut self, map: &Map) {
        for con in self.consoles.iter_mut() {
            if con.mode == ConsoleMode::WorldMap {
                con.zoom_to_fit(map);
            }
        }
    }

    // pub fn set_main_console_mode(&mut self, mode: ConsoleMode) {
    //     self.consoles[1].mode = mode;
    // }

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

    pub fn draw(&self, frame: &mut [u8], game: &Game) {
        // clear screen
        for (_, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba = [0x00, 0x00, 0x00, 0x00];
            pixel.copy_from_slice(&rgba);
        }

        for c in self.consoles.iter() {
            if !c.hidden {
                c.render(frame, game);
            }
        }
    }

    pub fn print_cp437(&self, assets: &Assets, frame: &mut [u8], glyph: Glyph, zoom: i32) {
        // let sprite = &assets.glyph(glyph);
        Screen::blit_glyph(frame, assets, glyph.pos, glyph, zoom);
    }

    pub fn print_string(&self, assets: &Assets, frame: &mut [u8], str: &str, pos: XY, color: Color, zoom: i32) {
        let chars = string_to_cp437(str);

        for (idx, ch) in chars.iter().enumerate() {
            self.print_cp437(assets, frame, Glyph { 
                pos: (pos.0 + idx as i32 * UI_GLYPH_SIZE, pos.1),
                ch: *ch, 
                fg: color, 
                bg: colors::COLOR_CLEAR 
            }, zoom);
            // let sprite = &assets.cp437[*ch as usize];
            // Screen::blit(
            //     frame,
            //     Point {
            //         x: pos.x + idx * 8,
            //         y: pos.y,
            //     },
            //     sprite,
            // );
        }
    }

    pub fn draw_box(&self, assets: &Assets, frame: &mut [u8], pos: XY, size: XY, fg: Color, bg: Color, zoom: i32) {
        let vertwall = 186;
        let horizwall = 205;
        let nwcorner = 201;
        let necorner = 187;
        let secorner = 188;
        let swcorner = 200;

        let gsize = UI_GLYPH_SIZE;

        for x in (pos.0 .. pos.0 + size.0).step_by(gsize as usize) {
            for y in (pos.1 .. pos.1 + size.1).step_by(gsize as usize) {
                let firstcolumn = x < pos.0 + gsize;
                let lastcolumn = x >= pos.0 + size.0 - gsize;
                let firstrow = y < pos.1 + gsize;
                let lastrow = y >= pos.1 + size.1 - gsize;

                let ch = if firstrow && firstcolumn {
                    nwcorner
                } else if firstrow && lastcolumn {
                    necorner
                } else if lastrow && firstcolumn {
                    swcorner
                } else if lastrow && lastcolumn {
                    secorner
                } else if firstrow || lastrow {
                    horizwall
                } else if firstcolumn || lastcolumn {
                    vertwall
                } else {
                    0
                };

                self.print_cp437(assets, frame, Glyph { pos: (x, y), ch, fg, bg }, zoom);
            }
        }

        // if x < 1 || x > map.width-2 || y < 1 || y > map.height-2 as i32 { return 35; }
        // let mut mask : u8 = 0;

        // if is_revealed_and_wall(map, x, y - 1) { mask +=1; }
        // if is_revealed_and_wall(map, x, y + 1) { mask +=2; }
        // if is_revealed_and_wall(map, x - 1, y) { mask +=4; }
        // if is_revealed_and_wall(map, x + 1, y) { mask +=8; }

        // match mask {
        //     0 => { 9 } // Pillar because we can't see neighbors
        //     1 => { 186 } // Wall only to the north
        //     2 => { 186 } // Wall only to the south
        //     3 => { 186 } // Wall to the north and south
        //     4 => { 205 } // Wall only to the west
        //     5 => { 188 } // Wall to the north and west
        //     6 => { 187 } // Wall to the south and west
        //     7 => { 185 } // Wall to the north, south and west
        //     8 => { 205 } // Wall only to the east
        //     9 => { 200 } // Wall to the north and east
        //     10 => { 201 } // Wall to the south and east
        //     11 => { 204 } // Wall to the north, south and east
        //     12 => { 205 } // Wall to the east and west
        //     13 => { 202 } // Wall to the east, west, and south
        //     14 => { 203 } // Wall to the east, west, and north
        //     15 => { 206 }  // ╬ Wall on all sides
        //     _ => { 35 } // We missed one?
        // }
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

    /// Blit a drawable to the pixel buffer. Assumes glyph asset has fuscia bg and grayscale fg
    pub fn blit_glyph(frame: &mut [u8], assets: &Assets, dest: XY, glyph: Glyph, zoom: i32) {
        let mut spritesheet = &assets.cp437[0];
        for ss in assets.cp437.iter() {
            if zoom == ss.0 as i32 {
                spritesheet = ss;
            } else if zoom < ss.0 as i32 {
                break;
            }
        }

        let sprite = &spritesheet.1[glyph.ch as usize];

        assert!(dest.0 + sprite.width() as i32 <= WIDTH);
        assert!(dest.1 + sprite.height() as i32 <= HEIGHT);

        let pixels = sprite.pixels();
        let width = sprite.width() * 4;

        let mut s = 0;
        for y in 0..sprite.height() {
            let i = (dest.0 * 4 + dest.1 * WIDTH * 4 + y as i32 * WIDTH * 4) as usize;

            let zipped = zip(
                frame[i..i + width].chunks_exact_mut(4),
                pixels[s..s + width].chunks_exact(4),
            );

            for (left, right) in zipped {
                // set color
                for i2 in 0..4 {
                    if right == colors::COLOR_FUCHSIA { // background
                        left[i2] = glyph.bg[i2];
                    } else { // foreground
                        left[i2] = (right[i2] as f32 * glyph.fg[i2] as f32 / 255 as f32) as u8;
                    }
                }
            }

            s += width;
        }
    }

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
