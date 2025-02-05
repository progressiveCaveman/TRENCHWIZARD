use crate::simulator::components::{Position, SpatialKnowledge, Vision};
use crate::simulator::map::Map;
use crate::utils::InvalidPoint;
use rltk;
use rltk::Point;
use shipyard::{Get, IntoIter, IntoWithId, UniqueView, View, ViewMut};

pub fn run_visibility_system(
    map: UniqueView<Map>,
    vpos: View<Position>,
    mut vvs: ViewMut<Vision>,
    mut vspace: ViewMut<SpatialKnowledge>,
) {
    for (id, (pos, vs)) in (&vpos, &mut vvs).iter().with_id() {
        // if vs.dirty {
        let pos = pos.ps.first().unwrap();

        vs.dirty = false;
        vs.visible_tiles = rltk::field_of_view(Point::new(pos.x, pos.y), vs.range, &*map);
        vs.visible_tiles
            .retain(|p| p.x >= 0 && p.x < map.size.0 && p.y >= 0 && p.y < map.size.1);

        if let Ok(space) = (&mut vspace).get(id) {
            for vis in vs.visible_tiles.iter() {
                let idx = map.xy_idx(vis.to_xy());
                space.tiles.insert(idx, (map.tiles[idx], map.tile_content[idx].clone()));
            }
        }
        // }
    }
}
