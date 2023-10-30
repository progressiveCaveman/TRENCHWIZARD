use std::{iter::zip, cmp};

use engine::{colors::{Color, self}, map::{XY, Map}, components::{PlayerID, PPoint, Vision}, utils::InvalidPoint};
use rltk::Point;
use shipyard::{UniqueView, View, Get};

use crate::{
    assets::{
        cp437_converter::{string_to_cp437, to_cp437},
        Assets, sprites::Drawable, Image,
    },
    HEIGHT, WIDTH, game::Game,
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
    // pub size: i32,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RangedTargetResult {
    Cancel,
    NoResponse,
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

    pub fn autozoomn_world_map(&mut self, map: &Map) {
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

    pub fn get_mouse_game_pos(&self) -> XY {
        if let Some(console) = self.get_map_console() {
            let mp_in_map_console = (
                self.mouse_pos.0 - console.pos.0,
                self.mouse_pos.1 - console.pos.1
            );

            let tile_mp = (
                mp_in_map_console.0 / console.gsize,
                mp_in_map_console.1 / console.gsize
            );

            return (
                console.map_pos.0 + tile_mp.0,
                console.map_pos.1 + tile_mp.1
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

    pub fn print_cp437(&self, assets: &Assets, frame: &mut [u8], glyph: Glyph, gsize: i32) {
        if glyph.pos.1 >= self.size.1 - gsize || glyph.pos.0 >= self.size.0 - gsize {
            return;
        }

        self.blit_glyph(frame, assets, glyph.pos, glyph, gsize);
    }

    pub fn print_string(&self, assets: &Assets, frame: &mut [u8], str: &str, pos: XY, color: Color, gsize: i32) {
        let chars = string_to_cp437(str);

        for (idx, ch) in chars.iter().enumerate() {
            self.print_cp437(assets, frame, Glyph { 
                pos: (pos.0 + idx as i32 * gsize, pos.1),
                ch: *ch, 
                fg: color, 
                bg: colors::COLOR_CLEAR 
            }, gsize);
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

    pub fn draw_box(&self, assets: &Assets, frame: &mut [u8], pos: XY, size: XY, fg: Color, bg: Color, gsize: i32) {
        let vertwall = 186;
        let horizwall = 205;
        let nwcorner = 201;
        let necorner = 187;
        let secorner = 188;
        let swcorner = 200;

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

                self.print_cp437(assets, frame, Glyph { pos: (x, y), ch, fg, bg }, gsize);
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

    pub fn highlight_map_coord(&mut self, frame: &mut [u8], game: &Game, map_pos: XY, mut color: Color) {
        // let xmap = self.map_offset.0 + (xscreen - self.pos.0) / self.gsize;
        
        let mapcon = self.get_map_console().unwrap();
        // set alpha
        color[3] = 128;

        let pos = (
            mapcon.pos.0 + (map_pos.0 - mapcon.map_pos.0) * mapcon.gsize, 
            mapcon.pos.1 + (map_pos.1 - mapcon.map_pos.1) * mapcon.gsize, 
        );

        let glyph = Glyph {
            pos,
            ch: to_cp437(' '),
            fg: colors::COLOR_CLEAR,
            bg: color,
        };

        self.print_cp437(&game.assets, frame, glyph, UI_GLYPH_SIZE);
    }

    /// Blit a drawable to the pixel buffer. 
    /// Assumes glyph asset has fuscia bg and grayscale fg
    pub fn blit_glyph(&self, frame: &mut [u8], assets: &Assets, dest: XY, glyph: Glyph, gsize: i32) {
        let mut spritesheet = &assets.cp437[0];
        for ss in assets.cp437.iter() {
            if gsize == ss.0 as i32 {
                spritesheet = ss;
            } else if gsize < ss.0 as i32 {
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

            for (left, right) in zipped { // left is screen frame, right is glyph
                // set color
                for i2 in 0..4 {
                    if right == colors::COLOR_FUCHSIA { // background
                        left[i2] = left[i2] * (1 - glyph.bg[3]/255) + glyph.bg[i2];
                    } else { // foreground
                        left[i2] = left[i2] * (1 - glyph.fg[3]/255) + (right[i2] as f32 * glyph.fg[i2] as f32 / 255 as f32) as u8;
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

    
    pub fn ranged_target(&mut self, frame: &mut [u8], assets: &Assets, game: &mut Game, range: i32, clicked: bool) -> (RangedTargetResult, Option<Point>) {
        dbg!("ranged target");
        let world = &game.engine.world;
        let map = world.borrow::<UniqueView<Map>>().unwrap();
        let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;
        let player_pos = world.borrow::<UniqueView<PPoint>>().unwrap().0;
        let vvs = world.borrow::<View<Vision>>().unwrap();

        self.print_string(assets, frame, "Select a target", self.consoles[0].size, colors::COLOR_UI_1, UI_GLYPH_SIZE);
        // ctx.print_color(5, 12, colors::COLOR_UI_1, colors::COLOR_BG, "Select a target");

        // let (min_x, max_x, min_y, max_y) = get_map_coords_for_screen(player_pos, ctx, (map.width, map.height));

        // calculate valid cells
        let mut valid_cells: Vec<Point> = Vec::new();
        match vvs.get(player_id) {
            Err(_e) => return (RangedTargetResult::Cancel, None),
            Ok(player_vs) => {
                for pt in player_vs.visible_tiles.iter() {
                    let dist = rltk::DistanceAlg::Pythagoras.distance2d(player_pos, *pt);
                    if dist as i32 <= range { // tile within range
                        valid_cells.push(*pt);
                        self.highlight_map_coord(frame, game, pt.to_xy(), colors::COLOR_BLUE);


                        // let screen_x = pt.x - min_x + OFFSET_X as i32;
                        // let screen_y = pt.y - min_y + OFFSET_Y as i32; // TODO why is offset needed here??
                        // if screen_x > 1 && screen_x < (max_x - min_x) - 1 && screen_y > 1 && screen_y < (max_y - min_y) - 1
                        // {
                        //     ctx.set_bg(screen_x, screen_y, RGB::named(rltk::BLUE));
                        //     valid_cells.push(*pt);
                        // }
                        // ctx.set_bg(screen_x, screen_y, Palette::COLOR_4);
                        // valid_cells.push(*pt);
                    }
                }
            }
        }

        let map_mouse_pos = self.get_mouse_game_pos();

        // let mouse_pos = ctx.mouse_pos();
        // let mut map_mouse_pos = map.transform_mouse_pos(mouse_pos);
        // map_mouse_pos.0 += min_x;
        // map_mouse_pos.1 += min_y;
        // let map_mouse_pos = (mouse_pos.0 - map::OFFSET_X as i32, mouse_pos.1 - map::OFFSET_Y as i32);
        let mut valid_target = false;
        for pt in valid_cells.iter() {
            if pt.x == map_mouse_pos.0 && pt.y == map_mouse_pos.1 {
                valid_target = true;
                break;
            }
        }
        if valid_target {
            self.highlight_map_coord(frame, game, map_mouse_pos, colors::COLOR_DARK_GREEN);
            // ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_GREEN_DARK);

            // if ctx.left_click {
            //     return (
            //         RangedTargetResult::Selected,
            //         Some(Point::new(map_mouse_pos.0, map_mouse_pos.1)),
            //     );
            // }
            if clicked {
                return (RangedTargetResult::Selected, Some(Point::new(map_mouse_pos.0, map_mouse_pos.1)));
            }
        } else {
            self.highlight_map_coord(frame, game, map_mouse_pos, colors::COLOR_RED);
            // ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_RED);
            if clicked {
                return (RangedTargetResult::Cancel, None);
            }
        }

        (RangedTargetResult::NoResponse, None)

        // match ctx.key {
        //     None => (RangedTargetResult::NoResponse, None),
        //     Some(key) => match key {
        //         VirtualKeyCode::Escape => return (RangedTargetResult::Cancel, None),
        //         _ => (RangedTargetResult::NoResponse, None),
        //     },
        // }
    }


    /*
    pub fn ranged_target(world: &World, ctx: &mut Rltk, range: i32) -> (RangedTargetResult, Option<Point>) {
        let map = world.borrow::<UniqueView<Map>>().unwrap();
        let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;
        let player_pos = world.borrow::<UniqueView<PPoint>>().unwrap().0;
        ctx.print_color(5, 12, Palette::COLOR_PURPLE, Palette::MAIN_BG, "Select a target");

        let (min_x, max_x, min_y, max_y) = get_map_coords_for_screen(player_pos, ctx, (map.width, map.height));

        let mut valid_cells: Vec<Point> = Vec::new();
        let vvs = world.borrow::<View<Vision>>().unwrap();
        match vvs.get(player_id) {
            Err(_e) => return (RangedTargetResult::Cancel, None),
            Ok(player_vs) => {
                for pt in player_vs.visible_tiles.iter() {
                    let dist = rltk::DistanceAlg::Pythagoras.distance2d(player_pos, *pt);
                    if dist as i32 <= range {
                        let screen_x = pt.x - min_x + OFFSET_X as i32;
                        let screen_y = pt.y - min_y + OFFSET_Y as i32; // TODO why is offset needed here??
                        if screen_x > 1 && screen_x < (max_x - min_x) - 1 && screen_y > 1 && screen_y < (max_y - min_y) - 1
                        {
                            ctx.set_bg(screen_x, screen_y, RGB::named(rltk::BLUE));
                            valid_cells.push(*pt);
                        }
                        ctx.set_bg(screen_x, screen_y, Palette::COLOR_4);
                        valid_cells.push(*pt);
                    }
                }
            }
        }

        let mouse_pos = ctx.mouse_pos();
        let mut mouse_map_pos = mouse_pos;
        mouse_map_pos.0 += min_x;
        mouse_map_pos.1 += min_y;

        let mouse_pos = ctx.mouse_pos();
        let mut map_mouse_pos = map.transform_mouse_pos(mouse_pos);
        map_mouse_pos.0 += min_x;
        map_mouse_pos.1 += min_y;
        // let map_mouse_pos = (mouse_pos.0 - map::OFFSET_X as i32, mouse_pos.1 - map::OFFSET_Y as i32);
        let mut valid_target = false;
        for pt in valid_cells.iter() {
            if pt.x == map_mouse_pos.0 && pt.y == map_mouse_pos.1 {
                valid_target = true
            }
        }
        if valid_target {
            ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_GREEN_DARK);
            if ctx.left_click {
                return (
                    RangedTargetResult::Selected,
                    Some(Point::new(map_mouse_pos.0, map_mouse_pos.1)),
                );
            }
        } else {
            ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_RED);
            if ctx.left_click {
                return (RangedTargetResult::Cancel, None);
            }
        }

        match ctx.key {
            None => (RangedTargetResult::NoResponse, None),
            Some(key) => match key {
                VirtualKeyCode::Escape => return (RangedTargetResult::Cancel, None),
                _ => (RangedTargetResult::NoResponse, None),
            },
        }
    }
    */
}
