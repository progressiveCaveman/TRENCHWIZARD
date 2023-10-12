// find an item and try to pick it up
for tile in vs.visible_tiles.iter() {
    let idx = map.xy_idx(tile.x, tile.y);
    let entities = &map.tile_content[idx];
    for e in entities.iter() {
        if let Ok(name) = world.get::<Name>(*e){
            dbg!(&name.name);
        }

        if let Ok(item) = world.get::<Item>(*e){
            if let Ok(p) = world.get::<Position>(*e){
                println!("Found an item");

                // visible_items.push(*e);
                let p = p.ps[0];

                dbg!(p);
                dbg!(pos.ps[0]);

                if p == pos.ps[0] {
                    println!("Needs want pickup");
                    //add wants to pick up intent and return
                    needs_wants_to_pick_up.push((id, *e));
                    break;
                } else {
                    retargeted = true;
                    target = p.clone();
                }
            }
        }

        // match world.get::<(Item, Position)>(*e) {
        //     Err(_e) => {},
        //     Ok(awe) => {

        //     }
        // }
    }
}