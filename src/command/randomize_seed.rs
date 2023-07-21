use eyre::{bail, Error};

use crate::data::World;

#[derive(Debug, clap::Parser)]
pub(crate) struct Command;

impl Command {
    #[culpa::throws]
    pub(super) fn run(self, world: World) {
        world.update_level(|mut level| {
            let Some(fastnbt::Value::Compound(data)) = level.get_mut("Data") else {
                bail!("bad Data")
            };
            let Some(fastnbt::Value::Compound(settings)) = data.get_mut("WorldGenSettings") else {
                bail!("bad WorldGenSettings")
            };
            settings.insert("seed".to_string(), fastnbt::Value::Long(rand::random()));
            Ok(level)
        })?;
    }
}
