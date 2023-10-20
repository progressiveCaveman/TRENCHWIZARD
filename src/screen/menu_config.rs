use strum_macros::{EnumCount as EnumCountMacro, EnumIter, FromRepr};

#[derive(PartialEq, Copy, Clone, Debug, EnumCountMacro, EnumIter, FromRepr)]
pub enum MainMenuSelection {
    Play = 0,
    ModeSelect,
    Quit
}

// There must be a better way to implement len and from
impl MainMenuSelection {
    pub fn modify(&self, dir: i32) -> Self {
        return if dir > 0 {
            self.inc()
        } else {
            self.dec()
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

#[derive(PartialEq, Copy, Clone, Debug, EnumCountMacro, EnumIter, FromRepr)]
pub enum ModeSelectSelection {
    MapDemo,
    RL,
    VillageSim,
}

impl ModeSelectSelection {
    pub fn modify(&self, dir: i32) -> Self {
        return if dir > 0 {
            self.inc()
        } else {
            self.dec()
        }
    }
    
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