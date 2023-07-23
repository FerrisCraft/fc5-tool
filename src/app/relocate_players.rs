use eyre::Error;

use crate::{
    config::{Config, OutOfBounds},
    data::World,
};

#[culpa::throws]
pub(super) fn run(world: &World, config: &Config) {
    if let Some(OutOfBounds::Relocate(relocate)) = config.players.out_of_bounds {
        'players: for uuid in world.players()? {
            let uuid = uuid?;

            let _guard = tracing::info_span!("relocate_player", player.uuid = %uuid).entered();
            let mut player = world.player(uuid)?;

            let old_position = player.position()?;
            let old_dimension = player.dimension()?;
            let chunk = old_position.to_coord().block_to_chunk();

            let Some(dimension) = config.dimension.get(&old_dimension) else {
                tracing::info!("Player {uuid} is in disabled dimension {old_dimension}");
                continue 'players;
            };

            for area in &dimension.persistent {
                if area.contains(chunk) {
                    tracing::info!("Player {uuid} is in bounds");
                    continue 'players;
                }
            }

            let new_dimension = relocate.dimension;
            let new_position = relocate.position;
            player.set_dimension(new_dimension)?;
            player.set_position(new_position)?;

            world.save_player(&player)?;
            tracing::info!("Relocated player {uuid} from {old_dimension} {old_position} to {new_dimension} {new_position}");
        }
    }
}
