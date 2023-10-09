use crate::uniques::GameLog;
use crate::{WINDOWHEIGHT, WINDOWWIDTH};
use engine::ai::decisions::Intent;
use engine::components::{CombatStats, Fire, Inventory, Name, Position, Vision};
use engine::map::Map;
use engine::palette::Palette;
use engine::player::get_player_map_knowledge;
use engine::uniques::{FrameTime, PPoint, PlayerID, Turn};
use engine::{GameSettings, OFFSET_X, OFFSET_Y, SCALE};
use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use shipyard::{Get, UniqueView, View, World};

pub mod camera;
pub use camera::*;

pub mod gui_menus;

/*
Render strategy:
Background color shows material
Glyph shows entity
glyph color is set by entity in general
Background color is modified by tile status such as gas, light, or fire
Glyph color is modified by some statuses?
 */

// https://dwarffortresswiki.org/index.php/Character_table

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn draw_gui(world: &World, ctx: &mut Rltk) {
    let world = &world;

    let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;
    let vstats = world.borrow::<View<CombatStats>>().unwrap();
    let map = world.borrow::<UniqueView<Map>>().unwrap();
    let turn = world.borrow::<UniqueView<Turn>>().unwrap();

    let hp_gui = if let Ok(player_stats) = vstats.get(player_id) {
        format!("{} / {} HP", player_stats.hp, player_stats.max_hp)
    } else {
        format!("")
    };

    // horizontal line
    ctx.print_color(
        0,
        OFFSET_Y - 1,
        Palette::MAIN_FG,
        Palette::MAIN_BG,
        "─".repeat(WINDOWWIDTH),
    );

    // player stats
    ctx.print_color(1, 1, Palette::MAIN_FG, Palette::MAIN_BG, hp_gui);
    ctx.print_color(1, 2, Palette::MAIN_FG, Palette::MAIN_BG, &format!("Turn: {:?}", *turn));
    ctx.print_color(
        1,
        9,
        Palette::MAIN_FG,
        Palette::MAIN_BG,
        format!("Depth: {}", map.depth),
    );

    // On fire display
    let vfire = world.borrow::<View<Fire>>().unwrap();
    match vfire.get(player_id) {
        Ok(_) => {
            ctx.print_color(1, 3, Palette::MAIN_FG, Palette::COLOR_FIRE, "FIRE");
        }
        Err(_) => {}
    }

    for y in 0..WINDOWHEIGHT {
        ctx.print_color(OFFSET_X - 1, y, Palette::MAIN_FG, Palette::MAIN_BG, "│");
    }

    // message log
    let log = world.borrow::<UniqueView<GameLog>>().unwrap();
    let mut y = 1;
    for m in log.messages.iter().rev() {
        if y < 9 {
            ctx.print_color(OFFSET_X + 1, y, Palette::MAIN_FG, Palette::MAIN_BG, m);
        }
        y += 1;
    }

    draw_tooltips(world, ctx);

    ctx.set_active_console(1);

    // draw mouse pos
    let mouse_pos = ctx.mouse_pos();
    if mouse_pos != (0, 0) {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_3);
    }

    ctx.set_active_console(0);
}

