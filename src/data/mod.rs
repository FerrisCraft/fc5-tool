use camino::Utf8Path;
use eyre::Error;

mod chunk;
mod coord;
mod coord3;
pub mod dimension;
mod player;
mod region;
mod world;

pub(crate) use self::{
    chunk::{Chunk, Direction},
    coord::Coord,
    coord3::Coord3,
    dimension::Dimension,
    player::Player,
    region::Region,
    world::World,
};

pub(crate) type Compound = std::collections::HashMap<String, fastnbt::Value>;

#[culpa::throws]
#[tracing::instrument]
fn read_compound(path: &Utf8Path) -> Compound {
    use std::io::Read;
    let mut data = Vec::with_capacity(4096);
    flate2::read::GzDecoder::new(std::fs::File::open(path)?).read_to_end(&mut data)?;
    fastnbt::from_bytes(&data)?
}

#[culpa::throws]
#[tracing::instrument(skip(value))]
fn write_compound(path: &Utf8Path, value: &Compound) {
    use std::io::Write;
    flate2::write::GzEncoder::new(std::fs::File::create(path)?, flate2::Compression::best())
        .write_all(&fastnbt::to_bytes(value)?)?;
}
