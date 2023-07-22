use camino::Utf8PathBuf;
use eyre::{Context, ContextCompat, Error, Result};
use std::fmt::Debug;

use super::{
    coord::{make_absolute, make_relative},
    Chunk, Coord,
};

pub(crate) struct Region {
    pub(crate) coord: Coord<i64>,
    pub(crate) path: Utf8PathBuf,
    region: fastanvil::Region<std::fs::File>,
}

impl Debug for Region {
    #[culpa::throws(std::fmt::Error)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) {
        f.debug_struct("Region")
            .field("coord", &self.coord)
            .field("path", &self.path)
            .finish()?
    }
}

impl Region {
    #[culpa::throws]
    #[tracing::instrument]
    pub(super) fn from_path(path: Utf8PathBuf) -> Option<Self> {
        match std::fs::metadata(&path) {
            Ok(metadata) if metadata.len() == 0 => return None,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            res => res,
        }?;
        let coord = Coord::from_region_file(path.file_name().context("missing filename")?)?;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .context("opening region file")?;
        let region = fastanvil::Region::from_stream(file).context("initializing region")?;
        Some(Self {
            coord,
            path,
            region,
        })
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(region.path = %self.path, region.coord = %self.coord, chunk.absolute_coord = %absolute_coord))]
    pub(crate) fn chunk(&mut self, absolute_coord: Coord<i64>) -> Chunk {
        let relative_coord = make_relative(self.coord, absolute_coord)?;
        let data = self
            .region
            .read_chunk(relative_coord.x, relative_coord.z)
            .context("reading chunk")?
            .context("chunk not in region file")?;
        Chunk::parse(relative_coord, absolute_coord, &data)?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(region.path = %self.path, region.coord = %self.coord, chunk.relative_coord = %chunk.relative_coord))]
    pub(crate) fn save_chunk(&mut self, chunk: &Chunk) {
        self.region.write_chunk(
            chunk.relative_coord.x,
            chunk.relative_coord.z,
            &chunk.serialize()?,
        )?;
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(region.path = %self.path, region.coord = %self.coord, chunk.absolute_coord = %absolute_coord, relative_coord))]
    pub(crate) fn remove_chunk(&mut self, absolute_coord: Coord<i64>) {
        let relative_coord = make_relative(self.coord, absolute_coord)?;
        tracing::Span::current().record("relative_coord", relative_coord.to_string());
        self.region
            .remove_chunk(relative_coord.x, relative_coord.z)
            .context("removing chunk")?;
    }

    #[tracing::instrument(skip_all, fields(region.path = %self.path, region.coord = %self.coord))]
    pub(crate) fn chunks(&mut self) -> impl Iterator<Item = Result<Coord<i64>>> + '_ {
        self.region.iter().map(|chunk| {
            let chunk = chunk.context("reading chunk")?;
            let relative_coord = Coord {
                x: chunk.x,
                z: chunk.z,
            };
            Ok(make_absolute(self.coord, relative_coord)?)
        })
    }
}
