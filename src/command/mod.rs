use eyre::Error;

use crate::data::World;

mod force_blending;
mod print_blending;
mod randomize_seed;
mod remove_chunks;
mod rescue_players;

#[derive(Debug, clap::Parser)]
pub(super) enum Command {
    RandomizeSeed(randomize_seed::Command),
    RemoveChunksOutsideSquareBorder(remove_chunks::Command),
    ForceBlending(force_blending::Command),
    ForceBlendingWithHeights(force_blending::WithHeights),
    PrintBlendingHeightMaps(print_blending::Command),
    RescueOutOfBoundsPlayers(rescue_players::Command),
}

impl Command {
    #[culpa::throws]
    pub(super) fn run(self, world: World) {
        match self {
            Command::RandomizeSeed(cmd) => cmd.run(world)?,
            Command::RemoveChunksOutsideSquareBorder(cmd) => cmd.run(world)?,
            Command::ForceBlending(cmd) => cmd.run(world)?,
            Command::ForceBlendingWithHeights(cmd) => cmd.run(world)?,
            Command::PrintBlendingHeightMaps(cmd) => cmd.run(world)?,
            Command::RescueOutOfBoundsPlayers(cmd) => cmd.run(world)?,
        }
    }
}