pub fn draw_tooltips(world: &World, ctx: &mut Rltk) {
    let world = &world;
    let player_pos = world.borrow::<UniqueView<PPoint>>().unwrap().0;
    let frametime = world.borrow::<UniqueView<FrameTime>>().unwrap().0;
    let map = world.borrow::<UniqueView<Map>>().unwrap();
    let settings = world.borrow::<UniqueView<GameSettings>>().unwrap();

    let (min_x, _max_x, min_y, _max_y) = get_map_coords_for_screen(player_pos, ctx, (map.width, map.height));

    let mouse_pos = ctx.mouse_pos();
    let mut map_mouse_pos = map.transform_mouse_pos(mouse_pos);
    map_mouse_pos.0 += min_x;
    map_mouse_pos.1 += min_y;
    if map_mouse_pos.0 >= map.width || map_mouse_pos.1 >= map.height || map_mouse_pos.0 < 0 || map_mouse_pos.1 < 0 {
        return;
    }

    let idx = map.xy_idx(map_mouse_pos.0, map_mouse_pos.1);
    if settings.use_player_los && !get_player_map_knowledge(world).contains_key(&idx) {
        return;
    }

    let vname = world.borrow::<View<Name>>().unwrap();
    let vpos = world.borrow::<View<Position>>().unwrap();
    let vstats = world.borrow::<View<CombatStats>>().unwrap();
    let vinv = world.borrow::<View<Inventory>>().unwrap();
    let vintent = world.borrow::<View<Intent>>().unwrap();

    let mut ypos = OFFSET_Y;

    /* Debug stuff */

    // ctx.print_color(2, ypos, Palette::MAIN_FG, Palette::MAIN_BG, format!("mouse: {:?}", map_mouse_pos));

    // ypos += 2;
    ctx.print_color(
        1,
        ypos,
        Palette::MAIN_FG,
        Palette::MAIN_BG,
        format!("PPOS: {:?}", player_pos),
    );

    ypos += 1;
    ctx.print_color(
        1,
        ypos,
        Palette::MAIN_FG,
        Palette::MAIN_BG,
        format!("Frametime: {:?}", frametime),
    );

    /* Normal stuff */
    ypos += 2;
    ctx.print_color(1, ypos, Palette::MAIN_FG, Palette::MAIN_BG, "Tile:");

    ypos += 1;
    ctx.print_color(
        2,
        ypos,
        Palette::MAIN_FG,
        Palette::MAIN_BG,
        format!("{:?}", map.tiles[idx]),
    );

    ypos += 2;
    ctx.print_color(1, ypos, Palette::MAIN_FG, Palette::MAIN_BG, "Entities:");

    for e in map.tile_content[idx].iter() {
        if let Ok(name) = vname.get(*e) {
            ypos += 1;
            ctx.print_color(
                2,
                ypos,
                Palette::MAIN_FG,
                Palette::MAIN_BG,
                format!("{:?} {}", e, name.name),
            );
        }

        if let Ok(pos) = vpos.get(*e) {
            ypos += 1;
            ctx.print_color(2, ypos, Palette::MAIN_FG, Palette::MAIN_BG, format!("{:?}", pos.ps[0]));
        }

        if let Ok(stats) = vstats.get(*e) {
            ypos += 1;
            ctx.print_color(
                2,
                ypos,
                Palette::MAIN_FG,
                Palette::MAIN_BG,
                format!("HP: {}/{}", stats.hp, stats.max_hp),
            );
        }

        if let Ok(intent) = vintent.get(*e) {
            ypos += 1;
            ctx.print_color(
                2,
                ypos,
                Palette::MAIN_FG,
                Palette::MAIN_BG,
                format!("Intent: {}", intent.name),
            );

            if intent.target.len() > 0 {
                ypos += 1;
                ctx.print_color(
                    3,
                    ypos,
                    Palette::MAIN_FG,
                    Palette::MAIN_BG,
                    format!("Target: {:?}", intent.target[0].get_point(&vpos)),
                );
            }
        }

        if let Ok(inv) = vinv.get(*e) {
            if inv.items.len() > 0 {
                ypos += 1;
                ctx.print_color(2, ypos, Palette::MAIN_FG, Palette::MAIN_BG, format!("Inventory:"));

                for item in inv.items.iter() {
                    if let Ok(name) = vname.get(*item) {
                        ypos += 1;
                        ctx.print_color(
                            3,
                            ypos,
                            Palette::MAIN_FG,
                            Palette::MAIN_BG,
                            format!("{:?}, {}", item, name.name),
                        );
                    }
                }
            }
        }

        ypos += 1;
    }

    // let mut tooltip: Vec<String> = Vec::new();

    // for e in map.tile_content[idx].iter() {
    //     if let Ok(name) = world.get::<Name>(*e) {
    //         tooltip.push(name.name.to_string());
    //     }
    // }

    // // for (_id, (name, pos)) in world.query::<(&Name, &Position)>().iter() {
    // //     for pos in pos.ps.iter() {
    // //         if pos.x == map_mouse_pos.0 && pos.y == map_mouse_pos.1 {
    // //             tooltip.push(name.name.to_string());
    // //         }
    // //     }
    // // }

    // if !tooltip.is_empty() {
    //     let mut width: i32 = 0;
    //     for s in tooltip.iter() {
    //         if width < s.len() as i32 { width = s.len() as i32; }
    //     }
    //     width += 3;

    //     let mut sign = 1;
    //     let mut arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
    //     let mut left_x = mouse_pos.0 + 4;
    //     let mut y = mouse_pos.1;
    //     if mouse_pos.0 > map.width / 2 {
    //         sign = -1;
    //         arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
    //         left_x = mouse_pos.0 - width;
    //     }

    //     if sign == -1 {ctx.fill_region(rltk::Rect{x1: left_x, x2: left_x - 3 + width, y1: y, y2: y + tooltip.len() as i32 - 1}, rltk::to_cp437(' '), Palette::MAIN_FG, Palette::COLOR_3);}
    //     else {ctx.fill_region(rltk::Rect{x1: left_x - 1, x2: left_x + width - 4, y1: y, y2: y + tooltip.len() as i32 - 1}, rltk::to_cp437(' '), Palette::MAIN_FG, Palette::COLOR_3);}

    //     for s in tooltip.iter() {
    //         ctx.print_color(left_x, y, Palette::MAIN_FG, Palette::COLOR_3, s);
    //         y += 1;
    //     }
    //     ctx.print_color(arrow_pos.x, arrow_pos.y, Palette::MAIN_FG, Palette::COLOR_3, "->");
    // }
}

