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

use engine::{map::{Map, XY}, colors::{self}, components::{Renderable, CombatStats}, utils::InvalidPoint};
use shipyard::{UniqueView, View, Get};
use strum::EnumCount;

use crate::{Game, WIDTH, assets::cp437_converter::to_cp437, GameState};

use super::{Glyph, UI_GLYPH_SIZE, DEBUG_OUTLINES, menu_config::{MainMenuSelection, ModeSelectSelection}, MAX_ZOOM};

#[derive(Debug, PartialEq)]
pub enum ConsoleMode {
    MainMenu,
    WorldMap,
    Log,
    Info,
}

#[derive(Debug)]
pub struct Console {
    pub size: XY,
    pub pos: XY,
    pub children: Vec<Console>,
    pub hidden: bool,
    pub z: i32, // not used yet
    pub mode: ConsoleMode,
    pub zoom: i32, // TODO tile_size would be more approp name
    pub map_pos: XY, // Only used for map mode
}

impl Console {
    pub fn new(size: XY, pos: XY, mode: ConsoleMode) -> Console {
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
                self.render_mode_select(frame, game);
            }
            ConsoleMode::WorldMap => {
                self.render_map(frame, game);
            }
            ConsoleMode::Log => {
                self.render_log(frame, game);
            }
            ConsoleMode::Info => {
                self.render_info(frame, game);
            },
        }

        if DEBUG_OUTLINES {
            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                let xscreen = i as i32 % WIDTH;
                let yscreen = i as i32 / WIDTH;

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
        if let GameState::MainMenu{selection} = game.state {
            let screen = &game.screen;

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

            for i in 0..=MainMenuSelection::COUNT {
                if let Some(opt) = MainMenuSelection::from_repr(i) {
                    screen.print_string(
                        &game.assets,
                        frame,
                        opt.text(),
                        (x, y),
                        if selection as usize == i { colors::COLOR_UI_3 } else { colors::COLOR_UI_2 },
                        UI_GLYPH_SIZE
                    );
        
                    y += UI_GLYPH_SIZE;
                }
            }
        }
    }

    pub fn render_mode_select(&self, frame: &mut [u8], game: &Game) {
        if let GameState::ModeSelect{selection} = game.state {
            let screen = &game.screen;

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
                "Select Game Mode",
                (x, y),
                colors::COLOR_UI_2,
                UI_GLYPH_SIZE
            );

            y += 2 * UI_GLYPH_SIZE;

            for i in 0..=ModeSelectSelection::COUNT {
                if let Some(opt) = ModeSelectSelection::from_repr(i) {
                    screen.print_string(
                        &game.assets,
                        frame,
                        opt.text(),
                        (x, y),
                        if selection as usize == i { colors::COLOR_UI_3 } else { colors::COLOR_UI_2 },
                        UI_GLYPH_SIZE
                    );
        
                    y += UI_GLYPH_SIZE;
                }
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

        let map_pos = if game.engine.settings.follow_player {
            let ppos = game.engine.get_player_pos().0.to_xy();

            let mp = (ppos.0 - self.size.0 / (2 * self.zoom), ppos.1 - self.size.1 / (2 * self.zoom));
            (i32::max(0, mp.0), i32::max(0, mp.1))
        } else {
            self.map_pos
        };

        if self.zoom < 8 {
            let xrange = self.pos.0..self.pos.0 + self.size.0;
            let yrange = self.pos.1..self.pos.1 + self.size.1;

            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                let xscreen = i as i32 % WIDTH;
                let yscreen = i as i32 / WIDTH;

                if xrange.contains(&xscreen) && yrange.contains(&yscreen) {
                    let xmap = map_pos.0 + (xscreen - self.pos.0) / self.zoom;
                    let ymap = map_pos.1 + (yscreen - self.pos.1) / self.zoom;

                    if map.in_bounds((xmap, ymap)) { 
                        let idx = map.xy_idx((xmap, ymap));
                        let mut render = tiles[idx].renderable();
                        for c in map.tile_content[idx].iter() {
                            if let Ok(rend) = vrend.get(*c) {
                                render = (rend.glyph, rend.fg, rend.bg);
                            }
                        }

                        // calculate whether we're on a border for glyph fg render
                        let xmod = map_pos.0 + (xscreen - self.pos.0) % self.zoom;
                        let ymod = map_pos.1 + (yscreen - self.pos.1) % self.zoom;
                        let border = xmod < self.zoom / 4 || xmod >= self.zoom * 3 / 4 || ymod < self.zoom / 4 || ymod >= self.zoom * 3 / 4;

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
                    let pos = (x + map_pos.0, y + map_pos.1);
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
            for ms in m.chars().collect::<Vec<_>>().chunks((self.size.0 / UI_GLYPH_SIZE) as usize - 2) {
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

    pub fn render_info(&self, frame: &mut [u8], game: &Game) {
        let screen = &game.screen;
        let player_id = game.engine.get_player_id().0;

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
        screen.print_string(
            &game.assets,
            frame,
            "calendar",
            (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
            colors::COLOR_UI_2,
            UI_GLYPH_SIZE
        );

        y += 1;
        if let Ok(vstats) = game.engine.world.borrow::<View<CombatStats>>() {
            if let Ok(stat) = vstats.get(player_id) {
                screen.print_string(
                    &game.assets,
                    frame,
                    &format!("HP: {}/{}", stat.hp, stat.max_hp),
                    (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                    colors::COLOR_UI_2,
                    UI_GLYPH_SIZE
                );
            }
        }
    }

    pub fn in_bounds(&self, pos: XY) -> bool {
        return pos.0 >= self.pos.0 && 
            pos.0 <= self.pos.0 + self.size.0 && 
            pos.1 >= self.pos.1 &&
            pos.1 <= self.pos.1 + self.size.1
    }

    pub fn zoom_to_fit(&mut self, map: &Map) {
        while self.zoom < MAX_ZOOM && (self.zoom + 1) * map.size.0 < self.size.0 && (self.zoom + 1) * map.size.1 < self.size.1 {
            self.zoom += 1;
        }
    }

    pub fn zoom_in(&mut self) {
        if self.zoom < MAX_ZOOM {
            self.zoom += 1;
        }
    }

    pub fn zoom_out(&mut self) {
        if self.zoom > 1 {
            self.zoom -= 1;
        }
    }
}
