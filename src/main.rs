#![allow(clippy::needless_question_mark)] // stupid lint

use camino::Utf8PathBuf;
use clap::Parser;
use eyre::{bail, ContextCompat, Error};
use std::collections::{BTreeMap, BTreeSet};
use tracing_subscriber::layer::SubscriberExt;

mod data;

use data::{Coord, World};

#[derive(Debug, clap::Parser)]
struct Args {
    /// Path to world directory
    world: Utf8PathBuf,
    #[arg(long, short)]
    debug: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Parser)]
enum Command {
    RandomizeSeed,

    /// Delete all chunks and regions outside this border
    RemoveChunksOutsideSquareBorder {
        /// Top-left (most negative x and z) corner chunk
        #[arg(allow_hyphen_values(true))]
        tl: Coord<i64>,
        /// Bottom-right (most positive x and z) corner chunk
        #[arg(allow_hyphen_values(true))]
        br: Coord<i64>,
    },

    ForceBlending {
        #[command(subcommand)]
        target: ForceBlendingTarget,
    },

    PrintBlendingHeightMaps {
        #[arg(allow_hyphen_values(true))]
        coord: Coord<i64>,
    },
}

#[derive(Debug, clap::Parser)]
enum ForceBlendingTarget {
    /// Target all chunks
    All,

    /// Target these chunk coordinates
    Chunks { chunks: Vec<Coord<i64>> },

    /// Target chunks in the border of a square
    SquareBorder {
        /// Top-left (most negative x and z) corner
        #[arg(allow_hyphen_values(true))]
        tl: Coord<i64>,
        /// Bottom-right (most positive x and z) corner
        #[arg(allow_hyphen_values(true))]
        br: Coord<i64>,
    },
}

#[culpa::throws]
fn main() {
    color_eyre::install()?;

    let args = Args::parse();

    tracing::subscriber::set_global_default({
        let mut fmt = tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .compact();
        if args.debug {
            fmt = fmt.with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER);
        }
        fmt.finish().with(tracing_error::ErrorLayer::default())
    })?;

    let world = World::new(args.world);

    match args.command {
        Command::RandomizeSeed => {
            world.update_level(|mut level| {
                let Some(fastnbt::Value::Compound(data)) = level.get_mut("Data") else {
                    bail!("bad Data")
                };
                let Some(fastnbt::Value::Compound(settings)) = data.get_mut("WorldGenSettings")
                else {
                    bail!("bad WorldGenSettings")
                };
                settings.insert("seed".to_string(), fastnbt::Value::Long(rand::random()));
                Ok(level)
            })?;
        }

        Command::RemoveChunksOutsideSquareBorder { tl, br } => {
            let tlr = tl.chunk_to_region();
            let brr = br.chunk_to_region();
            let kept_regions = BTreeSet::from_iter(
                ((tlr.x)..=(brr.x)).flat_map(|x| ((tlr.z)..=(brr.z)).map(move |z| Coord { x, z })),
            );
            let all_regions = Result::<BTreeSet<_>, _>::from_iter(
                world.regions()?.map(|r| Ok::<_, Error>(r?.coord)),
            )?;
            let deleted_regions = &all_regions - &kept_regions;
            for coord in deleted_regions {
                world.remove_region(coord)?;
            }
            let deleted_chunks =
                ((tlr.x << 5)..(tl.x))
                    .flat_map(|x| ((tlr.z << 5)..((brr.z + 1) << 5)).map(move |z| Coord { x, z }))
                    .chain(((tlr.z << 5)..(tl.z)).flat_map(|z| {
                        ((tlr.x << 5)..((brr.x + 1) << 5)).map(move |x| Coord { x, z })
                    }))
                    .chain(((br.x + 1)..((brr.x + 1) << 5)).flat_map(|x| {
                        ((tlr.z << 5)..((brr.z + 1) << 5)).map(move |z| Coord { x, z })
                    }))
                    .chain(((br.z + 1)..((brr.z + 1) << 5)).flat_map(|z| {
                        ((tlr.x << 5)..((brr.x + 1) << 5)).map(move |x| Coord { x, z })
                    }));

            let mut deleted_chunk_map = BTreeMap::<Coord<i64>, BTreeSet<Coord<i64>>>::new();
            for coord in deleted_chunks {
                deleted_chunk_map
                    .entry(coord.chunk_to_region())
                    .or_default()
                    .insert(coord);
            }
            for (region_coord, chunks) in deleted_chunk_map {
                if let Some(mut region) = world.region(region_coord)? {
                    for &chunk_coord in
                        chunks.intersection(&Result::<BTreeSet<_>, _>::from_iter(region.chunks())?)
                    {
                        region.remove_chunk(chunk_coord)?;
                    }
                }
            }
        }

        Command::ForceBlending { target } => {
            let coords = match target {
                ForceBlendingTarget::All => {
                    let mut chunks = Vec::new();
                    for region in world.regions()? {
                        for chunk in region?.chunks() {
                            chunks.push(chunk?);
                        }
                    }
                    chunks
                }
                ForceBlendingTarget::Chunks { chunks } => chunks,
                ForceBlendingTarget::SquareBorder { tl, br } => {
                    Vec::from_iter(BTreeSet::from_iter(
                        ((tl.x)..=(br.x))
                            .flat_map(|x| [Coord { x, ..tl }, Coord { x, ..br }])
                            .chain(
                                ((tl.z)..=(br.z))
                                    .flat_map(|z| [Coord { z, ..tl }, Coord { z, ..br }]),
                            ),
                    ))
                }
            };

            for coord in coords {
                world
                    .region_for_chunk(coord)?
                    .context("missing region")?
                    .update_chunk(coord, |chunk| Ok(chunk.force_blending()?))?
            }
        }

        Command::PrintBlendingHeightMaps { coord } => {
            let chunk = world
                .region_for_chunk(coord)?
                .context("missing region")?
                .read_chunk(coord)?;
            let Some(fastnbt::Value::Compound(blending)) = chunk.data.get("blending_data") else {
                bail!("bad blending_data")
            };
            let Some(fastnbt::Value::List(heights)) = blending.get("heights") else {
                bail!("bad heights")
            };
            let heights = Vec::from_iter(heights.iter().map(|height| height.as_f64().unwrap()));
            for height in heights {
                if height == f64::MAX {
                    print!("--- ");
                } else {
                    print!("{:3} ", height);
                }
            }
            println!();
            println!();
            for row in chunk.height_maps()?.ocean_floor()? {
                for cell in row {
                    print!("{:3} ", cell);
                }
                println!();
            }
        }
    }
}
