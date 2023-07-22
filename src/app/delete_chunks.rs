use eyre::Error;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    config::{Config, PersistentArea},
    data::{Coord, World},
};

#[culpa::throws]
pub(super) fn run(world: &World, config: &Config) {
    let PersistentArea::Square {
        top_left: tl,
        bottom_right: br,
    } = config.persistent[0];

    let tlr = tl.chunk_to_region();
    let brr = br.chunk_to_region();
    let kept_regions = BTreeSet::from_iter(
        ((tlr.x)..=(brr.x)).flat_map(|x| ((tlr.z)..=(brr.z)).map(move |z| Coord { x, z })),
    );
    let all_regions =
        Result::<BTreeSet<_>, _>::from_iter(world.regions()?.map(|r| Ok::<_, Error>(r?.coord)))?;
    let deleted_regions = &all_regions - &kept_regions;
    for coord in deleted_regions {
        world.remove_region(coord)?;
    }
    let deleted_chunks = ((tlr.x << 5)..(tl.x))
        .flat_map(|x| ((tlr.z << 5)..((brr.z + 1) << 5)).map(move |z| Coord { x, z }))
        .chain(
            ((tlr.z << 5)..(tl.z))
                .flat_map(|z| ((tlr.x << 5)..((brr.x + 1) << 5)).map(move |x| Coord { x, z })),
        )
        .chain(
            ((br.x + 1)..((brr.x + 1) << 5))
                .flat_map(|x| ((tlr.z << 5)..((brr.z + 1) << 5)).map(move |z| Coord { x, z })),
        )
        .chain(
            ((br.z + 1)..((brr.z + 1) << 5))
                .flat_map(|z| ((tlr.x << 5)..((brr.x + 1) << 5)).map(move |x| Coord { x, z })),
        );

    let mut deleted_chunk_map = BTreeMap::<Coord<i64>, BTreeSet<Coord<i64>>>::new();
    for coord in deleted_chunks {
        deleted_chunk_map
            .entry(coord.chunk_to_region())
            .or_default()
            .insert(coord);
    }
    for (region_coord, chunks) in deleted_chunk_map {
        if let Some(mut region) = world.region(region_coord)? {
            for &chunk_coord in
                chunks.intersection(&Result::<BTreeSet<_>, _>::from_iter(region.chunks())?)
            {
                region.remove_chunk(chunk_coord)?;
            }
        }
    }
}
