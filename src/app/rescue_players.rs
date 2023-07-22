use eyre::{bail, Error};

use crate::{
    config::Coord3,
    data::{Coord, World},
};

#[culpa::throws]
pub(super) fn run(world: &World, safe_position: Coord3) {
    for uuid in world.players()? {
        let uuid = uuid?;
        let mut player = world.player(uuid)?;

        let Some(fastnbt::Value::List(position)) = player.get_mut("Pos") else {
            bail!("bad Pos")
        };
        let [fastnbt::Value::Double(x), fastnbt::Value::Double(y), fastnbt::Value::Double(z)] =
            &mut position[..]
        else {
            bail!("bad Pos")
        };

        let chunk_coord = Coord {
            x: *x as i64,
            z: *z as i64,
        }
        .block_to_chunk();
        if let Some(mut region) = world.region_for_chunk(chunk_coord)? {
            if region.chunk(chunk_coord).is_ok() {
                // Player is still in bounds
                continue;
            }
        }

        *x = safe_position.x;
        *y = safe_position.y;
        *z = safe_position.z;

        world.save_player(uuid, player)?;
    }
}
