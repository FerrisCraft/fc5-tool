use camino::Utf8Path;
use eyre::Error;

use crate::data::{Coord, Coord3};

#[derive(Clone, PartialEq, Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) players: Players,

    #[serde(default)]
    /// Default blending settings to apply to areas that don't define their own
    pub(crate) blending: Blending,

    /// Areas of the world to persist through --delete-chunks passes
    #[serde(default)]
    pub(crate) persistent: Vec<PersistentArea>,
}

impl Config {
    #[culpa::throws]
    #[tracing::instrument]
    pub(crate) fn load(path: &Utf8Path) -> Self {
        std::fs::read_to_string(path)?.parse()?
    }
}

impl std::str::FromStr for Config {
    type Err = Error;

    #[culpa::throws]
    fn from_str(s: &str) -> Self {
        toml::from_str(s)?
    }
}

#[derive(Clone, PartialEq, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Players {
    /// How to deal with players that are out of bounds after a --delete-chunks pass
    #[serde(default)]
    pub(crate) out_of_bounds: Option<OutOfBounds>,
}

#[derive(Copy, Clone, PartialEq, Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum OutOfBounds {
    /// Re-locate players to persistent chunks,
    // TODO: to their current spawn location if that is persistent, otherwise
    /// to the defined safe position
    #[serde(rename_all = "kebab-case")]
    Relocate { safe_position: Coord3 },
    /// Persist a square of size×size chunks centered on the player (will round up to nearest odd
    /// value)
    PersistChunks { size: i64 },
}

#[derive(Copy, Clone, PartialEq, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Blending {
    /// Offset to apply to height data in blended chunks
    #[serde(default)]
    pub(crate) offset: Option<f64>,
}

#[derive(Copy, Clone, PartialEq, Debug, serde::Deserialize)]
#[serde(untagged, rename_all = "kebab-case")]
pub(crate) enum PersistentArea {
    /// Persist a square area, defined by (inclusive) corner chunks
    #[serde(rename_all = "kebab-case")]
    Square {
        /// Top-left (most negative x and z) corner chunk to include
        top_left: Coord<i64>,

        /// Bottom-right (most positive x and z) corner chunk to include
        bottom_right: Coord<i64>,

        /// Override blending settings to apply to this area's border
        #[serde(default)]
        blending: Option<Blending>,
    },
}

impl PersistentArea {
    pub(crate) fn contains(&self, coord: Coord<i64>) -> bool {
        match self {
            Self::Square {
                top_left,
                bottom_right,
                ..
            } => {
                top_left.x <= coord.x
                    && top_left.z <= coord.z
                    && bottom_right.x >= coord.x
                    && bottom_right.z >= coord.z
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Blending, Config, Coord, Coord3, OutOfBounds, PersistentArea, Players};
    use eyre::Error;
    use std::str::FromStr;

    #[test]
    #[culpa::throws]
    fn smoke() {
        assert_eq!(
            Config::from_str("")?,
            Config {
                players: Players {
                    out_of_bounds: None
                },
                blending: Blending { offset: None },
                persistent: vec![],
            }
        );

        assert_eq!(
            Config::from_str(
                "
                [players.out-of-bounds.relocate]
                safe-position = { x = -20.5, y = 70, z = 21.5 }

                [blending]
                offset = -10

                [[persistent]]
                top-left = { x = -31, z = -31 }
                bottom-right = { x = 31, z = 31 }
                blending.offset = 10

                [[persistent]]
                top-left = { x = 100, z = 100 }
                bottom-right = { x = 101, z = 101 }
            "
            )?,
            Config {
                players: Players {
                    out_of_bounds: Some(OutOfBounds::Relocate {
                        safe_position: Coord3 {
                            x: -20.5,
                            y: 70.0,
                            z: 21.5
                        },
                    }),
                },
                blending: Blending {
                    offset: Some(-10.0)
                },
                persistent: vec![
                    PersistentArea::Square {
                        top_left: Coord { x: -31, z: -31 },
                        bottom_right: Coord { x: 31, z: 31 },
                        blending: Some(Blending { offset: Some(10.0) }),
                    },
                    PersistentArea::Square {
                        top_left: Coord { x: 100, z: 100 },
                        bottom_right: Coord { x: 101, z: 101 },
                        blending: None,
                    }
                ]
            }
        );
    }
}
