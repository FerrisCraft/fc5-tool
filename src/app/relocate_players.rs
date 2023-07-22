use eyre::Error;

use crate::{
    config::{Config, OutOfBounds},
    data::World,
};

#[culpa::throws]
pub(super) fn run(world: &World, config: &Config) {
    if let Some(OutOfBounds::Relocate { safe_position }) = config.players.out_of_bounds {
        'players: for uuid in world.players()? {
            let uuid = uuid?;
            let mut player = world.player(uuid)?;

            let old_position = player.position()?;

            let chunk = old_position.to_coord().block_to_chunk();
            for area in &config.persistent {
                if area.contains(chunk) {
                    tracing::info!("Player {uuid} is in bounds");
                    continue 'players;
                }
            }

            let new_position = safe_position;
            player.set_position(new_position)?;

            world.save_player(&player)?;
            tracing::info!("Relocated player {uuid} from {old_position} to {new_position}");
        }
    }
}
