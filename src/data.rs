use camino::{Utf8Path, Utf8PathBuf};
use eyre::{ensure, Context, ContextCompat, Error, Result};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

pub type Compound = std::collections::HashMap<String, fastnbt::Value>;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Coord<T> {
    pub x: T,
    pub z: T,
}

impl Coord<i64> {
    #[culpa::throws]
    #[tracing::instrument]
    fn from_region_file(name: &str) -> Self {
        let mut it = name.split('.');
        ensure!(it.next() == Some("r"), "missing `r` segment");
        let x = it
            .next()
            .context("missing coordinate")
            .and_then(|x| Ok(x.parse()?))
            .context("reading x coordinate")?;
        let z = it
            .next()
            .context("missing coordinate")
            .and_then(|z| Ok(z.parse()?))
            .context("reading z coordinate")?;
        ensure!(it.next() == Some("mca"), "missing `mca` segment");
        ensure!(it.next().is_none(), "extra data");
        Self { x, z }
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(self = %self))]
    fn checked_mul(self, value: i64) -> Self {
        Self {
            x: self.x.checked_mul(value).context("out of range")?,
            z: self.z.checked_mul(value).context("out of range")?,
        }
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self, other), fields(self = %self, other = %other))]
    fn checked_add(self, other: Self) -> Self {
        Self {
            x: self.x.checked_add(other.x).context("out of range")?,
            z: self.z.checked_add(other.z).context("out of range")?,
        }
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self, other), fields(self = %self, other = %other))]
    fn checked_sub(self, other: Self) -> Self {
        Self {
            x: self.x.checked_sub(other.x).context("out of range")?,
            z: self.z.checked_sub(other.z).context("out of range")?,
        }
    }

    pub fn chunk_to_region(self) -> Self {
        Self {
            x: self.x >> 5,
            z: self.z >> 5,
        }
    }
}

impl<T> Coord<T> {
    #[culpa::throws]
    #[tracing::instrument(skip(other), fields(other = %other))]
    fn try_from<U: Display>(other: Coord<U>) -> Self
    where
        T: TryFrom<U>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Self {
            x: other.x.try_into()?,
            z: other.z.try_into()?,
        }
    }
}

impl<T: FromStr> FromStr for Coord<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
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
        let z = it
            .next()
            .context("missing coordinate")
            .and_then(|z| Ok(z.parse()?))
            .context("reading z coordinate")?;
        ensure!(it.next().is_none(), "extra data");
        Self { x, z }
    }
}

impl<T: Display> Display for Coord<T> {
    #[culpa::throws(std::fmt::Error)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) {
        let Self { x, z } = self;
        write!(f, "{x},{z}")?;
    }
}

#[derive(Debug)]
pub struct World {
    pub directory: Utf8PathBuf,
}

#[culpa::throws]
#[tracing::instrument]
pub fn read_level(path: &Utf8Path) -> Compound {
    use std::io::Read;
    let mut data = Vec::with_capacity(4096);
    flate2::read::GzDecoder::new(std::fs::File::open(path)?).read_to_end(&mut data)?;
    fastnbt::from_bytes(&data)?
}

#[culpa::throws]
#[tracing::instrument(skip(value))]
pub fn write_level(path: &Utf8Path, value: Compound) {
    use std::io::Write;
    flate2::write::GzEncoder::new(std::fs::File::create(path)?, flate2::Compression::best())
        .write_all(&fastnbt::to_bytes(&value)?)?;
}

impl World {
    pub fn new(directory: Utf8PathBuf) -> Self {
        Self { directory }
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(self.directory = %self.directory, coord = %coord))]
    pub fn region(&self, coord: Coord<i64>) -> Option<Region> {
        let Coord { x, z } = coord;
        let path = self.directory.join("region").join(format!("r.{x}.{z}.mca"));
        Region::from_path(path)?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(self.directory = %self.directory))]
    pub fn regions(&self) -> impl Iterator<Item = Result<Region>> {
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
    #[tracing::instrument(skip_all, fields(self.directory = %self.directory, coord = %coord))]
    pub fn remove_region(&self, coord: Coord<i64>) {
        let Coord { x, z } = coord;
        let path = self.directory.join("region").join(format!("r.{x}.{z}.mca"));
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            res => res,
        }?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(self.directory = %self.directory, coord = %coord))]
    pub fn region_for_chunk(&self, coord: Coord<i64>) -> Option<Region> {
        self.region(Coord {
            x: coord.x >> 5,
            z: coord.z >> 5,
        })?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(self.directory = %self.directory))]
    pub fn update_level(&self, mut f: impl FnMut(Compound) -> Result<Compound>) {
        let path = self.directory.join("level.dat");
        write_level(&path, f(read_level(&path)?)?)?;
    }
}

