use engine::{map::{Map, XY}, colors::{self}, components::{Renderable, CombatStats, PPoint, FrameTime, Name, Position, Inventory, Equippable, Consumable}, player::get_player_map_knowledge, ai::decisions::Intent};
use shipyard::{UniqueView, View, Get};
use strum::EnumCount;

use crate::{WIDTH, assets::cp437_converter::to_cp437, game::{Game, GameState}};

use super::{Glyph, UI_GLYPH_SIZE, DEBUG_OUTLINES, menu_config::{MainMenuSelection, ModeSelectSelection}, MAX_ZOOM};

#[derive(Debug, PartialEq)]
pub enum ConsoleMode {
    MainMenu,
    WorldMap,
    Log,
    Info,
    Context,
    Inventory,
    ItemInfo,
}

#[derive(Debug)]
pub struct Console {
    pub size: XY,
    pub pos: XY,
    pub children: Vec<Console>,
    pub hidden: bool,
    pub z: i32, // not used yet
    pub mode: ConsoleMode,
    pub gsize: i32, 
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
            gsize: 16,
            map_pos: (0, 0),
        }
    }

    pub fn in_bounds(&self, pos: XY) -> bool {
        return pos.0 >= self.pos.0 && 
            pos.0 <= self.pos.0 + self.size.0 && 
            pos.1 >= self.pos.1 &&
            pos.1 <= self.pos.1 + self.size.1
    }

    pub fn zoom_to_fit(&mut self, map: &Map) {
        while self.gsize < MAX_ZOOM && (self.gsize + 1) * map.size.0 < self.size.0 && (self.gsize + 1) * map.size.1 < self.size.1 {
            self.gsize += 1;
        }
    }

    pub fn zoom_in(&mut self) {
        if self.gsize < MAX_ZOOM {
            self.gsize += 1;
        }
    }

    pub fn zoom_out(&mut self) {
        if self.gsize > 1 {
            self.gsize -= 1;
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
            ConsoleMode::Context => {
                self.render_context(frame, game);
            },
            ConsoleMode::Inventory => {
                self.render_inventory(frame, game);
            },
            ConsoleMode::ItemInfo => {
                self.render_item_info(frame, game);
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

        if self.gsize < 8 {
            let xrange = self.pos.0..self.pos.0 + self.size.0;
            let yrange = self.pos.1..self.pos.1 + self.size.1;

            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                let xscreen = i as i32 % WIDTH;
                let yscreen = i as i32 / WIDTH;

                if xrange.contains(&xscreen) && yrange.contains(&yscreen) {
                    let xmap = self.map_pos.0 + (xscreen - self.pos.0) / self.gsize;
                    let ymap = self.map_pos.1 + (yscreen - self.pos.1) / self.gsize;

                    if map.in_bounds((xmap, ymap)) { 
                        let idx = map.xy_idx((xmap, ymap));
                        let mut render = tiles[idx].renderable();
                        for c in map.tile_content[idx].iter() {
                            if let Ok(rend) = vrend.get(*c) {
                                render = (rend.glyph, rend.fg, rend.bg);
                            }
                        }

                        // calculate whether we're on a border for glyph fg render
                        let xmod = self.map_pos.0 + (xscreen - self.pos.0) % self.gsize;
                        let ymod = self.map_pos.1 + (yscreen - self.pos.1) % self.gsize;
                        let border = xmod < self.gsize / 4 || xmod >= self.gsize * 3 / 4 || 
                            ymod < self.gsize / 4 || ymod >= self.gsize * 3 / 4;

                        let color = if border { render.2 } else { render.1 };
                        pixel.copy_from_slice(&color);
                    }
                }
            }
        } else {
            let widthchars = self.size.0 / self.gsize;
            let heightchars = self.size.1 / self.gsize;

            for x in 0 .. widthchars {
                for y in 0 .. heightchars {
                    let pos = (x + self.map_pos.0, y + self.map_pos.1);
                    // let idx = map.point_idx(point);
                    if x < self.pos.0 + self.size.0 + self.gsize && y < self.pos.1 + self.size.1 + self.gsize && map.in_bounds(pos){
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
                                pos: (self.pos.0 + x * self.gsize, self.pos.1 + y * self.gsize),
                                ch: to_cp437(render.0),
                                fg: render.1,
                                bg: render.2,
                            },
                            self.gsize
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
            self.pos,
            self.size,
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

    pub fn render_context(&self, frame: &mut [u8], game: &Game) {
        let screen = &game.screen;
        let world = &game.engine.world;
        let map = world.borrow::<UniqueView<Map>>().unwrap();
        let settings = game.engine.settings;

        screen.draw_box(
            &game.assets,
            frame,
            self.pos,
            self.size,
            colors::COLOR_UI_1,
            colors::COLOR_CLEAR,
            UI_GLYPH_SIZE
        );

        let mpos = screen.get_mouse_game_pos();
        if !map.in_bounds(mpos) {
            return;
        }

        let mut y = 1;
        let idx = map.xy_idx(mpos);
        if settings.use_player_los && !get_player_map_knowledge(world).contains_key(&idx) {
            return;
        }
    
        let player_pos = world.borrow::<UniqueView<PPoint>>().unwrap().0;
        let frametime = world.borrow::<UniqueView<FrameTime>>().unwrap().0;
        let vname = world.borrow::<View<Name>>().unwrap();
        let vpos = world.borrow::<View<Position>>().unwrap();
        let vstats = world.borrow::<View<CombatStats>>().unwrap();
        let vinv = world.borrow::<View<Inventory>>().unwrap();
        let vintent = world.borrow::<View<Intent>>().unwrap();
        
        /* Debug stuff */
        screen.print_string(
            &game.assets,
            frame,
            &format!("PPOS: {:?}", player_pos),
            (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
            colors::COLOR_UI_2,
            UI_GLYPH_SIZE
        );
    
        y += 1;
        screen.print_string(
            &game.assets,
            frame,
            &format!("Frametime: {:?}", frametime),
            (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
            colors::COLOR_UI_2,
            UI_GLYPH_SIZE
        );
    
        /* Normal stuff */
        y += 2;
        screen.print_string(
            &game.assets,
            frame,
            &format!("Tile: {:?}", map.tiles[idx]),
            (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
            colors::COLOR_UI_2,
            UI_GLYPH_SIZE
        );
    
        y += 2;
        screen.print_string(
            &game.assets,
            frame,
            &format!("Entities:"),
            (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
            colors::COLOR_UI_2,
            UI_GLYPH_SIZE
        );
    
        for e in map.tile_content[idx].iter() {
            if let Ok(name) = vname.get(*e) {
                y += 1;
                screen.print_string(
                    &game.assets,
                    frame,
                    &format!(" {:?} {}", e, name.name),
                    (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                    colors::COLOR_UI_2,
                    UI_GLYPH_SIZE
                );
            }
    
            if let Ok(pos) = vpos.get(*e) {
                y += 1;
                screen.print_string(
                    &game.assets,
                    frame,
                    &format!(" {:?}", pos.ps[0]),
                    (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                    colors::COLOR_UI_2,
                    UI_GLYPH_SIZE
                );
            }
    
            if let Ok(stats) = vstats.get(*e) {
                y += 1;
                screen.print_string(
                    &game.assets,
                    frame,
                    &format!(" HP: {}/{}", stats.hp, stats.max_hp),
                    (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                    colors::COLOR_UI_2,
                    UI_GLYPH_SIZE
                );
            }
    
            if let Ok(intent) = vintent.get(*e) {
                y += 1;
                screen.print_string(
                    &game.assets,
                    frame,
                    &format!(" Intent: {}", intent.name),
                    (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                    colors::COLOR_UI_2,
                    UI_GLYPH_SIZE
                );
    
                if intent.target.len() > 0 {
                    y += 1;
                    screen.print_string(
                        &game.assets,
                        frame,
                        &format!(" Target: {:?}", intent.target[0].get_point(&vpos)),
                        (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                        colors::COLOR_UI_2,
                        UI_GLYPH_SIZE
                    );
                }
            }
    
            if let Ok(inv) = vinv.get(*e) {
                if inv.items.len() > 0 {
                    y += 1;
                    screen.print_string(
                        &game.assets,
                        frame,
                        &format!(" Inventory:"),
                        (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                        colors::COLOR_UI_2,
                        UI_GLYPH_SIZE
                    );
    
                    for item in inv.items.iter() {
                        if let Ok(name) = vname.get(*item) {
                            y += 1;
                            if y > self.size.1 * self.gsize {
                                return;
                            }
                            screen.print_string(
                                &game.assets,
                                frame,
                                &format!("  {:?}, {}", item, name.name),
                                (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                                colors::COLOR_UI_2,
                                UI_GLYPH_SIZE
                            );
                        }
                    }
                }
            }
    
            y += 1;
        }
    
    }

    pub fn render_inventory(&self, frame: &mut [u8], game: &Game) {
        let screen = &game.screen;
        let player_id = game.engine.get_player_id().0;
        let vinv = game.engine.world.borrow::<View<Inventory>>().unwrap();
        let vname = game.engine.world.borrow::<View<Name>>().unwrap();

        if let GameState::ShowInventory { selection } = game.state { 
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
                "Inventory", // insert a verb here?
                (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                colors::COLOR_UI_2,
                UI_GLYPH_SIZE,
            );
    
            y += 1;
            let mut invnum = 0;
            if let Ok(inv) = vinv.get(player_id) {
                for item in inv.items.iter() {
                    if let Ok(name) = vname.get(*item) {
                        y += 1;
                        screen.print_string(
                            &game.assets,
                            frame,
                            &format!("- {}", name.name),
                            (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                            if selection == invnum { colors::COLOR_UI_3 } else { colors::COLOR_UI_2 },
                            UI_GLYPH_SIZE
                        );
                        invnum += 1;
                    }
                }
            }
        }
    }

    pub fn render_item_info(&self, frame: &mut [u8], game: &Game) {
        let screen = &game.screen;
        let vname = game.engine.world.borrow::<View<Name>>().unwrap();
        let vequip = game.engine.world.borrow::<View<Equippable>>().unwrap();
        let vconsumable = game.engine.world.borrow::<View<Consumable>>().unwrap();

        if let GameState::ShowItemActions { item } = game.state {
            if let Ok(name) = vname.get(item) {
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
                    &format!("{}", name.name),
                    (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                    colors::COLOR_UI_2,
                    UI_GLYPH_SIZE
                );
        
                y += 2;
                if let Ok(_) = vconsumable.get(item) {
                    screen.print_string(
                        &game.assets,
                        frame,
                        &format!("(a) - Apply"), 
                        (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                        colors::COLOR_UI_2,
                        UI_GLYPH_SIZE
                    );
                    y += 1;
                }

                if let Ok(_) = vequip.get(item) {
                    screen.print_string(
                        &game.assets,
                        frame,
                        &format!("(e) - Equip"), 
                        (self.pos.0 + UI_GLYPH_SIZE, self.pos.1 + y * UI_GLYPH_SIZE),
                        colors::COLOR_UI_2,
                        UI_GLYPH_SIZE
                    );
                    y += 1;
                }
            }
        }
    }
}
