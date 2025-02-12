use rltk::RGBA;

// NOTE; colour names and values copied from:
// tcod-sys/libtcod/include/libtcod_int.h
pub type Color = [u8; 4];

pub trait ColorUtils {
    fn scale(&self, scale: f32) -> Self;
    fn to_rgba(&self) -> RGBA;
    fn add(&self, color: Color) -> Self;
}

impl ColorUtils for Color {
    fn scale(&self, scale: f32) -> Self {
        let mut c = self.clone();
        for i in 0..3 { // don't scale alpha
            c[i] = (c[i] as f32 * scale) as u8;
        }
        c
    }

    fn add(&self, color: Color) -> Self {
        let mut c: [u8; 4] = self.clone();
        for i in 0..3 { // don't scale alpha
            c[i] = u8::min(c[i] + color[i], 255);
        }
        c
    }

    fn to_rgba(&self) -> RGBA {
        RGBA::from_u8(self[0], self[1], self[2], self[3])
    }
}

pub const COLOR_UI_1: Color = COLOR_DARK_PURPLE;
pub const COLOR_UI_2: Color = COLOR_GREEN;
pub const COLOR_UI_3: Color = COLOR_CYAN;
pub const COLOR_UI_4: Color = COLOR_LIGHTER_PURPLE;
pub const COLOR_WATER: Color = COLOR_DARK_BLUE;
pub const COLOR_SAND: Color = COLOR_DESATURATED_YELLOW;
pub const COLOR_DIRT: Color = COLOR_DARKEST_GREEN;
pub const COLOR_STONE: Color = COLOR_GREY;
pub const COLOR_BG: Color = [0, 0, 0, 200];
pub const COLOR_HIGHLIGHT1: Color = COLOR_DARK_GREY;
pub const COLOR_HIGHLIGHT2: Color = COLOR_GREY;
pub const COLOR_ITEM: Color = COLOR_DARK_YELLOW;
pub const COLOR_WALL: Color = COLOR_GREY;
pub const COLOR_FLOOR: Color = COLOR_DARKEST_GREY;
pub const COLOR_GRASS: Color = COLOR_DARKER_GREEN;

pub const COLOR_FIRE: Color = COLOR_RED;

// : RGBA = RGBA {
//     r: 0.88,
//     g: 0.34,
//     b: 0.13,
//     a: 1.,
// };
pub const COLOR_CEDAR: Color = COLOR_BRASS;

// : RGBA = RGBA {
//     r: 0.39,
//     g: 0.22,
//     b: 0.17,
//     a: 1.,
// };

pub const FACTION_COLORS: [Color; 2] = [COLOR_CRIMSON, COLOR_AZURE];

// pub const FACTION_COLORS: [RGBA; 2] = [
//     RGBA {
//         r: 1.0,
//         g: 0.,
//         b: 0.,
//         a: 1.,
//     },
//     RGBA {
//         r: 0.0,
//         g: 0.0,
//         b: 1.0,
//         a: 1.,
//     },
// ];

