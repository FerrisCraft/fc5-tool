mod chunk;
mod coord;
mod region;
mod world;

pub(crate) type Compound = std::collections::HashMap<String, fastnbt::Value>;

pub(crate) use self::{
    chunk::{Chunk, Direction},
    coord::Coord,
    region::Region,
    world::World,
};
