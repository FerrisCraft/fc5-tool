use eyre::Error;
use std::collections::BTreeSet;

use crate::{
    config::{Config, PersistentArea},
    data::{Coord, World},
};

#[culpa::throws]
pub(super) fn run(world: &World, config: &Config) {
    let kept_regions = BTreeSet::from_iter(config.persistent.iter().flat_map(
        |PersistentArea::Square {
             top_left: tl,
             bottom_right: br,
             ..
         }| {
            let tlr = tl.chunk_to_region();
            let brr = br.chunk_to_region();
            ((tlr.x)..=(brr.x)).flat_map(move |x| ((tlr.z)..=(brr.z)).map(move |z| Coord { x, z }))
        },
    ));

    let kept_chunks = BTreeSet::from_iter(config.persistent.iter().flat_map(
        |PersistentArea::Square {
             top_left: tl,
             bottom_right: br,
             ..
         }| {
            ((tl.x)..=(br.x)).flat_map(move |x| ((tl.z)..=(br.z)).map(move |z| Coord { x, z }))
        },
    ));

    let all_regions =
        Result::<BTreeSet<_>, _>::from_iter(world.regions()?.map(|r| Ok::<_, Error>(r?.coord)))?;

    let mut deleted_region_count = 0;
    for coord in &all_regions - &kept_regions {
        let _guard = tracing::info_span!("delete_region", region.coord = %coord).entered();
        world.remove_region(coord)?;
        tracing::debug!("Deleted region {coord}");
        deleted_region_count += 1;
    }

    let mut deleted_chunk_count = 0;
    for region_coord in kept_regions {
        let _guard =
            tracing::info_span!("delete_chunks_in_region", region.coord = %region_coord).entered();
        if let Some(mut region) = world.region(region_coord)? {
            let all_chunks = Result::<BTreeSet<_>, _>::from_iter(region.chunks())?;
            for chunk_coord in &all_chunks - &kept_chunks {
                let _guard =
                    tracing::info_span!("delete_chunk", chunk.absolute_coord = %chunk_coord)
                        .entered();
                region.remove_chunk(chunk_coord)?;
                tracing::debug!("Deleted chunk {chunk_coord} from {region_coord}");
                deleted_chunk_count += 1;
            }
        }
    }

    tracing::info!("Deleted {deleted_region_count} regions and {deleted_chunk_count} chunks");
}
