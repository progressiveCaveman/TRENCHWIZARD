use engine::Engine;

use crate::{screen::Screen, assets::Assets, WIDTH, HEIGHT};


pub struct Game {
    pub engine: Engine,
    pub screen: Screen,
    pub assets: Assets,
    pub tick: i32,
    pub game_log: Vec<String>
}

impl Game {
    pub fn new() -> Self {
        Self {
            engine: Engine::new((WIDTH, HEIGHT)),
            screen: Screen::new((WIDTH, HEIGHT)),
            assets: Assets::new(),
            tick: 0,
            game_log: Vec::new(),
        }
    }

    /// Update the `World` internal state
    pub fn update(&mut self) {
        self.tick += 1;
        if self.tick % 100 == 0 {
            self.game_log.push(format!("Test {}", self.tick / 100));
            // let mut rng = RandomNumberGenerator::new();

            // let x = rng.roll_dice(1, self.map.size.0);
            // let y = rng.roll_dice(1, self.map.size.1);

            // self.screen.pos = (x, y);
        }
    }

    /// Draw the `World` state to the frame buffer.
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&self, frame: &mut [u8]) {
        // clear screen
        for (_, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba = [0x00, 0x00, 0x00, 0x00];
            pixel.copy_from_slice(&rgba);
        }

        self.screen.draw(frame, &self);
        // self.screen.draw_image(&self.image, frame, Point{ x: 0, y: 0 })
    }
}
