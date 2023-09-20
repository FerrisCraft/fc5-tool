use camino::{Utf8Path, Utf8PathBuf};
use eyre::{bail, Context, ContextCompat, Error, Result};

use super::{Coord, Region};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Kind {
    Overworld,
    Nether,
    End,
}

impl std::fmt::Display for Kind {
    #[culpa::throws(std::fmt::Error)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) {
        match self {
            Kind::Overworld => f.write_str("overworld")?,
            Kind::Nether => f.write_str("nether")?,
            Kind::End => f.write_str("end")?,
        }
    }
}

impl Kind {
    pub(super) fn nbt(&self) -> fastnbt::Value {
        match self {
            Kind::Overworld => fastnbt::Value::String("minecraft:the_overworld".into()),
            Kind::Nether => fastnbt::Value::String("minecraft:the_nether".into()),
            Kind::End => fastnbt::Value::String("minecraft:the_end".into()),
        }
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(value = ?value))]
    pub(super) fn from_nbt(value: &fastnbt::Value) -> Self {
        match value.as_str().context("not string value")? {
            "minecraft:overworld" => Kind::Overworld,
            "minecraft:the_nether" => Kind::Nether,
            "minecraft:the_end" => Kind::End,
            other => bail!("unknown dimension {other}"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Dimension {
    pub(crate) kind: Kind,
    pub(crate) directory: Utf8PathBuf,
}

impl Dimension {
    pub(super) fn new(kind: Kind, relative_to: &Utf8Path) -> Self {
        Self {
            kind,
            directory: match kind {
                Kind::Overworld => relative_to.to_owned(),
                Kind::Nether => relative_to.join("DIM-1"),
                Kind::End => relative_to.join("DIM1"),
            },
        }
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(dimension.kind = %self.kind, dimension.directory = %self.directory, region.coord = %coord))]
    pub(crate) fn region(&self, coord: Coord<i64>) -> Option<Region> {
        let Coord { x, z } = coord;
        let path = self.directory.join("region").join(format!("r.{x}.{z}.mca"));
        Region::from_path(path)?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(dimension.kind = %self.kind, dimension.directory = %self.directory))]
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
    #[tracing::instrument(skip_all, fields(dimension.kind = %self.kind, dimension.directory = %self.directory, region.coord = %coord))]
    pub(crate) fn remove_region(&self, coord: Coord<i64>) {
        let Coord { x, z } = coord;
        let path = self.directory.join("region").join(format!("r.{x}.{z}.mca"));
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            res => res,
        }?;
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(dimension.kind = %self.kind, dimension.directory = %self.directory, chunk.absolute_coord = %absolute_coord))]
    pub(crate) fn region_for_chunk(&self, absolute_coord: Coord<i64>) -> Option<Region> {
        self.region(absolute_coord.chunk_to_region())?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(dimension.kind = %self.kind, dimension.directory = %self.directory, region.coord = %coord))]
    pub(crate) fn entity_region(&self, coord: Coord<i64>) -> Option<Region> {
        let Coord { x, z } = coord;
        let path = self
            .directory
            .join("entities")
            .join(format!("r.{x}.{z}.mca"));
        Region::from_path(path)?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(dimension.kind = %self.kind, dimension.directory = %self.directory))]
    pub(crate) fn entity_regions(&self) -> impl Iterator<Item = Result<Region>> {
        std::fs::read_dir(self.directory.join("entities"))
            .context("reading entities dir")?
            .filter_map(|entry| {
                entry
                    .context("reading dir entry")
                    .and_then(|entry| Ok(Region::from_path(entry.path().try_into()?)?))
                    .transpose()
            })
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(dimension.kind = %self.kind, dimension.directory = %self.directory, region.coord = %coord))]
    pub(crate) fn remove_entity_region(&self, coord: Coord<i64>) {
        let Coord { x, z } = coord;
        let path = self
            .directory
            .join("entities")
            .join(format!("r.{x}.{z}.mca"));
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            res => res,
        }?;
    }
}
