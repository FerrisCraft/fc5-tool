use camino::Utf8PathBuf;
use eyre::Error;

use crate::{
    config::{Config, OutOfBounds},
    data::World,
};

mod force_blending;
// mod print_blending;
mod delete_chunks;
mod randomize_seed;
mod rescue_players;

#[derive(Debug, clap::Parser)]
pub(crate) struct App {
    /// Path to world directory
    world: Utf8PathBuf,

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
        let config = Config::load(&self.world.join("fc5-tool.toml"))?;

        if self.delete_chunks {
            delete_chunks::run(&world, &config)?;
            if let Some(OutOfBounds::Relocate { safe_position }) = config.players.out_of_bounds {
                rescue_players::run(&world, safe_position)?;
            }
        }

        if self.force_blending {
            force_blending::run(&world, &config)?;
        }

        if self.randomize_seed {
            randomize_seed::run(&world)?;
        }
    }
}
