use rltk::RGBA;

pub struct Palette;
impl Palette {
    pub const MAIN_BG: rltk::RGBA = RGBA {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    };
    pub const MAIN_FG: rltk::RGBA = RGBA {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.,
    };
    pub const COLOR_PURPLE: rltk::RGBA = RGBA {
        r: 1.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
    pub const COLOR_RED: rltk::RGBA = RGBA {
        r: 1.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
    pub const COLOR_GREEN: rltk::RGBA = RGBA {
        r: 0.,
        g: 0.7,
        b: 0.,
        a: 1.,
    };
    pub const COLOR_GREEN_DARK: rltk::RGBA = RGBA {
        r: 0.,
        g: 0.2,
        b: 0.,
        a: 1.,
    };
    pub const COLOR_3: rltk::RGBA = RGBA {
        r: 0.7,
        g: 0.2,
        b: 0.2,
        a: 1.,
    };
    pub const COLOR_4: rltk::RGBA = RGBA {
        r: 0.7,
        g: 0.7,
        b: 0.,
        a: 1.,
    };
    pub const COLOR_AMBER: rltk::RGBA = RGBA {
        r: 1.,
        g: 0.74,
        b: 0.,
        a: 1.,
    };
    pub const COLOR_WOOD: RGBA = RGBA {
        r: 0.45,
        g: 0.38,
        b: 0.26,
        a: 1.,
    };
    pub const COLOR_DIRT: RGBA = RGBA {
        r: 0.6,
        g: 0.46,
        b: 0.32,
        a: 1.,
    };
    pub const COLOR_WATER: RGBA = RGBA {
        r: 0.0,
        g: 0.0,
        b: 0.82,
        a: 1.,
    };
    pub const COLOR_FIRE: RGBA = RGBA {
        r: 0.88,
        g: 0.34,
        b: 0.13,
        a: 1.,
    };
    pub const COLOR_CEDAR: RGBA = RGBA {
        r: 0.39,
        g: 0.22,
        b: 0.17,
        a: 1.,
    };
    pub const COLOR_CLEAR: RGBA = RGBA {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    };
    pub const FACTION_COLORS: [RGBA; 2] = [
        RGBA {
            r: 1.0,
            g: 0.,
            b: 0.,
            a: 1.,
        },
        RGBA {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.,
        },
    ];
}
