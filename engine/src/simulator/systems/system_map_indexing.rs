use crate::simulator::components::{BlocksTile, Position};
use crate::map::Map;
use shipyard::{Get, IntoIter, IntoWithId, UniqueViewMut, View, ViewMut};

pub fn run_map_indexing_system(mut map: UniqueViewMut<Map>, vpos: View<Position>, vblocks: ViewMut<BlocksTile>) {
    map.set_blocked();
    map.clear_tile_content();

    for (id, pos) in vpos.iter().with_id() {
        for pos in pos.ps.iter() {
            let idx = map.xy_idx((pos.x, pos.y));
            if idx > map.tiles.len() {
                continue;
            }

            if let Ok(_bt) = vblocks.get(id) {
                map.blocked[idx] = true;
            }

            map.tile_content[idx].push(id);
        }
    }
}
