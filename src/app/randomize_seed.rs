use eyre::{bail, Error};

use crate::data::World;

#[culpa::throws]
pub(super) fn run(world: &World) {
    let mut level = world.level()?;
    let Some(fastnbt::Value::Compound(data)) = level.get_mut("Data") else {
        bail!("bad Data")
    };
    let Some(fastnbt::Value::Compound(settings)) = data.get_mut("WorldGenSettings") else {
        bail!("bad WorldGenSettings")
    };
    let Some(fastnbt::Value::Long(seed)) = settings.get_mut("seed") else {
        bail!("bad seed")
    };
    let new_seed = rand::random();
    let old_seed = std::mem::replace(seed, new_seed);
    world.save_level(&level)?;
    tracing::info!("Randomized seed from {old_seed} to {new_seed}");
}
