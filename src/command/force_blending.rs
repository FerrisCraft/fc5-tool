use eyre::{bail, ContextCompat, Error};
use std::collections::BTreeSet;

use crate::data::{
    Coord,
    Direction::{East, North, South, West},
    World,
};

#[derive(Debug, clap::Parser)]
pub(crate) struct Command {
    #[command(subcommand)]
    target: ForceBlendingTarget,
}

#[derive(Debug, clap::Parser)]
pub(crate) struct WithHeights {
    #[command(subcommand)]
    target: ForceBlendingTarget,

    /// Offset to apply to calculated heights
    #[arg(allow_negative_numbers(true))]
    offset: f64,
}

#[derive(Debug, clap::Parser)]
enum ForceBlendingTarget {
    /// Target all chunks
    All,

    /// Target these chunk coordinates
    Chunks { chunks: Vec<Coord<i64>> },

    /// Target chunks in the border of a square
    SquareBorder {
        /// Top-left (most negative x and z) corner
        #[arg(allow_hyphen_values(true))]
        tl: Coord<i64>,
        /// Bottom-right (most positive x and z) corner
        #[arg(allow_hyphen_values(true))]
        br: Coord<i64>,
    },
}

impl Command {
    #[culpa::throws]
    pub(super) fn run(self, world: World) {
        let coords = match self.target {
            ForceBlendingTarget::All => {
                let mut chunks = Vec::new();
                for region in world.regions()? {
                    for chunk in region?.chunks() {
                        chunks.push(chunk?);
                    }
                }
                chunks
            }
            ForceBlendingTarget::Chunks { chunks } => chunks,
            ForceBlendingTarget::SquareBorder { tl, br } => Vec::from_iter(BTreeSet::from_iter(
                ((tl.x)..=(br.x))
                    .flat_map(|x| [Coord { x, ..tl }, Coord { x, ..br }])
                    .chain(((tl.z)..=(br.z)).flat_map(|z| [Coord { z, ..tl }, Coord { z, ..br }])),
            )),
        };

        for coord in coords {
            world
                .region_for_chunk(coord)?
                .context("missing region")?
                .update_chunk(coord, |chunk| Ok(chunk.force_blending()?))?
        }
    }
}

impl WithHeights {
    #[culpa::throws]
    pub(super) fn run(self, world: World) {
        let coords = match self.target {
            ForceBlendingTarget::SquareBorder { tl, br } => Vec::from_iter(
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
            ),
            _ => bail!("unsupported target with heights"),
        };

        for (coord, directions) in coords {
            world
                .region_for_chunk(coord)?
                .context("missing region")?
                .update_chunk(coord, |chunk| {
                    Ok(chunk.force_blending_with_heights(directions, self.offset)?)
                })?
        }
    }
}