//  Basic Colors
pub const COLOR_CLEAR: Color = [0, 0, 0, 0];
pub const COLOR_BLACK: Color = [0, 0, 0, 255];
pub const COLOR_DARKEST_GREY: Color = [31, 31, 31, 255];
pub const COLOR_DARKER_GREY: Color = [63, 63, 63, 255];
pub const COLOR_DARK_GREY: Color = [95, 95, 95, 255];
pub const COLOR_GREY: Color = [127, 127, 127, 255];
pub const COLOR_LIGHT_GREY: Color = [159, 159, 159, 255];
pub const COLOR_LIGHTER_GREY: Color = [191, 191, 191, 255];
pub const COLOR_LIGHTEST_GREY: Color = [223, 223, 223, 255];
pub const COLOR_WHITE: Color = [255, 255, 255, 255];
pub const COLOR_DARKEST_SEPIA: Color = [31, 24, 15, 255];
pub const COLOR_DARKER_SEPIA: Color = [63, 50, 31, 255];
pub const COLOR_DARK_SEPIA: Color = [94, 75, 47, 255];
pub const COLOR_SEPIA: Color = [127, 101, 63, 255];
pub const COLOR_LIGHT_SEPIA: Color = [158, 134, 100, 255];
pub const COLOR_LIGHTER_SEPIA: Color = [191, 171, 143, 255];
pub const COLOR_LIGHTEST_SEPIA: Color = [222, 211, 195, 255];
pub const COLOR_DESATURATED_RED: Color = [127, 63, 63, 255];
pub const COLOR_DESATURATED_FLAME: Color = [127, 79, 63, 255];
pub const COLOR_DESATURATED_ORANGE: Color = [127, 95, 63, 255];
pub const COLOR_DESATURATED_AMBER: Color = [127, 111, 63, 255];
pub const COLOR_DESATURATED_YELLOW: Color = [127, 127, 63, 255];
pub const COLOR_DESATURATED_LIME: Color = [111, 127, 63, 255];
pub const COLOR_DESATURATED_CHARTREUSE: Color = [95, 127, 63, 255];
pub const COLOR_DESATURATED_GREEN: Color = [63, 127, 63, 255];
pub const COLOR_DESATURATED_SEA: Color = [63, 127, 95, 255];
pub const COLOR_DESATURATED_TURQUOISE: Color = [63, 127, 111, 255];
pub const COLOR_DESATURATED_CYAN: Color = [63, 127, 127, 255];
pub const COLOR_DESATURATED_SKY: Color = [63, 111, 127, 255];
pub const COLOR_DESATURATED_AZURE: Color = [63, 95, 127, 255];
pub const COLOR_DESATURATED_BLUE: Color = [63, 63, 127, 255];
pub const COLOR_DESATURATED_HAN: Color = [79, 63, 127, 255];
pub const COLOR_DESATURATED_VIOLET: Color = [95, 63, 127, 255];
pub const COLOR_DESATURATED_PURPLE: Color = [111, 63, 127, 255];
pub const COLOR_DESATURATED_FUCHSIA: Color = [127, 63, 127, 255];
pub const COLOR_DESATURATED_MAGENTA: Color = [127, 63, 111, 255];
pub const COLOR_DESATURATED_PINK: Color = [127, 63, 95, 255];
pub const COLOR_DESATURATED_CRIMSON: Color = [127, 63, 79, 255];
pub const COLOR_LIGHTEST_RED: Color = [255, 191, 191, 255];
pub const COLOR_LIGHTEST_FLAME: Color = [255, 207, 191, 255];
pub const COLOR_LIGHTEST_ORANGE: Color = [255, 223, 191, 255];
pub const COLOR_LIGHTEST_AMBER: Color = [255, 239, 191, 255];
pub const COLOR_LIGHTEST_YELLOW: Color = [255, 255, 191, 255];
pub const COLOR_LIGHTEST_LIME: Color = [239, 255, 191, 255];
pub const COLOR_LIGHTEST_CHARTREUSE: Color = [223, 255, 191, 255];
pub const COLOR_LIGHTEST_GREEN: Color = [191, 255, 191, 255];
pub const COLOR_LIGHTEST_SEA: Color = [191, 255, 223, 255];
pub const COLOR_LIGHTEST_TURQUOISE: Color = [191, 255, 239, 255];
pub const COLOR_LIGHTEST_CYAN: Color = [191, 255, 255, 255];
pub const COLOR_LIGHTEST_SKY: Color = [191, 239, 255, 255];
pub const COLOR_LIGHTEST_AZURE: Color = [191, 223, 255, 255];
pub const COLOR_LIGHTEST_BLUE: Color = [191, 191, 255, 255];
pub const COLOR_LIGHTEST_HAN: Color = [207, 191, 255, 255];
pub const COLOR_LIGHTEST_VIOLET: Color = [223, 191, 255, 255];
pub const COLOR_LIGHTEST_PURPLE: Color = [239, 191, 255, 255];
pub const COLOR_LIGHTEST_FUCHSIA: Color = [255, 191, 255, 255];
pub const COLOR_LIGHTEST_MAGENTA: Color = [255, 191, 239, 255];
pub const COLOR_LIGHTEST_PINK: Color = [255, 191, 223, 255];
pub const COLOR_LIGHTEST_CRIMSON: Color = [255, 191, 207, 255];
pub const COLOR_LIGHTER_RED: Color = [255, 127, 127, 255];
pub const COLOR_LIGHTER_FLAME: Color = [255, 159, 127, 255];
pub const COLOR_LIGHTER_ORANGE: Color = [255, 191, 127, 255];
pub const COLOR_LIGHTER_AMBER: Color = [255, 223, 127, 255];
pub const COLOR_LIGHTER_YELLOW: Color = [255, 255, 127, 255];
pub const COLOR_LIGHTER_LIME: Color = [223, 255, 127, 255];
pub const COLOR_LIGHTER_CHARTREUSE: Color = [191, 255, 127, 255];
pub const COLOR_LIGHTER_GREEN: Color = [127, 255, 127, 255];
pub const COLOR_LIGHTER_SEA: Color = [127, 255, 191, 255];
pub const COLOR_LIGHTER_TURQUOISE: Color = [127, 255, 223, 255];
pub const COLOR_LIGHTER_CYAN: Color = [127, 255, 255, 255];
pub const COLOR_LIGHTER_SKY: Color = [127, 223, 255, 255];
pub const COLOR_LIGHTER_AZURE: Color = [127, 191, 255, 255];
pub const COLOR_LIGHTER_BLUE: Color = [127, 127, 255, 255];
pub const COLOR_LIGHTER_HAN: Color = [159, 127, 255, 255];
pub const COLOR_LIGHTER_VIOLET: Color = [191, 127, 255, 255];
pub const COLOR_LIGHTER_PURPLE: Color = [223, 127, 255, 255];
pub const COLOR_LIGHTER_FUCHSIA: Color = [255, 127, 255, 255];
pub const COLOR_LIGHTER_MAGENTA: Color = [255, 127, 223, 255];
pub const COLOR_LIGHTER_PINK: Color = [255, 127, 191, 255];
pub const COLOR_LIGHTER_CRIMSON: Color = [255, 127, 159, 255];
pub const COLOR_LIGHT_RED: Color = [255, 63, 63, 255];
pub const COLOR_LIGHT_FLAME: Color = [255, 111, 63, 255];
pub const COLOR_LIGHT_ORANGE: Color = [255, 159, 63, 255];
pub const COLOR_LIGHT_AMBER: Color = [255, 207, 63, 255];
pub const COLOR_LIGHT_YELLOW: Color = [255, 255, 63, 255];
pub const COLOR_LIGHT_LIME: Color = [207, 255, 63, 255];
pub const COLOR_LIGHT_CHARTREUSE: Color = [159, 255, 63, 255];
pub const COLOR_LIGHT_GREEN: Color = [63, 255, 63, 255];
pub const COLOR_LIGHT_SEA: Color = [63, 255, 159, 255];
pub const COLOR_LIGHT_TURQUOISE: Color = [63, 255, 207, 255];
pub const COLOR_LIGHT_CYAN: Color = [63, 255, 255, 255];
pub const COLOR_LIGHT_SKY: Color = [63, 207, 255, 255];
pub const COLOR_LIGHT_AZURE: Color = [63, 159, 255, 255];
pub const COLOR_LIGHT_BLUE: Color = [63, 63, 255, 255];
pub const COLOR_LIGHT_HAN: Color = [111, 63, 255, 255];
pub const COLOR_LIGHT_VIOLET: Color = [159, 63, 255, 255];
pub const COLOR_LIGHT_PURPLE: Color = [207, 63, 255, 255];
pub const COLOR_LIGHT_FUCHSIA: Color = [255, 63, 255, 255];
pub const COLOR_LIGHT_MAGENTA: Color = [255, 63, 207, 255];
pub const COLOR_LIGHT_PINK: Color = [255, 63, 159, 255];
pub const COLOR_LIGHT_CRIMSON: Color = [255, 63, 111, 255];
pub const COLOR_RED: Color = [255, 0, 0, 255];
pub const COLOR_FLAME: Color = [255, 63, 0, 255];
pub const COLOR_ORANGE: Color = [255, 127, 0, 255];
pub const COLOR_AMBER: Color = [255, 191, 0, 255];
pub const COLOR_YELLOW: Color = [255, 255, 0, 255];
pub const COLOR_LIME: Color = [191, 255, 0, 255];
pub const COLOR_CHARTREUSE: Color = [127, 255, 0, 255];
pub const COLOR_GREEN: Color = [0, 255, 0, 255];
pub const COLOR_SEA: Color = [0, 255, 127, 255];
pub const COLOR_TURQUOISE: Color = [0, 255, 191, 255];
pub const COLOR_CYAN: Color = [0, 255, 255, 255];
pub const COLOR_SKY: Color = [0, 191, 255, 255];
pub const COLOR_AZURE: Color = [0, 127, 255, 255];
pub const COLOR_BLUE: Color = [0, 0, 255, 255];
pub const COLOR_HAN: Color = [63, 0, 255, 255];
pub const COLOR_VIOLET: Color = [127, 0, 255, 255];
pub const COLOR_PURPLE: Color = [191, 0, 255, 255];
pub const COLOR_FUCHSIA: Color = [255, 0, 255, 255];
pub const COLOR_MAGENTA: Color = [255, 0, 191, 255];
pub const COLOR_PINK: Color = [255, 0, 127, 255];
pub const COLOR_CRIMSON: Color = [255, 0, 63, 255];
pub const COLOR_DARK_RED: Color = [191, 0, 0, 255];
pub const COLOR_DARK_FLAME: Color = [191, 47, 0, 255];
pub const COLOR_DARK_ORANGE: Color = [191, 95, 0, 255];
pub const COLOR_DARK_AMBER: Color = [191, 143, 0, 255];
pub const COLOR_DARK_YELLOW: Color = [191, 191, 0, 255];
pub const COLOR_DARK_LIME: Color = [143, 191, 0, 255];
pub const COLOR_DARK_CHARTREUSE: Color = [95, 191, 0, 255];
pub const COLOR_DARK_GREEN: Color = [0, 191, 0, 255];
pub const COLOR_DARK_SEA: Color = [0, 191, 95, 255];
pub const COLOR_DARK_TURQUOISE: Color = [0, 191, 143, 255];
pub const COLOR_DARK_CYAN: Color = [0, 191, 191, 255];
pub const COLOR_DARK_SKY: Color = [0, 143, 191, 255];
pub const COLOR_DARK_AZURE: Color = [0, 95, 191, 255];
pub const COLOR_DARK_BLUE: Color = [0, 0, 191, 255];
pub const COLOR_DARK_HAN: Color = [47, 0, 191, 255];
pub const COLOR_DARK_VIOLET: Color = [95, 0, 191, 255];
pub const COLOR_DARK_PURPLE: Color = [143, 0, 191, 255];
pub const COLOR_DARK_FUCHSIA: Color = [191, 0, 191, 255];
pub const COLOR_DARK_MAGENTA: Color = [191, 0, 143, 255];
pub const COLOR_DARK_PINK: Color = [191, 0, 95, 255];
pub const COLOR_DARK_CRIMSON: Color = [191, 0, 47, 255];
pub const COLOR_DARKER_RED: Color = [127, 0, 0, 255];
pub const COLOR_DARKER_FLAME: Color = [127, 31, 0, 255];
pub const COLOR_DARKER_ORANGE: Color = [127, 63, 0, 255];
pub const COLOR_DARKER_AMBER: Color = [127, 95, 0, 255];
pub const COLOR_DARKER_YELLOW: Color = [127, 127, 0, 255];
pub const COLOR_DARKER_LIME: Color = [95, 127, 0, 255];
pub const COLOR_DARKER_CHARTREUSE: Color = [63, 127, 0, 255];
pub const COLOR_DARKER_GREEN: Color = [0, 127, 0, 255];
pub const COLOR_DARKER_SEA: Color = [0, 127, 63, 255];
pub const COLOR_DARKER_TURQUOISE: Color = [0, 127, 95, 255];
pub const COLOR_DARKER_CYAN: Color = [0, 127, 127, 255];
pub const COLOR_DARKER_SKY: Color = [0, 95, 127, 255];
pub const COLOR_DARKER_AZURE: Color = [0, 63, 127, 255];
pub const COLOR_DARKER_BLUE: Color = [0, 0, 127, 255];
pub const COLOR_DARKER_HAN: Color = [31, 0, 127, 255];
pub const COLOR_DARKER_VIOLET: Color = [63, 0, 127, 255];
pub const COLOR_DARKER_PURPLE: Color = [95, 0, 127, 255];
pub const COLOR_DARKER_FUCHSIA: Color = [127, 0, 127, 255];
pub const COLOR_DARKER_MAGENTA: Color = [127, 0, 95, 255];
pub const COLOR_DARKER_PINK: Color = [127, 0, 63, 255];
pub const COLOR_DARKER_CRIMSON: Color = [127, 0, 31, 255];
pub const COLOR_DARKEST_RED: Color = [63, 0, 0, 255];
pub const COLOR_DARKEST_FLAME: Color = [63, 15, 0, 255];
pub const COLOR_DARKEST_ORANGE: Color = [63, 31, 0, 255];
pub const COLOR_DARKEST_AMBER: Color = [63, 47, 0, 255];
pub const COLOR_DARKEST_YELLOW: Color = [63, 63, 0, 255];
pub const COLOR_DARKEST_LIME: Color = [47, 63, 0, 255];
pub const COLOR_DARKEST_CHARTREUSE: Color = [31, 63, 0, 255];
pub const COLOR_DARKEST_GREEN: Color = [0, 63, 0, 255];
pub const COLOR_DARKEST_SEA: Color = [0, 63, 31, 255];
pub const COLOR_DARKEST_TURQUOISE: Color = [0, 63, 47, 255];
pub const COLOR_DARKEST_CYAN: Color = [0, 63, 63, 255];
pub const COLOR_DARKEST_SKY: Color = [0, 47, 63, 255];
pub const COLOR_DARKEST_AZURE: Color = [0, 31, 63, 255];
pub const COLOR_DARKEST_BLUE: Color = [0, 0, 63, 255];
pub const COLOR_DARKEST_HAN: Color = [15, 0, 63, 255];
pub const COLOR_DARKEST_VIOLET: Color = [31, 0, 63, 255];
pub const COLOR_DARKEST_PURPLE: Color = [47, 0, 63, 255];
pub const COLOR_DARKEST_FUCHSIA: Color = [63, 0, 63, 255];
pub const COLOR_DARKEST_MAGENTA: Color = [63, 0, 47, 255];
pub const COLOR_DARKEST_PINK: Color = [63, 0, 31, 255];
pub const COLOR_DARKEST_CRIMSON: Color = [63, 0, 15, 255];
pub const COLOR_BRASS: Color = [191, 151, 96, 255];
pub const COLOR_COPPER: Color = [197, 136, 124, 255];
pub const COLOR_GOLD: Color = [229, 191, 0, 255];
pub const COLOR_SILVER: Color = [203, 203, 203, 255];
pub const COLOR_CELADON: Color = [172, 255, 175, 255];
pub const COLOR_PEACH: Color = [255, 159, 127, 255];
pub const COLOR_BROWN: Color = [139, 69, 19, 255];
