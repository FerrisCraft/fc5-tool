use eyre::{bail, ensure, Context, Error};
use itertools::Itertools;

use super::{Compound, Coord};

#[derive(Debug)]
pub(crate) struct Chunk {
    pub(crate) relative_coord: Coord<usize>,
    pub(crate) absolute_coord: Coord<i64>,
    pub(crate) data: Compound,
}

// Calculates average heights around the border of the heightmap,
// starting in the top-right winding widdershins:
//
//        3 2 1 0
//      4         f
//      5         e
//      6         d
//      7         c
//        8 9 a b
fn calculate_blending_heights(heightmap: [[i16; 16]; 16], offset: f64) -> [f64; 16] {
    fn average(iter: impl Iterator<Item = i16>) -> impl Iterator<Item = f64> {
        let chunks = iter.chunks(4);
        let mut values = chunks.into_iter().map(|v| f64::from(v.sum::<i16>()) / 4.0);
        [(); 4]
            .map(|()| {
                values
                    .next()
                    .expect("by construction there will be enough values")
            })
            .into_iter()
    }
    let mut values = average(heightmap[0].into_iter().rev())
        .chain(average(heightmap.iter().map(|&[v, ..]| v)))
        .chain(average(heightmap.iter().map(|&[.., v]| v).rev()))
        .chain(average(heightmap[15].into_iter()));
    [(); 16].map(|()| {
        values
            .next()
            .expect("by construction there will be enough values")
            .floor()
            + offset
    })
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Direction {
    North,
    East,
    West,
    South,
}

impl Chunk {
    #[culpa::throws]
    #[tracing::instrument(skip(data), fields(chunk.relative_coord = %relative_coord, chunk.absolute_coord = %absolute_coord))]
    pub(super) fn parse(
        relative_coord: Coord<usize>,
        absolute_coord: Coord<i64>,
        data: &[u8],
    ) -> Self {
        Self {
            relative_coord,
            absolute_coord,
            data: fastnbt::from_bytes(data).context("parsing chunk")?,
        }
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(chunk.absolute_coord = %self.absolute_coord))]
    pub(super) fn serialize(&self) -> Vec<u8> {
        fastnbt::to_bytes(&self.data)?
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(chunk.absolute_coord = %self.absolute_coord))]
    pub(crate) fn force_blending(&mut self) {
        // ensure!(self.data.get("Status") == Some(&fastnbt::Value::String("minecraft:full".into())), "chunk is not fully generated");
        self.data.remove("isLightOn");
        self.data.insert(
            "blending_data".into(),
            fastnbt::nbt!({
                "min_section": -4,
                "max_section": 20,
            }),
        );
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(chunk.absolute_coord = %self.absolute_coord))]
    pub(crate) fn force_blending_with_heights(&mut self, directions: [Direction; 2], offset: f64) {
        // ensure!(self.data.get("Status") == Some(&fastnbt::Value::String("minecraft:full".into())), "chunk is not fully generated");
        let heights = {
            let heights = calculate_blending_heights(self.heightmaps()?.ocean_floor()?, offset);
            let mut base = [f64::MAX; 16];
            for direction in directions {
                match direction {
                    Direction::North => {
                        base[0..4].copy_from_slice(&heights[0..4]);
                    }
                    Direction::West => {
                        // yes, don't ask me why
                        base[3..7].copy_from_slice(&heights[4..8]);
                    }
                    Direction::South => {
                        // yes, don't ask me why
                        base[7..11].copy_from_slice(&heights[8..12]);
                        base[11] = heights[11];
                    }
                    Direction::East => {
                        base[12..16].copy_from_slice(&heights[12..16]);
                    }
                };
            }
            base
        };

        self.data.remove("isLightOn");
        self.data.insert(
            "blending_data".into(),
            fastnbt::nbt!({
                "min_section": -4,
                "max_section": 20,
                "heights": heights,
            }),
        );
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(chunk.absolute_coord = %self.absolute_coord))]
    pub(crate) fn heightmaps(&self) -> Heightmaps<'_> {
        let Some(fastnbt::Value::Compound(data)) = self.data.get("Heightmaps") else {
            bail!("bad Heightmaps")
        };
        Heightmaps { chunk: self, data }
    }
}

#[derive(Debug)]
pub(crate) struct Heightmaps<'a> {
    chunk: &'a Chunk,
    data: &'a Compound,
}

fn bitand<T, U>(x: T, y: U) -> U
where
    T: std::ops::BitAnd<T> + From<U>,
    U: TryFrom<T::Output>,
    U::Error: std::fmt::Debug,
{
    U::try_from(x & T::from(y)).expect("cannot overflow because of the &")
}

impl Heightmaps<'_> {
    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(chunk.absolute_coord = %self.chunk.absolute_coord))]
    pub(crate) fn ocean_floor(&self) -> [[i16; 16]; 16] {
        let Some(fastnbt::Value::LongArray(data)) = self.data.get("OCEAN_FLOOR") else {
            bail!("bad OCEAN_FLOOR")
        };
        let values = Vec::from_iter(data.iter().map(|&i| 0u64.wrapping_add_signed(i)).flat_map(
            |u| {
                (0..7).map(move |j| {
                    i16::try_from(bitand(u >> (j * 9), 0x1ff_u16))
                        .expect("cannot overflow because 0x1ff < i16::MAX")
                })
            },
        ));
        ensure!(values.len() >= 16 * 16, "not enough values in heightmap");
        let mut values = values.into_iter();
        [(); 16].map(|()| {
            [(); 16].map(|()| {
                values
                    .next()
                    .expect("check above verified there will be enough values")
                    - 64
            })
        })
    }
}
