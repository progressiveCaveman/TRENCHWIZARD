use action::Action;
use intent::Intent;
use shipyard::{AllStorages, EntityId};

// pub mod decisions;
pub mod action;
pub mod intent;
pub mod consideration;
pub mod input;
pub mod labors;

pub struct AI {}

impl AI {
    pub fn choose_intent(actions: Vec<Action>, store: &AllStorages, id: EntityId) -> Intent {
        if actions.len() < 1 {
            panic!("No actions to choose from");
        }

        let mut best = (0.0, Intent::idle());

        for i in 0..actions.len() {
            let action = &actions[i];
            let score = action.evaluate(store, id);

            // println!("Action: {}, score: {}", action.name, score);

            if score.0 > best.0 {
                best = score;
            }
        }

        best.1.clone()
    }
}