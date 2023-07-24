use eyre::Error;

use crate::{
    config::{self, Config, PersistentArea},
    data::{
        Coord,
        Direction::{East, North, South, West},
        World,
    },
};

#[culpa::throws]
#[tracing::instrument(name = "blend", skip_all)]
pub(super) fn run(world: &World, config: &Config) {
    for (dimension_kind, config::Dimension { persistent }) in &config.dimension {
        let _guard = tracing::info_span!("dimension", dimension.kind = %dimension_kind).entered();

        let dimension = world.dimension(*dimension_kind);

        let coords = Vec::from_iter(
            persistent
                .iter()
                .filter_map(
                    |&PersistentArea::Square {
                         top_left: tl,
                         bottom_right: br,
                         blending,
                     }| {
                        let blending = blending?;
                        Some(
                            std::iter::once((tl, [North, West], blending))
                                .chain(std::iter::once((
                                    Coord { x: br.x, z: tl.z },
                                    [North, East],
                                    blending,
                                )))
                                .chain(std::iter::once((
                                    Coord { x: tl.x, z: br.z },
                                    [South, West],
                                    blending,
                                )))
                                .chain(std::iter::once((br, [South, East], blending)))
                                .chain(((tl.x + 1)..=(br.x - 1)).flat_map(move |x| {
                                    [
                                        (Coord { x, ..tl }, [North, South], blending),
                                        (Coord { x, ..br }, [North, South], blending),
                                    ]
                                }))
                                .chain(((tl.z + 1)..=(br.z - 1)).flat_map(move |z| {
                                    [
                                        (Coord { z, ..tl }, [East, West], blending),
                                        (Coord { z, ..br }, [East, West], blending),
                                    ]
                                })),
                        )
                    },
                )
                .flatten(),
        );

        let mut forced_chunk_count = 0;
        for (coord, directions, blending) in coords {
            let _guard = tracing::info_span!("chunk", chunk.absolute_coord = %coord).entered();
            let Some(mut region) = dimension.region_for_chunk(coord)? else {
                tracing::warn!(
                    "Missing chunk on persistent border, will result in unblended regeneration"
                );
                continue;
            };
            let Some(mut chunk) = region.chunk(coord)? else {
                tracing::warn!(
                    "Missing chunk on persistent border, will result in unblended regeneration"
                );
                continue;
            };
            if let Some(offset) = blending.offset {
                chunk.force_blending_with_heights(directions, offset)?;
            } else {
                chunk.force_blending()?;
            }
            region.save_chunk(&chunk)?;
            tracing::debug!(directions = ?directions, "Forced blending");
            forced_chunk_count += 1;
        }

        tracing::info!("Forced blending on {forced_chunk_count} chunks");
    }
}