pub fn get_map_coords_for_screen(focus: Point, ctx: &mut Rltk, mapsize: (i32, i32)) -> (i32, i32, i32, i32) {
    let (mut x_chars, mut y_chars) = ctx.get_char_size();
    x_chars -= (OFFSET_X as f32 / SCALE).ceil() as u32;
    y_chars -= (OFFSET_Y as f32 / SCALE).ceil() as u32;

    let center_x = (x_chars as f32 / 2.0) as i32;
    let center_y = (y_chars as f32 / 2.0) as i32;

    let mut min_x = focus.x - center_x;
    let mut max_x = focus.x + center_x;
    let mut min_y = focus.y - center_y;
    let mut max_y = focus.y + center_y;

    let w = mapsize.0;
    let h = mapsize.1;

    // Now check for borders, don't scroll past map edge
    if min_x < 0 {
        max_x -= min_x;
        min_x = 0;
    } else if max_x > w {
        min_x -= max_x - w;
        max_x = w - 1;
    }

    if min_y < 0 {
        max_y += 0 - min_y;
        min_y = 0;
    } else if max_y > h {
        min_y -= max_y - h;
        max_y = h - 1;
    }

    (min_x, max_x, min_y, max_y)
}

pub fn ranged_target(world: &World, ctx: &mut Rltk, range: i32) -> (ItemMenuResult, Option<Point>) {
    let map = world.borrow::<UniqueView<Map>>().unwrap();
    let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;
    let player_pos = world.borrow::<UniqueView<PPoint>>().unwrap().0;
    ctx.print_color(5, 12, Palette::COLOR_PURPLE, Palette::MAIN_BG, "Select a target");

    let (min_x, max_x, min_y, max_y) = get_map_coords_for_screen(player_pos, ctx, (map.width, map.height));

    let mut valid_cells: Vec<Point> = Vec::new();
    let vvs = world.borrow::<View<Vision>>().unwrap();
    match vvs.get(player_id) {
        Err(_e) => return (ItemMenuResult::Cancel, None),
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
                ItemMenuResult::Selected,
                Some(Point::new(map_mouse_pos.0, map_mouse_pos.1)),
            );
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_RED);
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => return (ItemMenuResult::Cancel, None),
            _ => (ItemMenuResult::NoResponse, None),
        },
    }
}
