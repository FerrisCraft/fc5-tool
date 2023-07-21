use eyre::{bail, ContextCompat, Error};

use crate::data::{Coord, World};

#[derive(Debug, clap::Parser)]
pub(crate) struct Command {
    #[arg(allow_hyphen_values(true))]
    coord: Coord<i64>,
}

impl Command {
    #[culpa::throws]
    pub(super) fn run(self, world: World) {
        let chunk = world
            .region_for_chunk(self.coord)?
            .context("missing region")?
            .read_chunk(self.coord)?;
        let Some(fastnbt::Value::Compound(blending)) = chunk.data.get("blending_data") else {
            bail!("bad blending_data")
        };
        let Some(fastnbt::Value::List(heights)) = blending.get("heights") else {
            bail!("bad heights")
        };
        let heights = Vec::from_iter(heights.iter().map(|height| height.as_f64().unwrap()));
        for height in heights {
            if height == f64::MAX {
                print!("--- ");
            } else {
                print!("{:3} ", height);
            }
        }
        println!();
        println!();
        for row in chunk.heightmaps()?.ocean_floor()? {
            for cell in row {
                print!("{:3} ", cell);
            }
            println!();
        }
    }
}
