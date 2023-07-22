use eyre::{ContextCompat, Error};

use crate::{
    config::{Config, PersistentArea},
    data::{
        Coord,
        Direction::{East, North, South, West},
        World,
    },
};

#[culpa::throws]
pub(super) fn run(world: &World, config: &Config) {
    let PersistentArea::Square {
        top_left: tl,
        bottom_right: br,
    } = config.persistent[0];

    let coords = Vec::from_iter(
        std::iter::once((tl, [North, West]))
            .chain(std::iter::once((Coord { x: br.x, z: tl.z }, [North, East])))
            .chain(std::iter::once((Coord { x: tl.x, z: br.z }, [South, West])))
            .chain(std::iter::once((br, [South, East])))
            .chain(((tl.x + 1)..=(br.x - 1)).flat_map(|x| {
                [
                    (Coord { x, ..tl }, [North, South]),
                    (Coord { x, ..br }, [North, South]),
                ]
            }))
            .chain(((tl.z + 1)..=(br.z - 1)).flat_map(|z| {
                [
                    (Coord { z, ..tl }, [East, West]),
                    (Coord { z, ..br }, [East, West]),
                ]
            })),
    );

    for (coord, directions) in coords {
        world
            .region_for_chunk(coord)?
            .context("missing region")?
            .update_chunk(coord, |chunk| {
                Ok(if let Some(offset) = config.blending.offset {
                    chunk.force_blending_with_heights(directions, offset)?
                } else {
                    chunk.force_blending()?
                })
            })?
    }
}
