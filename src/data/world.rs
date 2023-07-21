use camino::{Utf8Path, Utf8PathBuf};
use eyre::{Context, Error, Result};

use super::{Compound, Coord, Region};

#[derive(Debug)]
pub(crate) struct World {
    pub(crate) directory: Utf8PathBuf,
}

#[culpa::throws]
#[tracing::instrument]
fn read_level(path: &Utf8Path) -> Compound {
    use std::io::Read;
    let mut data = Vec::with_capacity(4096);
    flate2::read::GzDecoder::new(std::fs::File::open(path)?).read_to_end(&mut data)?;
    fastnbt::from_bytes(&data)?
}

#[culpa::throws]
#[tracing::instrument(skip(value))]
fn write_level(path: &Utf8Path, value: Compound) {
    use std::io::Write;
    flate2::write::GzEncoder::new(std::fs::File::create(path)?, flate2::Compression::best())
        .write_all(&fastnbt::to_bytes(&value)?)?;
}

impl World {
    pub(crate) fn new(directory: Utf8PathBuf) -> Self {
        Self { directory }
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory, region.coord = %coord))]
    pub(crate) fn region(&self, coord: Coord<i64>) -> Option<Region> {
        let Coord { x, z } = coord;
        let path = self.directory.join("region").join(format!("r.{x}.{z}.mca"));
        Region::from_path(path)?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory))]
    pub(crate) fn regions(&self) -> impl Iterator<Item = Result<Region>> {
        std::fs::read_dir(self.directory.join("region"))
            .context("reading region dir")?
            .filter_map(|entry| {
                entry
                    .context("reading dir entry")
                    .and_then(|entry| Ok(Region::from_path(entry.path().try_into()?)?))
                    .transpose()
            })
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory, region.coord = %coord))]
    pub(crate) fn remove_region(&self, coord: Coord<i64>) {
        let Coord { x, z } = coord;
        let path = self.directory.join("region").join(format!("r.{x}.{z}.mca"));
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            res => res,
        }?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory, chunk.absolute_coord = %absolute_coord))]
    pub(crate) fn region_for_chunk(&self, absolute_coord: Coord<i64>) -> Option<Region> {
        self.region(absolute_coord.chunk_to_region())?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory))]
    pub(crate) fn update_level(&self, mut f: impl FnMut(Compound) -> Result<Compound>) {
        let path = self.directory.join("level.dat");
        write_level(&path, f(read_level(&path)?)?)?;
    }
}