pub struct Region {
    pub coord: Coord<i64>,
    pub path: Utf8PathBuf,
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

#[culpa::throws]
#[tracing::instrument(skip_all, fields(region_coord = %region_coord, absolute_chunk_coord = %absolute_chunk_coord))]
fn make_relative(region_coord: Coord<i64>, absolute_chunk_coord: Coord<i64>) -> Coord<usize> {
    Coord::<usize>::try_from(absolute_chunk_coord.checked_sub(region_coord.checked_mul(32)?)?)?
}

#[culpa::throws]
#[tracing::instrument(skip_all, fields(region_cord = %region_coord, relative_chunk_coord = %relative_chunk_coord))]
fn make_absolute(region_coord: Coord<i64>, relative_chunk_coord: Coord<usize>) -> Coord<i64> {
    Coord::<i64>::try_from(relative_chunk_coord)?.checked_add(region_coord.checked_mul(32)?)?
}

impl Region {
    #[culpa::throws]
    #[tracing::instrument]
    fn from_path(path: Utf8PathBuf) -> Option<Self> {
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
    #[tracing::instrument(skip_all, fields(self.path = %self.path, self.coord = %self.coord, chunk.relative_coord = %chunk.relative_coord))]
    fn save_chunk(&mut self, chunk: &Chunk) {
        self.region.write_chunk(
            chunk.relative_coord.x,
            chunk.relative_coord.z,
            &chunk.serialize()?,
        )?;
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(self.path = %self.path, self.coord = %self.coord, absolute_coord = %absolute_coord))]
    pub fn update_chunk(
        &mut self,
        absolute_coord: Coord<i64>,
        mut f: impl FnMut(Chunk) -> Result<Chunk>,
    ) {
        let relative_coord = make_relative(self.coord, absolute_coord)?;
        let data = self
            .region
            .read_chunk(relative_coord.x, relative_coord.z)
            .context("reading chunk")?
            .context("chunk not in region file")?;
        let chunk = Chunk::parse(relative_coord, absolute_coord, &data)?;
        self.save_chunk(&f(chunk)?)?;
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(self.path = %self.path, self.coord = %self.coord, absolute_coord = %absolute_coord, relative_coord))]
    pub fn remove_chunk(&mut self, absolute_coord: Coord<i64>) {
        let relative_coord = make_relative(self.coord, absolute_coord)?;
        tracing::Span::current().record("relative_coord", relative_coord.to_string());
        self.region
            .remove_chunk(relative_coord.x, relative_coord.z)
            .context("removing chunk")?;
    }

    #[tracing::instrument(skip_all, fields(self.path = %self.path, self.coord = %self.coord))]
    pub fn chunks(&mut self) -> impl Iterator<Item = Result<Coord<i64>>> + '_ {
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

#[derive(Debug)]
pub struct Chunk {
    pub relative_coord: Coord<usize>,
    pub absolute_coord: Coord<i64>,
    data: Compound,
}

impl Chunk {
    #[culpa::throws]
    #[tracing::instrument(skip(data), fields(relative_coord = %relative_coord, absolute_coord = %absolute_coord))]
    fn parse(relative_coord: Coord<usize>, absolute_coord: Coord<i64>, data: &[u8]) -> Self {
        Self {
            relative_coord,
            absolute_coord,
            data: fastnbt::from_bytes(data).context("parsing chunk")?,
        }
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(self.relative_coord = %self.relative_coord))]
    fn serialize(&self) -> Vec<u8> {
        fastnbt::to_bytes(&self.data)?
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(self.relative_coord = %self.relative_coord))]
    pub fn force_blending(mut self) -> Self {
        // ensure!(self.data.get("Status") == Some(&fastnbt::Value::String("minecraft:full".into())), "chunk is not fully generated");
        self.data.remove("Heightmaps");
        self.data.remove("isLightOn");
        self.data.insert(
            "blending_data".into(),
            fastnbt::nbt!({
                "min_section": -4,
                "max_section": 20,
            }),
        );
        self
    }
}
