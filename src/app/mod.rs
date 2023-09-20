use camino::Utf8PathBuf;
use eyre::Error;
use itertools::Itertools;

use crate::{
    config::{Config, OutOfBounds, PersistentArea},
    data::{Coord, World},
};

mod force_blending;
// mod print_blending;
mod delete_chunks;
mod randomize_seed;
mod relocate_players;

#[derive(Debug, clap::Parser)]
pub(crate) struct App {
    /// Path to world directory
    world: Utf8PathBuf,

    /// Enable all stages
    #[arg(long)]
    all: bool,

    /// If relocation is configured, relocate players outside configured persistence areas
    #[arg(long)]
    relocate_players: bool,

    /// Delete all chunks outside configured persistence areas
    #[arg(long)]
    delete_chunks: bool,

    /// Force blending on all border chunks
    #[arg(long)]
    force_blending: bool,

    /// Randomize the world seed
    #[arg(long)]
    randomize_seed: bool,
}

impl App {
    #[culpa::throws]
    pub(super) fn run(self) {
        let world = World::new(&self.world);
        let mut config = Config::load(&self.world.join("fc5-tool.toml"))?;

        if let Some(OutOfBounds::PersistChunks { size, blending }) = config.players.out_of_bounds {
            let radius = size.max(1) / 2;
            let offset = Coord {
                x: radius,
                z: radius,
            };

            let new_areas = Result::<Vec<_>, _>::from_iter(world.players()?.map(|uuid| {
                let uuid = uuid?;

                let _guard = tracing::info_span!("player", player.uuid = %uuid).entered();
                let player = world.player(uuid)?;

                let position = player.position()?;
                let dimension_kind = player.dimension()?;
                let _guard = tracing::info_span!("position", player.dimension = %dimension_kind, player.position = %position).entered();

                let Some(dimension) = config.dimension.get(&dimension_kind) else {
                    tracing::info!("Player is in disabled dimension");
                    return Ok(None);
                };

                let chunk = position.to_coord().block_to_chunk();
                for area in &dimension.persistent {
                    let PersistentArea::Square { top_left, bottom_right, .. } = area;
                    let _guard = tracing::info_span!("persistent", area.top_left = %top_left, area.bottom_right = %bottom_right).entered();
                    if area.contains(chunk) {
                        tracing::info!("Player is in-bounds");
                        return Ok(None);
                    }
                }

                let top_left = chunk.checked_sub(offset)?;
                let bottom_right = chunk.checked_add(offset)?;

                let _guard = tracing::info_span!("area", new_area.top_left = %top_left, new_area.bottom_right = %bottom_right).entered();
                tracing::info!("Player is out-of-bounds, adding persistent area");

                Ok::<_, Error>(Some((dimension_kind, PersistentArea::Square { top_left, bottom_right, blending })))
            }).filter_map(Result::transpose))?;

            for (dimension, new_areas) in new_areas.into_iter().into_group_map() {
                config
                    .dimension
                    .get_mut(&dimension)
                    .expect("only configured dimensions have areas added")
                    .persistent
                    .extend(new_areas);
            }
        }

        if self.all || self.relocate_players {
            relocate_players::run(&world, &config)?;
        }

        if self.all || self.delete_chunks {
            delete_chunks::run(&world, &config)?;
        }

        if self.all || self.force_blending {
            force_blending::run(&world, &config)?;
        }

        if self.all || self.randomize_seed {
            randomize_seed::run(&world)?;
        }
    }
}
