use eyre::{bail, ensure, Context, ContextCompat, Error};

use crate::data::{Coord, World};

#[derive(Copy, Clone, Debug)]
struct Coord3 {
    x: f64,
    y: f64,
    z: f64,
}

impl std::str::FromStr for Coord3 {
    type Err = Error;

    #[culpa::throws]
    #[tracing::instrument]
    fn from_str(s: &str) -> Self {
        let mut it = s.split(',');
        let x = it
            .next()
            .context("missing coordinate")
            .and_then(|x| Ok(x.parse()?))
            .context("reading x coordinate")?;
        let y = it
            .next()
            .context("missing coordinate")
            .and_then(|y| Ok(y.parse()?))
            .context("reading y coordinate")?;
        let z = it
            .next()
            .context("missing coordinate")
            .and_then(|z| Ok(z.parse()?))
            .context("reading z coordinate")?;
        ensure!(it.next().is_none(), "extra data");
        Self { x, y, z }
    }
}

impl std::fmt::Display for Coord3 {
    #[culpa::throws(std::fmt::Error)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) {
        let Self { x, y, z } = self;
        write!(f, "{x},{y},{z}")?;
    }
}

#[derive(Debug, clap::Parser)]
pub(crate) struct Command {
    safe_position: Coord3,
}

impl Command {
    #[culpa::throws]
    pub(super) fn run(self, world: World) {
        for uuid in world.players()? {
            let uuid = uuid?;
            let mut player = world.player(uuid)?;

            let Some(fastnbt::Value::List(position)) = player.get_mut("Pos") else {
                bail!("bad Pos")
            };
            let [fastnbt::Value::Double(x), fastnbt::Value::Double(y), fastnbt::Value::Double(z)] =
                &mut position[..]
            else {
                bail!("bad Pos")
            };

            let chunk_coord = Coord {
                x: *x as i64,
                z: *z as i64,
            }
            .block_to_chunk();
            if let Some(mut region) = world.region_for_chunk(chunk_coord)? {
                if region.chunk(chunk_coord).is_ok() {
                    // Player is still in bounds
                    continue;
                }
            }

            *x = self.safe_position.x;
            *y = self.safe_position.y;
            *z = self.safe_position.z;

            world.save_player(uuid, player)?;
        }
    }
}
