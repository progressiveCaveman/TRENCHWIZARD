use rltk::{BaseMap, DijkstraMap, NavigationPath, Point, RGBA};
use shipyard::EntityId;
use crate::map::Map;

pub mod rect;
pub mod weighted_table;

#[derive(Clone, Debug, Copy)]
pub enum Target {
    LOCATION(Point),
    ENTITY(EntityId),
}

/// returns the point adjacent to origin that will lead to target
pub fn dijkstra_backtrace(dijkstra: DijkstraMap, map: &mut Map, origin: usize, mut target: usize) -> usize {
    // dbg!("dijkstra_backtrace");
    for _ in 0..1000 {
        // dbg!("How many times does this run?");

        let neighbor_indices = map.get_available_exits(target);

        for &i in neighbor_indices.iter() {
            if i.0 == origin {
                return target;
            }

            if dijkstra.map[i.0] < dijkstra.map[target] {
                target = i.0;
            }
        }
    }

    target
}

pub fn get_neighbors(point: Point) -> Vec<Point> {
    vec![
        Point {
            x: point.x - 1,
            y: point.y - 1,
        },
        Point {
            x: point.x - 1,
            y: point.y,
        },
        Point {
            x: point.x - 1,
            y: point.y + 1,
        },
        Point {
            x: point.x,
            y: point.y - 1,
        },
        Point {
            x: point.x,
            y: point.y + 1,
        },
        Point {
            x: point.x + 1,
            y: point.y - 1,
        },
        Point {
            x: point.x + 1,
            y: point.y,
        },
        Point {
            x: point.x + 1,
            y: point.y + 1,
        },
    ]
}

// translates dir according to roguelike numpad convention - 1 is SW, 9 is NE
// pub fn get_movement(point: Point, dir: usize, movemod: i32) -> Point {
//     match dir {
//         1 => Point { x: point.x - movemod, y: point.y + movemod },
//         2 => Point { x: point.x, y: point.y + movemod },
//         3 => Point { x: point.x + movemod, y: point.y + movemod },
//         4 => Point { x: point.x - movemod, y: point.y },
//         6 => Point { x: point.x + movemod, y: point.y },
//         7 => Point { x: point.x - movemod, y: point.y - movemod },
//         8 => Point { x: point.x, y: point.y - movemod },
//         9 => Point { x: point.x + movemod, y: point.y - movemod },
//         _ => point
//     }
// }

// translates dir according to roguelike numpad convention - 1 is SW, 9 is NE
pub fn dir_to_point(pos: Point, dir: usize, dismod: i32) -> Point {
    match dir {
        1 => Point {
            x: pos.x - dismod,
            y: pos.y + dismod,
        },
        2 => Point {
            x: pos.x,
            y: pos.y + dismod,
        },
        3 => Point {
            x: pos.x + dismod,
            y: pos.y + dismod,
        },
        4 => Point {
            x: pos.x - dismod,
            y: pos.y,
        },
        6 => Point {
            x: pos.x + dismod,
            y: pos.y,
        },
        7 => Point {
            x: pos.x - dismod,
            y: pos.y - dismod,
        },
        8 => Point {
            x: pos.x,
            y: pos.y - dismod,
        },
        9 => Point {
            x: pos.x + dismod,
            y: pos.y - dismod,
        },
        _ => Point { x: pos.x, y: pos.y },
    }
}

pub fn point_plus(p1: Point, p2: Point) -> Point {
    Point {
        x: p2.x + p1.x,
        y: p2.y + p1.y,
    }
}

// pub fn point_diff(p1: Point, p2: Point) -> Point {
//     Point { x: p2.x - p1.x, y: p2.y - p1.y }
// }

pub trait Scale {
    fn scaled(&mut self, amount: f32) -> RGBA;
}

impl Scale for RGBA {
    fn scaled(&mut self, amount: f32) -> RGBA {
        RGBA {
            r: self.r * amount,
            g: self.g * amount,
            b: self.b * amount,
            a: self.a * amount,
        }
    }
}

pub trait InvalidPoint {
    fn invalid_point() -> Point;
}

impl InvalidPoint for Point {
    fn invalid_point() -> Point {
        Point { x: -1, y: -1 }
    }
}

pub fn get_path(map: &Map, from: Point, tp: Point) -> NavigationPath {
    let path = rltk::a_star_search(map.point_idx(from) as i32, map.point_idx(tp) as i32, map);

    return path;
}

pub fn normalize(num: i32) -> i32 {
    if num == 0 {
        0
    } else {
        num / num.abs()
    }
}
