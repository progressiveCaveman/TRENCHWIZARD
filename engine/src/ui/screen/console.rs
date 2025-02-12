use std::iter::zip;

use crate::{ai::intent::Intent, player::get_player_map_knowledge, ui::colors::{self, Color}, utils::InvalidPoint, world::{components::{Consumable, Equipment, Equippable, FrameTime, Inventory, Name, OnFire, PPoint, PhysicalStats, PlayerID, Position, Turn, Vision}, map::{Map, XY}, Game, GameState}};
use rltk::Point;
use shipyard::{UniqueView, View, Get, World, IntoIter, IntoWithId};
use strum::EnumCount;

use crate::{WIDTH, ui::assets::{cp437_converter::{to_cp437, string_to_cp437}, Assets, sprites::Drawable}, HEIGHT, ui::screen::RangedTargetResult};

use super::{Glyph, menu_config::{MainMenuSelection, ModeSelectSelection}, MAX_ZOOM};

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

// todo add a way to dim a console, for overlay purposes
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
    pub fn new(size: XY, pos: XY, mode: ConsoleMode, gsize: i32) -> Console {
        Self {
            size: size,
            pos: pos,
            children: vec![],
            hidden: false,
            z: 1,
            mode: mode,
            gsize,
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
    }

    pub fn render_main_menu(&self, frame: &mut [u8], game: &Game) {
        if let GameState::MainMenu{selection} = game.state {
            self.draw_box(
                &game.assets,
                frame,
                self.pos,
                self.size,
                colors::COLOR_UI_1,
                colors::COLOR_BG,
                self.gsize,
                "".to_string()
            );

            let x = self.pos.0 + 3 * self.gsize;
            let mut y = self.pos.1 + 2 * self.gsize;

            self.print_string(
                &game.assets,
                frame,
                "Main Menu",
                (x, y),
                colors::COLOR_UI_2,
                self.gsize
            );

            y += 2 * self.gsize;

            for i in 0..=MainMenuSelection::COUNT {
                if let Some(opt) = MainMenuSelection::from_repr(i) {
                    self.print_string(
                        &game.assets,
                        frame,
                        &opt.to_string(),
                        (x, y),
                        if selection as usize == i { colors::COLOR_UI_3 } else { colors::COLOR_UI_2 },
                        self.gsize
                    );
        
                    y += self.gsize;
                }
            }
        }
    }

    pub fn render_mode_select(&self, frame: &mut [u8], game: &Game) {
        if let GameState::ModeSelect{selection} = game.state {
            self.draw_box(
                &game.assets,
                frame,
                self.pos,
                self.size,
                colors::COLOR_UI_1,
                colors::COLOR_BG, // todo transparancy doesn't work
                self.gsize,
                "".to_string()
            );

            let x = self.pos.0 + 3 * self.gsize;
            let mut y = self.pos.1 + 2 * self.gsize;

            self.print_string(
                &game.assets,
                frame,
                "Select Game Mode",
                (x, y),
                colors::COLOR_UI_2,
                self.gsize
            );

            y += 2 * self.gsize;

            for i in 0..=ModeSelectSelection::COUNT {
                if let Some(opt) = ModeSelectSelection::from_repr(i) {
                    self.print_string(
                        &game.assets,
                        frame,
                        &opt.to_string(),
                        (x, y),
                        if selection as usize == i { colors::COLOR_UI_3 } else { colors::COLOR_UI_2 },
                        self.gsize
                    );
        
                    y += self.gsize;
                }
            }
        }
    }

    pub fn render_map(&self, frame: &mut [u8], game: &Game) {
        let map = game.world_sim.world.borrow::<UniqueView<Map>>().unwrap();
        let hidx = if game.state == GameState::ShowMapHistory {
            Some(game.history_step)
        } else {
            None
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
                        // let idx = map.xy_idx((xmap, ymap));
                        // let mut render = tiles[idx].renderable();
                        // for c in map.tile_content[idx].iter() {
                        //     if let Ok(rend) = vrend.get(*c) {
                        //         render = (rend.glyph, rend.fg, rend.bg);
                        //     }
                        // }
                        let render = map.get_renderable((xmap, ymap), &game.world_sim.settings, &game.world_sim.world, hidx);

                        // calculate whether we're on a border for glyph fg render
                        let xmod = (xscreen - self.pos.0) % self.gsize;
                        let ymod = (yscreen - self.pos.1) % self.gsize;
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
                    if x < self.pos.0 + self.size.0 + self.gsize && y < self.pos.1 + self.size.1 + self.gsize && map.in_bounds(pos){
                        let render = map.get_renderable(pos, &game.world_sim.settings, &game.world_sim.world, hidx);
                        self.print_cp437(
                            &game.assets,
                            frame,
                            Glyph {
                                pos: (self.pos.0 + x * self.gsize, self.pos.1 + y * self.gsize),
                                ch: to_cp437(render.0),
                                fg: render.1,
                                bg: render.2,
                                gsize: self.gsize,
                            },
                        );
                    }
                }
            }
        }
    }

    pub fn render_log(&self, frame: &mut [u8], game: &Game) {
        self.draw_box(
            &game.assets,
            frame,
            self.pos,
            self.size,
            colors::COLOR_UI_1,
            colors::COLOR_BG,
            self.gsize,
            "Log".to_string()
        );
        
        let mut y = 1;
        for m in game.world_sim.get_log().messages.iter().rev() {
            for ms in m.chars().collect::<Vec<_>>().chunks((self.size.0 / self.gsize) as usize - 2) {
                if y * self.gsize < self.size.1 - self.gsize {
                    let s: String = ms.into_iter().collect();
                    self.print_string(
                        &game.assets,
                        frame,
                        &s,
                        (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                        colors::COLOR_UI_2,
                        self.gsize
                    );
                    y += 1;
                } else {
                    return; // todo this will be a bug if more is added to this function
                }
            }
        }
    }

    pub fn render_info(&self, frame: &mut [u8], game: &Game) {
        let player_id = game.world_sim.get_player_id().0;

        self.draw_box(
            &game.assets,
            frame,
            (self.pos.0, self.pos.1),
            (self.size.0, self.size.1),
            colors::COLOR_UI_1,
            colors::COLOR_BG,
            self.gsize,
            "Stats".to_string()
        );

        let mut y = 1;
        // self.print_string(
        //     &game.assets,
        //     frame,
        //     "calendar",
        //     (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
        //     colors::COLOR_UI_2,
        //     self.gsize
        // );
        if let Ok(turn) = game.world_sim.world.borrow::<UniqueView<Turn>>() {
            // if let Ok(fire) = vonfire.get(player_id) {
                self.print_string(
                    &game.assets,
                    frame,
                    &format!("Turn: {}", turn.0),
                    (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                    colors::COLOR_FIRE,
                    self.gsize
                );
                // y += 1;
            // }
        }

        y += 1;
        if let Ok(vstats) = game.world_sim.world.borrow::<View<PhysicalStats>>() {
            if let Ok(stat) = vstats.get(player_id) {
                self.print_string(
                    &game.assets,
                    frame,
                    &format!("HP: {}/{}", stat.hp, stat.max_hp),
                    (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                    colors::COLOR_UI_2,
                    self.gsize
                );
                y += 1;
            }
        }

        if let Ok(vonfire) = game.world_sim.world.borrow::<View<OnFire>>() {
            if let Ok(fire) = vonfire.get(player_id) {
                self.print_string(
                    &game.assets,
                    frame,
                    &format!("FIRE {}", fire.turns),
                    (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                    colors::COLOR_FIRE,
                    self.gsize
                );
                // y += 1;
            }
        }
    }

    pub fn render_context(&self, frame: &mut [u8], game: &Game) {
        let screen = &game.screen;
        let world = &game.world_sim.world;
        let map = world.borrow::<UniqueView<Map>>().unwrap();
        let settings = game.world_sim.settings;

        self.draw_box(
            &game.assets,
            frame,
            self.pos,
            self.size,
            colors::COLOR_UI_1,
            colors::COLOR_BG,
            self.gsize,
            "debug info".to_string()
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
        let vstats = world.borrow::<View<PhysicalStats>>().unwrap();
        let vinv = world.borrow::<View<Inventory>>().unwrap();
        let vintent = world.borrow::<View<Intent>>().unwrap();
        let vonfire = world.borrow::<View<OnFire>>().unwrap();
        
        /* Debug stuff */
        self.print_string(
            &game.assets,
            frame,
            &format!("PPOS: {:?}", player_pos),
            (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
            colors::COLOR_UI_2,
            self.gsize
        );
    
        y += 1;
        self.print_string(
            &game.assets,
            frame,
            &format!("Frametime: {:?}", frametime),
            (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
            colors::COLOR_UI_2,
            self.gsize
        );
    
        /* Normal stuff */
        y += 2;
        self.print_string(
            &game.assets,
            frame,
            &format!("mouse pos: {:?}", mpos),
            (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
            colors::COLOR_UI_2,
            self.gsize
        );

        y += 1;
        self.print_string(
            &game.assets,
            frame,
            &format!("Tile: {:?}", map.tiles[idx]),
            (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
            colors::COLOR_UI_2,
            self.gsize
        );

        y += 1;
        self.print_string(
            &game.assets,
            frame,
            &format!("Gases:"),
            (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
            colors::COLOR_UI_2,
            self.gsize
        );
    
        for e in map.gases[idx].0.iter() {
            y += 1;
            if (y + 1) * self.gsize >= self.size.1 {
                return;
            }
            self.print_string(
                &game.assets,
                frame,
                &format!(" {:?}", e),
                (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                colors::COLOR_UI_2,
                self.gsize
            );
        }
    
        y += 2;
        self.print_string(
            &game.assets,
            frame,
            &format!("Entities:"),
            (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
            colors::COLOR_UI_2,
            self.gsize
        );
    
        for e in map.tile_content[idx].iter() {
            if let Ok(name) = vname.get(*e) {
                y += 1;
                self.print_string(
                    &game.assets,
                    frame,
                    &format!(" {:?} {}", e, name.name),
                    (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                    colors::COLOR_UI_2,
                    self.gsize
                );
            }
    
            if let Ok(pos) = vpos.get(*e) {
                y += 1;
                self.print_string(
                    &game.assets,
                    frame,
                    &format!(" {:?}", pos.ps[0]),
                    (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                    colors::COLOR_UI_2,
                    self.gsize
                );
            }
    
            if let Ok(stats) = vstats.get(*e) {
                y += 1;
                self.print_string(
                    &game.assets,
                    frame,
                    &format!(" HP: {}/{}", stats.hp, stats.max_hp),
                    (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                    colors::COLOR_UI_2,
                    self.gsize
                );
            }

            if let Ok(fire) = vonfire.get(*e) {
                y += 1;
                self.print_string(
                    &game.assets,
                    frame,
                    &format!("FIRE {}", fire.turns),
                    (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                    colors::COLOR_FIRE,
                    self.gsize
                );
            }
    
            if let Ok(intent) = vintent.get(*e) {
                y += 1;
                self.print_string(
                    &game.assets,
                    frame,
                    &format!(" Intent: {}", intent.name),
                    (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                    colors::COLOR_UI_2,
                    self.gsize
                );
    
                if intent.target.len() > 0 {
                    y += 1;
                    self.print_string(
                        &game.assets,
                        frame,
                        &format!(" Target: {:?}", intent.target[0].get_point(&vpos)),
                        (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                        colors::COLOR_UI_2,
                        self.gsize
                    );
                }
            }
    
            if let Ok(inv) = vinv.get(*e) {
                if inv.items.len() > 0 {
                    y += 1;
                    self.print_string(
                        &game.assets,
                        frame,
                        &format!(" Inventory:"),
                        (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                        colors::COLOR_UI_2,
                        self.gsize
                    );
    
                    for item in inv.items.iter() {
                        if let Ok(name) = vname.get(*item) {
                            y += 1;
                            if (y + 1) * self.gsize >= self.size.1 {
                                return;
                            }
                            self.print_string(
                                &game.assets,
                                frame,
                                &format!("  {}", name.name),
                                (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                                colors::COLOR_UI_2,
                                self.gsize
                            );
                        }
                    }
                }
            }
    
            y += 1;
        }
    
    }

    pub fn render_inventory(&self, frame: &mut [u8], game: &Game) {
        let player_id = game.world_sim.get_player_id().0;
        let vinv = game.world_sim.world.borrow::<View<Inventory>>().unwrap();
        let vequipment = game.world_sim.world.borrow::<View<Equipment>>().unwrap();
        let vname = game.world_sim.world.borrow::<View<Name>>().unwrap();

        if let GameState::ShowInventory { selection } = game.state {    
            let mut y = 1;
            for (_id, equipment) in (&vequipment).iter().with_id() {
                for (eslot, item) in equipment.items.iter() {
                    let mut name = "";
                    if let Some(item) = item {
                        if let Ok(iname) = vname.get(*item) {
                            name = &iname.name
                        } 
                    }

                    self.print_string(
                        &game.assets,
                        frame,
                        &format!("{:?} - {}", eslot, name),
                        (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                        colors::COLOR_UI_2,
                        self.gsize
                    );
                    y += 1;
                }
            }
            
            // draw box once we know how much we're printing
            y += 1;
            self.draw_box(
                &game.assets,
                frame,
                (self.pos.0, self.pos.1),
                (self.size.0, y * self.gsize),
                colors::COLOR_UI_1,
                colors::COLOR_BG,
                self.gsize,
                "Equipment".to_string()
            );
            let invstart = y;

            // y += 2;

            // y += 3;
            // self.print_string(
            //     &game.assets,
            //     frame,
            //     "Inventory",
            //     (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
            //     colors::COLOR_UI_2,
            //     self.gsize,
            // );
    
            // y += 1;
            let mut invnum = 0;
            if let Ok(inv) = vinv.get(player_id) {
                for item in inv.items.iter() {
                    if let Ok(name) = vname.get(*item) {
                        y += 1;
                        self.print_string(
                            &game.assets,
                            frame,
                            &format!("- {}", name.name),
                            (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                            if selection == invnum { colors::COLOR_UI_3 } else { colors::COLOR_UI_2 },
                            self.gsize
                        );
                        invnum += 1;
                    }
                }
            }

            y += 1;
            self.draw_box(
                &game.assets,
                frame,
                (self.pos.0, self.pos.1 + invstart * self.gsize),
                (self.size.0, (y - invstart + 1) * self.gsize),
                colors::COLOR_UI_1,
                colors::COLOR_BG,
                self.gsize,
                "Inventory".to_string()
            );
        }
    }

    pub fn render_item_info(&self, frame: &mut [u8], game: &Game) {
        let vname = game.world_sim.world.borrow::<View<Name>>().unwrap();
        let vequip = game.world_sim.world.borrow::<View<Equippable>>().unwrap();
        let vconsumable = game.world_sim.world.borrow::<View<Consumable>>().unwrap();

        if let GameState::ShowItemActions { item } = game.state {
            if let Ok(name) = vname.get(item) {
                self.draw_box(
                    &game.assets,
                    frame,
                    (self.pos.0, self.pos.1),
                    (self.size.0, self.size.1),
                    colors::COLOR_UI_1,
                    colors::COLOR_BG,
                    self.gsize,
                    "Item Info".to_string()
                );
        
                let mut y = 1;
                self.print_string(
                    &game.assets,
                    frame,
                    &format!("{}", name.name),
                    (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                    colors::COLOR_UI_2,
                    self.gsize
                );
        
                y += 2;
                if let Ok(_) = vconsumable.get(item) {
                    self.print_string(
                        &game.assets,
                        frame,
                        &format!("(a) - Apply"), 
                        (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                        colors::COLOR_UI_2,
                        self.gsize
                    );
                    y += 1;
                }

                if let Ok(_) = vequip.get(item) {
                    self.print_string(
                        &game.assets,
                        frame,
                        &format!("(e) - Equip"), 
                        (self.pos.0 + self.gsize, self.pos.1 + y * self.gsize),
                        colors::COLOR_UI_2,
                        self.gsize
                    );
                    // y += 1;
                }
            }
        }
    }


















    // render stuff

    /// Blit a drawable to the pixel buffer. 
    /// Assumes glyph asset has fuscia bg and grayscale fg
    pub fn blit_glyph(&self, frame: &mut [u8], assets: &Assets, dest: XY, glyph: Glyph) {
        let gsize = glyph.gsize;
        let mut spritesheet = &assets.cp437[0];
        for ss in assets.cp437.iter() {
            if gsize == ss.0 as i32 {
                spritesheet = ss;
            } else if gsize < ss.0 as i32 {
                break;
            }
        }

        let sprite = &spritesheet.1[glyph.ch as usize];

        if dest.0 + sprite.width() as i32 > WIDTH || dest.1 + sprite.height() as i32 > HEIGHT {
            // dbg!("ERROR: Print off screen");
            return;
        }

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
                        left[i2] = (left[i2] as f32 * (1.0 - glyph.bg[3] as f32 /255.0)) as u8 + glyph.bg[i2];
                    } else { // foreground
                        left[i2] = (left[i2] as f32 * (1.0 - glyph.fg[3] as f32 /255.0)) as u8 + (right[i2] as f32 * glyph.fg[i2] as f32 / 255 as f32) as u8;
                    }
                }
            }

            s += width;
        }
    }

    pub fn print_cp437(&self, assets: &Assets, frame: &mut [u8], glyph: Glyph) {
        // if glyph.pos.1 >= self.size.1 - gsize || glyph.pos.0 >= self.size.0 - gsize {
        //     return;
        // }

        self.blit_glyph(frame, assets, glyph.pos, glyph);
    }

    pub fn print_string(&self, assets: &Assets, frame: &mut [u8], str: &str, pos: XY, color: Color, gsize: i32) {
        let chars = string_to_cp437(str);

        for (idx, ch) in chars.iter().enumerate() {
            self.print_cp437(assets, frame, Glyph { 
                pos: (pos.0 + idx as i32 * gsize, pos.1),
                ch: *ch, 
                fg: color, 
                bg: colors::COLOR_BG,
                gsize, 
            });
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

    pub fn draw_box(&self, assets: &Assets, frame: &mut [u8], pos: XY, size: XY, fg: Color, bg: Color, gsize: i32, title: String) {
        let vertwall = 186;
        let horizwall = 205;
        let nwcorner = 201;
        let necorner = 187;
        let secorner = 188;
        let swcorner = 200;

        let title = string_to_cp437(title);

        for x in (pos.0 .. pos.0 + size.0).step_by(gsize as usize) {
            for y in (pos.1 .. pos.1 + size.1).step_by(gsize as usize) {
                let firstcolumn = x < pos.0 + gsize;
                let lastcolumn = x >= pos.0 + size.0 - gsize;
                let firstrow = y < pos.1 + gsize;
                let lastrow = y >= pos.1 + size.1 - gsize;

                let mut ch = if firstrow && firstcolumn {
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

                if y == pos.1 && x > pos.0  && x <= pos.0 + gsize * title.len() as i32 {
                    let idx = ((x - pos.0) / gsize - 1) as usize;
                    if idx < title.len() {
                        ch = title[idx];
                    }
                }

                self.print_cp437(assets, frame, Glyph { pos: (x, y), ch, fg, bg, gsize });
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

    pub fn highlight_map_coord(&mut self, frame: &mut [u8], assets: &Assets, map_pos: XY, mut color: Color) {
        // let xmap = self.map_offset.0 + (xscreen - self.pos.0) / self.gsize;
        
        // set alpha
        color[3] = 128;

        let pos = (
            self.pos.0 + (map_pos.0 - self.map_pos.0) * self.gsize, 
            self.pos.1 + (map_pos.1 - self.map_pos.1) * self.gsize, 
        );

        let glyph = Glyph {
            pos,
            ch: to_cp437(' '),
            fg: colors::COLOR_BG,
            bg: color,
            gsize: self.gsize,
        };

        self.print_cp437(assets, frame, glyph);
    }

    pub fn ranged_target(&mut self, frame: &mut [u8], assets: &Assets, world: &mut World, map_mouse_pos: XY, range: i32, clicked: bool, target: XY) -> (RangedTargetResult, Option<Point>) {
        let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;
        let player_pos = world.borrow::<UniqueView<PPoint>>().unwrap().0;
        let vvs = world.borrow::<View<Vision>>().unwrap();

        self.draw_box(assets, frame, self.pos, (20 * self.gsize, 3 * self.gsize), colors::COLOR_UI_2, colors::COLOR_BG, self.gsize, "".to_string());
        self.print_string(assets, frame, "Select a target", (self.pos.0 + self.gsize, self.pos.1 + self.gsize), colors::COLOR_UI_1, self.gsize);

        // calculate valid cells
        let mut valid_cells: Vec<Point> = Vec::new();
        match vvs.get(player_id) {
            Err(_e) => return (RangedTargetResult::Cancel, None),
            Ok(player_vs) => {
                for pt in player_vs.visible_tiles.iter() {
                    let dist = rltk::DistanceAlg::Pythagoras.distance2d(player_pos, *pt);
                    if dist as i32 <= range { // tile within range
                        let color = if pt.to_xy() == target {
                            colors::COLOR_HIGHLIGHT2
                        } else {
                            colors::COLOR_HIGHLIGHT1
                        };
                        valid_cells.push(*pt);
                        self.highlight_map_coord(frame, assets, pt.to_xy(), color);
                    }
                }
            }
        }

        // handle mouse
        let mut valid_target = false;
        for pt in valid_cells.iter() {
            if pt.x == map_mouse_pos.0 && pt.y == map_mouse_pos.1 {
                valid_target = true;
                break;
            }
        }
        if valid_target {
            self.highlight_map_coord(frame, assets, map_mouse_pos, colors::COLOR_HIGHLIGHT2);

            if clicked {
                return (RangedTargetResult::Selected, Some(Point::new(map_mouse_pos.0, map_mouse_pos.1)));
            } else {
                return (RangedTargetResult::NewTarget { target: map_mouse_pos }, None);
            }
        } else {
            if clicked {
                return (RangedTargetResult::Cancel, None);
            }
        }

        (RangedTargetResult::NoResponse, None)
    }
}
