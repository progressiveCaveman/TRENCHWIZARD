/*

Each screen will have a
state
size
position
menustate index

any screen can have any number of screens embedded?
main screen has some named components?
Simplify render to use map and only render component

Have a screen mode and an active screen
Screen mode defines game mode behavior like controls

Representing the screen flow
One file is a state machine for the screen flow
Implementation is broken off into different file Eventually

Representing menu
Message
[Options]
how to pass control flow?

Screen always has an active menu
No mouse interaction to start


Initial use cases:
Main menu
inventory screen
item label


Targeting is a special function of a screen?


console types:
Main menu
Any submenus
local map
world map
log
stats
inventory
ais
overlays
label

*/

use engine::{map::Map, colors::{self}, components::Renderable};
use shipyard::{UniqueView, View, Get};

use crate::{Game, WIDTH, assets::cp437_converter::to_cp437, GameState};

use super::{Glyph, UI_GLYPH_SIZE, DEBUG_OUTLINES, menu_config::MainMenuSelection};

#[derive(Debug, PartialEq)]
pub enum ConsoleMode {
    MainMenu,
    WorldMap,
    Log,
}

#[derive(Debug)]
pub struct Console {
    pub size: (usize, usize),
    pub pos: (usize, usize),
    pub children: Vec<Console>,
    pub hidden: bool,
    pub z: i32, // not used yet
    pub mode: ConsoleMode,
    pub zoom: usize, // Only used for map mode
    pub map_pos: (usize, usize), // Only used for map mode
}

impl Console {
    pub fn new(size: (usize, usize), pos: (usize, usize), mode: ConsoleMode) -> Console {
        Self {
            size: size,
            pos: pos,
            children: vec![],
            hidden: false,
            z: 1,
            mode: mode,
            zoom: 1,
            map_pos: (0, 0),
        }
    }

