use camino::{Utf8Path, Utf8PathBuf};
use eyre::{Context, ContextCompat, Error, Result};
use uuid::Uuid;

use super::{read_compound, write_compound, Compound, Coord, Region};

#[derive(Debug)]
pub(crate) struct World {
    pub(crate) directory: Utf8PathBuf,
}

impl World {
    pub(crate) fn new(directory: &Utf8Path) -> Self {
        Self {
            directory: directory.to_owned(),
        }
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
        write_compound(&path, f(read_compound(&path)?)?)?;
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory, uuid = %uuid))]
    pub(crate) fn player(&self, uuid: Uuid) -> Compound {
        read_compound(
            &self
                .directory
                .join("playerdata")
                .join(format!("{uuid}.dat")),
        )?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory, uuid = %uuid))]
    pub(crate) fn save_player(&self, uuid: Uuid, data: Compound) {
        write_compound(
            &self
                .directory
                .join("playerdata")
                .join(format!("{uuid}.dat")),
            data,
        )?;
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory))]
    pub(crate) fn players(&self) -> impl Iterator<Item = Result<Uuid>> {
        std::fs::read_dir(self.directory.join("playerdata"))
            .context("reading region dir")?
            .filter_map(|entry| {
                entry
                    .context("reading dir entry")
                    .and_then(|entry| {
                        let filename = entry.file_name();
                        let filename = filename.to_str().context("non utf-8 filename")?;
                        let _guard =
                            tracing::info_span!("parsing filename", filename = %filename).entered();
                        let Some((uuid, "dat")) = filename.split_once('.') else {
                            return Ok(None);
                        };
                        Ok(Some(uuid.parse()?))
                    })
                    .transpose()
            })
    }
}
