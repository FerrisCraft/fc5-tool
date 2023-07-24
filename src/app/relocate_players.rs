use eyre::Error;

use crate::{
    config::{Config, OutOfBounds, PersistentArea},
    data::World,
};

#[culpa::throws]
#[tracing::instrument(name = "relocate", skip_all)]
pub(super) fn run(world: &World, config: &Config) {
    if let Some(OutOfBounds::Relocate(relocate)) = config.players.out_of_bounds {
        'players: for uuid in world.players()? {
            let uuid = uuid?;

            let _guard = tracing::info_span!("player", player.uuid = %uuid).entered();
            let mut player = world.player(uuid)?;

            let old_dimension = player.dimension()?;
            let old_position = player.position()?;
            let _guard = tracing::info_span!("from", player.dimension = %old_dimension, player.position = %old_position).entered();

            let Some(dimension) = config.dimension.get(&old_dimension) else {
                tracing::info!("Player is in disabled dimension");
                continue 'players;
            };

            let chunk = old_position.to_coord().block_to_chunk();
            for area in &dimension.persistent {
                let PersistentArea::Square {
                    top_left,
                    bottom_right,
                    ..
                } = area;
                let _guard = tracing::info_span!("persistent", area.top_left = %top_left, area.bottom_right = %bottom_right).entered();
                if area.contains(chunk) {
                    tracing::info!("Player is in bounds");
                    continue 'players;
                }
            }

            let new_dimension = relocate.dimension;
            let new_position = relocate.position;

            let _guard = tracing::info_span!("to", new.dimension = %new_dimension, new.position = %new_position).entered();
            player.set_dimension(new_dimension)?;
            player.set_position(new_position)?;

            world.save_player(&player)?;
            tracing::info!("Relocated player");
        }
    }
}
