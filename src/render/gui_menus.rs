use crate::RunState;
use engine::components::{Equippable, Equipped, InBackpack, Inventory, Name, Player};
use engine::palette::Palette;
use engine::uniques::PlayerID;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rltk::{Rltk, VirtualKeyCode};
use shipyard::{EntityId, Get, IntoIter, IntoWithId, UniqueView, View, World};
use std::convert::TryFrom;

use super::ItemMenuResult;

pub enum ItemActionSelection {
    Cancel,
    NoSelection,
    Used,
    Dropped,
    Unequipped,
}

#[derive(PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, Debug)]
#[repr(i8)]
pub enum MainMenuSelection {
    Roguelike,
    Simulator,
    OrcHalls,
    Exit,
}

pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selection { selected: MainMenuSelection },
}

pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn main_menu(ctx: &mut Rltk, runstate: RunState) -> MainMenuResult {
    let get_fg = |sel, menu_item| {
        if sel == menu_item {
            return Palette::COLOR_RED;
        } else {
            return Palette::MAIN_FG;
        }
    };

    ctx.print_color_centered(15, Palette::COLOR_GREEN_DARK, Palette::MAIN_BG, "Select a mode");

    if let RunState::MainMenu {
        menu_selection: selection,
    } = runstate
    {
        ctx.print_color_centered(
            25,
            get_fg(selection, MainMenuSelection::Roguelike),
            Palette::MAIN_BG,
            "Roguelike",
        );
        ctx.print_color_centered(
            30,
            get_fg(selection, MainMenuSelection::Simulator),
            Palette::MAIN_BG,
            "Simulator",
        );
        ctx.print_color_centered(
            35,
            get_fg(selection, MainMenuSelection::OrcHalls),
            Palette::MAIN_BG,
            "Orc Halls",
        );
        ctx.print_color_centered(40, get_fg(selection, MainMenuSelection::Exit), Palette::MAIN_BG, "Exit");

        match ctx.key {
            None => return MainMenuResult::NoSelection { selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => {
                        return MainMenuResult::Selection {
                            selected: MainMenuSelection::Exit,
                        }
                    }
                    VirtualKeyCode::Up => {
                        let sel: i8 = selection.into();
                        // TODO: use len of menu selections instead of hard coded 3
                        let new_sel = MainMenuSelection::try_from((sel - 1i8).rem_euclid(4)).unwrap();
                        return MainMenuResult::NoSelection { selected: new_sel };
                    }
                    VirtualKeyCode::Down => {
                        let sel: i8 = selection.into();
                        // TODO: use len of menu selections instead of hard coded 3
                        let new_sel = MainMenuSelection::try_from((sel + 1i8).rem_euclid(4)).unwrap();
                        return MainMenuResult::NoSelection { selected: new_sel };
                    }
                    VirtualKeyCode::Return => return MainMenuResult::Selection { selected: selection },
                    _ => return MainMenuResult::NoSelection { selected: selection },
                }
            }
        }
    }

    MainMenuResult::NoSelection {
        selected: MainMenuSelection::Roguelike,
    }
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(19, Palette::MAIN_FG, Palette::MAIN_BG, "You are dead.");
    ctx.print_color_centered(
        23,
        Palette::MAIN_FG,
        Palette::MAIN_BG,
        "Press any key to return to the main menu.",
    );
    match ctx.key {
        None => return GameOverResult::NoSelection,
        Some(_key) => return GameOverResult::QuitToMenu,
    }
}

