use image::GenericImageView;
use image::{self};

use self::sprites::Sprite;

pub mod cp437_converter;
pub mod sprites;

const GLYPHS_PER_ROW: usize = 16;

pub struct Assets {
    pub cp437: Vec<(usize, Vec<Sprite>)>,
}

impl Assets {
    pub fn new() -> Assets {
        let mut sheets: Vec<(usize, Vec<Sprite>)> = vec![];
        sheets.push((8, Self::read_tilesheet("res/RDE_8x8.png", 8)));
        sheets.push((9, Self::read_tilesheet("res/Curses_9x9.png", 9)));
        sheets.push((10, Self::read_tilesheet("res/Paul_10x10.png", 10)));
        sheets.push((12, Self::read_tilesheet("res/Alloy_curses_12x12.png", 12)));
        sheets.push((16, Self::read_tilesheet("res/LCD_16x16.png", 16)));

        Assets { cp437: sheets }
    }

    pub fn read_tilesheet(path: &str, size: usize) -> Vec<Sprite> {
        let img = image::open(path).expect("File not found!");

        let empty_glyph = Sprite {
            width: size,
            height: size,
            pixels: vec![0; size * size * 4],
        };

        let mut cp: Vec<Sprite> = vec![empty_glyph; 256];

        for pixel in img.pixels() {
            let x: usize = pixel.0 as usize;
            let y: usize = pixel.1 as usize;

            let glyph_num = x / size + (GLYPHS_PER_ROW * (y / size));
            let xlocal = x % size;
            let ylocal = y % size;
            let idxlocal = (xlocal + ylocal * size) * 4;

            for i in 0..4 {
                cp[glyph_num].pixels[idxlocal + i] = pixel.2[i];
            }
        }

        return cp;
    }

    // pub fn glyph(&self, glyph: Glyph) -> Sprite {
    //     dbg!("WARNING: very slow");
    //     self.cp437[glyph.ch as usize].with_color(glyph.bg, glyph.fg)
    // }
}
