#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
#[repr(usize)]
pub enum MainMenuSelection {
    Play = 0,
    ModeSelect,
    Quit
}

impl MainMenuSelection {
    pub fn len() -> usize {
        MainMenuSelection::Quit as usize
    }

    pub fn from(u: usize) -> MainMenuSelection {
        match u {
            0 => MainMenuSelection::Play,
            1 => MainMenuSelection::ModeSelect,
            2 => MainMenuSelection::Quit,
            _ => unreachable!(),
        }
    }

    pub fn inc(&self) -> Self {
        match *self {
            MainMenuSelection::Play => MainMenuSelection::ModeSelect,
            MainMenuSelection::ModeSelect => MainMenuSelection::Quit,
            MainMenuSelection::Quit => MainMenuSelection::Play,
        }
    }

    pub fn dec(&self) -> Self {
        match *self {
            MainMenuSelection::Play => MainMenuSelection::Quit,
            MainMenuSelection::ModeSelect => MainMenuSelection::Play,
            MainMenuSelection::Quit => MainMenuSelection::ModeSelect,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            MainMenuSelection::Play => "Play Game",
            MainMenuSelection::ModeSelect => "Mode Select",
            MainMenuSelection::Quit => "Quit",
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ModeSelectSelection {
    MapDemo,
    RL,
    VillageSim,
}

impl ModeSelectSelection {
    pub fn inc(&self) -> Self {
        match *self {
            ModeSelectSelection::MapDemo => ModeSelectSelection::RL,
            ModeSelectSelection::RL => ModeSelectSelection::VillageSim,
            ModeSelectSelection::VillageSim => ModeSelectSelection::MapDemo,
        }
    }

    pub fn dec(&self) -> Self {
        match *self {
            ModeSelectSelection::MapDemo => ModeSelectSelection::VillageSim,
            ModeSelectSelection::RL => ModeSelectSelection::MapDemo,
            ModeSelectSelection::VillageSim => ModeSelectSelection::RL,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            ModeSelectSelection::MapDemo => "Map Demo",
            ModeSelectSelection::RL => "RL",
            ModeSelectSelection::VillageSim => "Village Simulator",
        }
    }
}

// fn main() {
//     use ModeSelectSelection::*;

//     for i in &[North, South, East, West] {
//         println!("{:?} -> {:?}", i, i.turn());
//     }
// }