pub fn show_inventory(world: &World, ctx: &mut Rltk) -> (ItemMenuResult, Option<EntityId>) {
    let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;

    // Items in backpack
    // let mut query = //world.query::<(&InBackpack, &Name)>();
    let vinv = world.borrow::<View<Inventory>>().unwrap();
    let inventory = vinv.get(player_id).unwrap(); //world.get::<Inventory>(player_id).unwrap();//query.iter().filter(|item| item.1.0.owner == *player_id);
    let backpack_count = inventory.items.len();
    let mut y = 25 - (backpack_count / 2);
    ctx.draw_box(10, y - 2, 31, backpack_count + 3, Palette::MAIN_FG, Palette::MAIN_BG);

    let title = "Inventory";
    ctx.print_color(13, y - 2, Palette::MAIN_FG, Palette::MAIN_BG, title);

    let useable: Vec<EntityId> = world.run(|vpack: View<InBackpack>, vname: View<Name>| {
        let mut useable: Vec<EntityId> = Vec::new();
        for (j, (id, (_pack, name))) in (&vpack, &vname)
            .iter()
            .with_id()
            .filter(|item| item.1 .0.owner == player_id)
            .enumerate()
        {
            ctx.set(12, y, Palette::MAIN_FG, Palette::MAIN_BG, rltk::to_cp437('('));
            ctx.set(
                13,
                y,
                Palette::COLOR_PURPLE,
                Palette::MAIN_BG,
                97 + j as rltk::FontCharType,
            );
            ctx.set(14, y, Palette::MAIN_FG, Palette::MAIN_BG, rltk::to_cp437(')'));

            ctx.print_color(16, y, Palette::MAIN_FG, Palette::MAIN_BG, &name.name.to_string());
            useable.push(id);
            y += 1;
        }
        useable
    });

    // Items equipped
    // let mut query = world.query::<(&Equipped, &Name)>();
    let equipped_count = world.run(|vplayer: View<Player>, vequipped: View<Equipped>| {
        let mut count = 0;
        for (_, _) in (&vplayer, &vequipped).iter() {
            count += 1;
        }
        count
    }); //query.iter().filter(|item| item.1.0.owner == *player_id);
        // let equipped_count = equipped_items.count();

    let mut y = 25 - (equipped_count / 2);
    ctx.draw_box(45, y - 2, 31, equipped_count + 3, Palette::MAIN_FG, Palette::MAIN_BG);

    let title = "Equipment";
    ctx.print_color(48, y - 2, Palette::MAIN_FG, Palette::MAIN_BG, title);

    let equipped: Vec<EntityId> = world.run(|vewquipped: View<Equipped>, vname: View<Name>| {
        let mut equipped: Vec<EntityId> = Vec::new();
        for (j, (id, (_pack, name))) in (&vewquipped, &vname)
            .iter()
            .with_id()
            .filter(|item| item.1 .0.owner == player_id)
            .enumerate()
        {
            let offset = j + backpack_count;
            ctx.set(47, y, Palette::MAIN_FG, Palette::MAIN_BG, rltk::to_cp437('('));
            ctx.set(
                48,
                y,
                Palette::COLOR_PURPLE,
                Palette::MAIN_BG,
                97 + offset as rltk::FontCharType,
            );
            ctx.set(49, y, Palette::MAIN_FG, Palette::MAIN_BG, rltk::to_cp437(')'));

            ctx.print_color(51, y, Palette::MAIN_FG, Palette::MAIN_BG, &name.name.to_string());
            equipped.push(id);
            y += 1;
        }
        equipped
    });

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < backpack_count as i32 {
                    return (ItemMenuResult::Selected, Some(useable[selection as usize]));
                } else if selection >= backpack_count as i32 && selection < (backpack_count + equipped_count) as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equipped[selection as usize - backpack_count]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn show_item_actions(world: &World, item: EntityId, ctx: &mut Rltk) -> ItemActionSelection {
    let vpack = world.borrow::<View<InBackpack>>().unwrap();
    let vequippable = world.borrow::<View<Equippable>>().unwrap();
    let vequipped = world.borrow::<View<Equipped>>().unwrap();
    let vname = world.borrow::<View<Name>>().unwrap();

    let item_name = vname.get(item).unwrap();
    ctx.draw_box(15, 23, 31, 5, Palette::MAIN_FG, Palette::MAIN_BG);
    ctx.print_color(18, 23, Palette::MAIN_FG, Palette::MAIN_BG, item_name.name.to_string());

    let mut in_backpack = false;
    let mut in_equip = false;

    if let Ok(_in_backpack) = vpack.get(item) {
        in_backpack = true;
        if let Ok(_equippable) = vequippable.get(item) {
            ctx.print_color(17, 25, Palette::MAIN_FG, Palette::MAIN_BG, "(a) Equip");
        } else {
            ctx.print_color(17, 25, Palette::MAIN_FG, Palette::MAIN_BG, "(a) Use");
        }
        ctx.print_color(18, 25, Palette::COLOR_PURPLE, Palette::MAIN_BG, "a");
        ctx.print_color(17, 26, Palette::MAIN_FG, Palette::MAIN_BG, "(b) Drop");
        ctx.print_color(18, 26, Palette::COLOR_PURPLE, Palette::MAIN_BG, "b");
    } else if let Ok(_in_equip) = vequipped.get(item) {
        in_equip = true;
        ctx.print_color(17, 25, Palette::MAIN_FG, Palette::MAIN_BG, "(a) Unequip");
        ctx.print_color(18, 25, Palette::COLOR_PURPLE, Palette::MAIN_BG, "a");
        ctx.print_color(17, 26, Palette::MAIN_FG, Palette::MAIN_BG, "(b) Drop");
        ctx.print_color(18, 26, Palette::COLOR_PURPLE, Palette::MAIN_BG, "b");
    } else {
        panic!("Item is not in backpack or equipped? Where is it?");
    }

    match ctx.key {
        None => ItemActionSelection::NoSelection,
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => ItemActionSelection::Cancel,
                VirtualKeyCode::A => {
                    //TODO: Add unequip action and select here based on if item has Equipped
                    //component?
                    if in_backpack {
                        return ItemActionSelection::Used;
                    }
                    if in_equip {
                        return ItemActionSelection::Unequipped;
                    }
                    ItemActionSelection::Used
                }
                VirtualKeyCode::B => ItemActionSelection::Dropped,
                _ => ItemActionSelection::NoSelection,
            }
        }
    }
}
