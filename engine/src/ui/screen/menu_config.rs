use strum::EnumCount;
use strum_macros::{FromRepr, EnumIter, EnumCount as EnumCountMacro};
use strum_macros::Display;

#[derive(PartialEq, Copy, Clone, Debug, EnumCountMacro, FromRepr, Display, EnumIter)]
pub enum MainMenuSelection {
    #[strum(serialize = "Play Game")]
    Play = 0,

    #[strum(serialize = "Mode Select")]
    ModeSelect,

    #[strum(serialize = "Quit")]
    Quit
}

impl MainMenuSelection {
    pub fn modify(&self, dir: i32) -> Self {
        let i = *self as usize;

        if dir == 0 {
            return *self;
        }else if dir > 0 {
            if i + 1 == Self::COUNT {
                return Self::from_repr(0).unwrap();
            } else {
                return Self::from_repr(i + 1).unwrap();
            }
        } else {
            if i == 0 {
                return Self::from_repr(Self::COUNT - 1).unwrap();
            } else {
                return Self::from_repr(i - 1).unwrap();
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug, EnumCountMacro, FromRepr, Display)]
pub enum ModeSelectSelection {
    #[strum(serialize = "Map Demo")]
    MapDemo,

    #[strum(serialize = "Roguelike")]
    RL,

    #[strum(serialize = "Village Sim")]
    VillageSim,

    #[strum(serialize = "ORC ARENA")]
    OrcArena,
}

impl ModeSelectSelection {
    pub fn modify(&self, dir: i32) -> Self {
        let i = *self as usize;

        if dir == 0 {
            return *self;
        }else if dir > 0 {
            if i + 1 == Self::COUNT {
                return Self::from_repr(0).unwrap();
            } else {
                return Self::from_repr(i + 1).unwrap();
            }
        } else {
            if i == 0 {
                return Self::from_repr(Self::COUNT - 1).unwrap();
            } else {
                return Self::from_repr(i - 1).unwrap();
            }
        }
    }
}