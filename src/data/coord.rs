use eyre::{ensure, Context, ContextCompat, Error};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Coord<T> {
    pub(crate) x: T,
    pub(crate) z: T,
}

impl Coord<i64> {
    #[culpa::throws]
    #[tracing::instrument]
    pub(super) fn from_region_file(name: &str) -> Self {
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
    pub(super) fn checked_mul(self, value: i64) -> Self {
        Self {
            x: self.x.checked_mul(value).context("out of range")?,
            z: self.z.checked_mul(value).context("out of range")?,
        }
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self, other), fields(self = %self, other = %other))]
    pub(super) fn checked_add(self, other: Self) -> Self {
        Self {
            x: self.x.checked_add(other.x).context("out of range")?,
            z: self.z.checked_add(other.z).context("out of range")?,
        }
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self, other), fields(self = %self, other = %other))]
    pub(super) fn checked_sub(self, other: Self) -> Self {
        Self {
            x: self.x.checked_sub(other.x).context("out of range")?,
            z: self.z.checked_sub(other.z).context("out of range")?,
        }
    }

    pub(crate) fn chunk_to_region(self) -> Self {
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

#[culpa::throws]
#[tracing::instrument(skip_all, fields(region.coord = %region_coord, chunk.absolute_coord = %absolute_chunk_coord))]
pub(super) fn make_relative(
    region_coord: Coord<i64>,
    absolute_chunk_coord: Coord<i64>,
) -> Coord<usize> {
    Coord::<usize>::try_from(absolute_chunk_coord.checked_sub(region_coord.checked_mul(32)?)?)?
}

#[culpa::throws]
#[tracing::instrument(skip_all, fields(region.coord = %region_coord, chunk.relative_coord = %relative_chunk_coord))]
pub(super) fn make_absolute(
    region_coord: Coord<i64>,
    relative_chunk_coord: Coord<usize>,
) -> Coord<i64> {
    Coord::<i64>::try_from(relative_chunk_coord)?.checked_add(region_coord.checked_mul(32)?)?
}
