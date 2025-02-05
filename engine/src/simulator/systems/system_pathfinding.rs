use crate::simulator::components::{DijkstraMapToMe, Position};
use crate::map::Map;
use shipyard::{IntoIter, IntoWithId, UniqueView, View, ViewMut};

pub fn run_pathfinding_system(map: UniqueView<Map>, vpos: View<Position>, mut vmaps: ViewMut<DijkstraMapToMe>) {
    for (_, (pos, dijkstra)) in (&vpos, &mut vmaps).iter().with_id() {
        let mut starts: Vec<usize> = vec![];
        for pos in pos.ps.iter() {
            starts.push(map.point_idx(*pos));
        }

        dijkstra.map = rltk::DijkstraMap::new(map.size.0, map.size.1, &starts, &*map, 100.0);
    }
}
