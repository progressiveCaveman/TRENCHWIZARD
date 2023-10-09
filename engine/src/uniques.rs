use rltk::Point;
use shipyard::{EntityId, Unique};

#[derive(Debug, Unique)]
pub struct GameLog {
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Unique, Copy)]
pub struct PlayerID(pub EntityId);

#[derive(Clone, Debug, Unique, Copy)]
pub struct Turn(pub i32);

#[derive(Clone, Unique)]
pub struct RNG(pub rltk::RandomNumberGenerator);

#[derive(Clone, Debug, Unique, Copy)]
pub struct PPoint(pub Point);

#[derive(Clone, Debug, Unique, Copy)]
pub struct FrameTime(pub f32);
