use engine::{config::GameMode, world::{Game, GameState}};

#[test]
fn run_village_sim() -> Result<(), Box<dyn std::error::Error>> {
    let mut game = Game::new(GameMode::VillageSim);
    game.set_state(GameState::PreTurn);
    game.autorun = true;

    for _ in 0..1000 {
        game.update();
    }

    Ok(())
}