    pub fn render(&self, frame: &mut [u8], game: &Game) {
        match self.mode {
            ConsoleMode::MainMenu => {
                self.render_main_menu(frame, game);
            }
            ConsoleMode::WorldMap => {
                self.render_map(frame, game);
            }
            ConsoleMode::Log => {
                self.render_log(frame, game);
            }
        }

        if DEBUG_OUTLINES {
            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                let xscreen = i % WIDTH;
                let yscreen = i / WIDTH;

                if self.in_bounds((xscreen, yscreen)) &&
                    (xscreen == self.pos.0 || 
                    xscreen == self.pos.0 + self.size.0 ||
                    yscreen == self.pos.1 ||
                    yscreen == self.pos.1 + self.size.1 )
                {
                    pixel.copy_from_slice(&colors::COLOR_PURPLE);               
                }
            }
        }
    }

    pub fn render_main_menu(&self, frame: &mut [u8], game: &Game) {
        let screen = &game.screen;

        // only render if gamestate is mainmenu
        if let GameState::MainMenu{selection} = game.state {
            screen.draw_box(
                &game.assets,
                frame,
                self.pos,
                self.size,
                colors::COLOR_UI_1,
                colors::COLOR_BLACK_SEMI_TRANS, // todo transparancy doesn't work
                UI_GLYPH_SIZE
            );

            let x = self.pos.0 + 3 * UI_GLYPH_SIZE;
            let mut y = self.pos.1 + 2 * UI_GLYPH_SIZE;

            screen.print_string(
                &game.assets,
                frame,
                "Main Menu",
                (x, y),
                colors::COLOR_UI_2,
                UI_GLYPH_SIZE
            );

            y += 2 * UI_GLYPH_SIZE;

            for i in 0..=MainMenuSelection::len() {
                let opt = MainMenuSelection::from(i);
                screen.print_string(
                    &game.assets,
                    frame,
                    opt.text(),
                    (x, y),
                    if selection == opt { colors::COLOR_UI_3 } else { colors::COLOR_UI_2 },
                    UI_GLYPH_SIZE
                );
    
                y += UI_GLYPH_SIZE;
            }
        }
    }

    pub fn render_map(&self, frame: &mut [u8], game: &Game) {
        let map = game.engine.world.borrow::<UniqueView<Map>>().unwrap();
        let vrend = &mut game.engine.world.borrow::<View<Renderable>>().unwrap();
        let screen = &game.screen;

        let tiles = if game.history_step >= map.history.len() || game.state != GameState::ShowMapHistory {
            map.tiles.clone()
        } else {
            map.history[game.history_step].clone()
        };

        if self.zoom < 8 {
            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                let xscreen = i % WIDTH;
                let yscreen = i / WIDTH;

                let xrange = self.pos.0..self.pos.0 + self.size.0;
                let yrange = self.pos.1..self.pos.1 + self.size.1;

                if xrange.contains(&xscreen) && yrange.contains(&yscreen) {
                    let xmap = self.map_pos.0 + (xscreen - self.pos.0) / self.zoom;
                    let ymap = self.map_pos.1 + (yscreen - self.pos.1) / self.zoom;

                    // calculate whether we're on a border for glyph fg render
                    let xmod = self.map_pos.0 + (xscreen - self.pos.0) % self.zoom;
                    let ymod = self.map_pos.1 + (yscreen - self.pos.1) % self.zoom;
                    let border = xmod < self.zoom / 4 || xmod >= self.zoom * 3 / 4 || ymod < self.zoom / 4 || ymod >= self.zoom * 3 / 4;

                    if map.in_bounds((xmap, ymap)) { 
                        let idx = map.xy_idx((xmap, ymap));
                        let mut render = tiles[idx].renderable();
                        for c in map.tile_content[idx].iter() {
                            if let Ok(rend) = vrend.get(*c) {
                                render = (rend.glyph, rend.fg, rend.bg);
                            }
                        }
                        let color = if border { render.2 } else { render.1 };
                        pixel.copy_from_slice(&color);
                    }
                }
            }
        } else {
            let widthchars = self.size.0 / self.zoom;
            let heightchars = self.size.1 / self.zoom;

            for x in 0 .. widthchars {
                for y in 0 .. heightchars {
                    let pos = (x + self.map_pos.0, y + self.map_pos.1);
                    // let idx = map.point_idx(point);
                    if x < self.pos.0 + self.size.0 + self.zoom && y < self.pos.1 + self.size.1 + self.zoom && map.in_bounds(pos){
                        let idx = map.xy_idx(pos);
                        let mut render = tiles[idx].renderable();
                        for c in map.tile_content[idx].iter() {
                            if let Ok(rend) = vrend.get(*c) {
                                render = (rend.glyph, rend.fg, rend.bg);
                            }
                        }                        
                        screen.print_cp437(
                            &game.assets,
                            frame,
                            Glyph {
                                pos: (self.pos.0 + x * self.zoom, self.pos.1 + y * self.zoom),
                                ch: to_cp437(render.0),
                                fg: render.1,
                                bg: render.2,
                            },
                            self.zoom
                        );
                    }
                }
            }
        }
    }

    pub fn render_log(&self, frame: &mut [u8], game: &Game) {
        let screen = &game.screen;

        screen.draw_box(
            &game.assets,
            frame,
            (self.pos.0, self.pos.1),
            (self.size.0, self.size.1),
            colors::COLOR_UI_1,
            colors::COLOR_CLEAR,
            UI_GLYPH_SIZE
        );
        
        let mut y = 1;
        for m in game.engine.get_log().messages.iter().rev() {
            for ms in m.chars().collect::<Vec<_>>().chunks(self.size.0 / UI_GLYPH_SIZE - 2) {
                if y * UI_GLYPH_SIZE < self.size.1 - UI_GLYPH_SIZE {
                    let s: String = ms.into_iter().collect();
                    screen.print_string(
                        &game.assets,
                        frame,
                        &s,
                        (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                        colors::COLOR_UI_2,
                        UI_GLYPH_SIZE
                    );
                    y += 1;
                } else {
                    return; // todo this will be a bug if more is added to this function
                }
            }
        }
    }

    pub fn in_bounds(&self, pos: (usize, usize)) -> bool {
        return pos.0 >= self.pos.0 && 
            pos.0 <= self.pos.0 + self.size.0 && 
            pos.1 >= self.pos.1 &&
            pos.1 <= self.pos.1 + self.size.1
    }
}
