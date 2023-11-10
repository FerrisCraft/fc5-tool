use camino::Utf8Path;
use eyre::{ensure, Error};
use std::collections::HashMap;

use crate::data::{dimension, Coord, Coord3};

#[derive(Clone, PartialEq, Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) players: Players,

    #[serde(default)]
    pub(crate) entities: Entities,

    #[serde(default)]
    pub(crate) dimension: HashMap<dimension::Kind, Dimension>,
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

#[serde_with::serde_as]
#[derive(Copy, Clone, PartialEq, Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum OutOfBounds {
    /// Re-locate players to persistent chunks,
    // TODO: to their current spawn location if that is persistent, otherwise
    /// to the defined safe position
    #[serde(rename_all = "kebab-case")]
    Relocate(Relocate),

    /// Persist a square of size×size chunks centered on the player
    PersistChunks {
        /// Size of the square, will round up to the nearest odd value
        size: u32,

        /// Blending settings to apply to the area around each player,
        /// if unset no blending will be applied
        #[serde(default)]
        blending: Option<Blending>,
    },
}

#[derive(Copy, Clone, PartialEq, Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Relocate {
    pub(crate) dimension: dimension::Kind,
    pub(crate) position: Coord3,
}

#[derive(Clone, PartialEq, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Entities {
    /// Whether to delete entities from removed chunks
    #[serde(default)]
    pub(crate) cull: bool,
}

#[serde_with::serde_as]
#[derive(Clone, PartialEq, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Dimension {
    /// Areas of this dimension to persist through --delete-chunks passes
    #[serde(default)]
    #[serde_as(as = "Vec<serde_with::TryFromInto<UnvalidatedPersistentArea>>")]
    pub(crate) persistent: Vec<PersistentArea>,
}

#[derive(Copy, Clone, PartialEq, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Blending {
    /// Offset to apply to height data in blended chunks,
    /// if unset will delegate to minecraft to create height data
    /// (note that setting 0 is different from unset,
    /// as this tool may generate height data differently from minecraft)
    #[serde(default)]
    pub(crate) offset: Option<f64>,
}

#[derive(Copy, Clone, PartialEq, Debug, serde::Deserialize)]
#[serde(untagged, rename_all = "kebab-case")]
enum UnvalidatedPersistentArea {
    #[serde(rename_all = "kebab-case")]
    Square {
        top_left: Coord<i64>,
        bottom_right: Coord<i64>,
        #[serde(default)]
        blending: Option<Blending>,
    },
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum PersistentArea {
    /// Persist a square area, defined by (inclusive) corner chunks
    Square {
        /// Top-left (most negative x and z) corner chunk to include
        top_left: Coord<i64>,

        /// Bottom-right (most positive x and z) corner chunk to include
        bottom_right: Coord<i64>,

        /// Blending settings to apply to this area, if unset no blending will be applied
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

impl From<PersistentArea> for UnvalidatedPersistentArea {
    fn from(area: PersistentArea) -> Self {
        match area {
            PersistentArea::Square {
                top_left,
                bottom_right,
                blending,
            } => Self::Square {
                top_left,
                bottom_right,
                blending,
            },
        }
    }
}

impl TryFrom<UnvalidatedPersistentArea> for PersistentArea {
    type Error = Error;

    #[culpa::throws]
    fn try_from(area: UnvalidatedPersistentArea) -> Self {
        match area {
            UnvalidatedPersistentArea::Square {
                top_left,
                bottom_right,
                blending,
            } => {
                ensure!(
                    top_left.x <= bottom_right.x,
                    "top-left is to the right of bottom-right"
                );
                ensure!(
                    top_left.z <= bottom_right.z,
                    "top-left is below the bottom-right"
                );
                ensure!(top_left != bottom_right, "area is empty");
                Self::Square {
                    top_left,
                    bottom_right,
                    blending,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        dimension, Blending, Config, Coord, Coord3, Dimension, Entities, HashMap, OutOfBounds,
        PersistentArea, Players, Relocate,
    };
    use eyre::Error;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
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
                entities: Entities { cull: false },
                dimension: HashMap::new(),
            }
        );

        assert_eq!(
            Config::from_str(indoc! { "
                [players.out-of-bounds.persist-chunks]
                size = 3
                # empty inline table to use builtin blending
                blending = {}
            " })?,
            Config {
                players: Players {
                    out_of_bounds: Some(OutOfBounds::PersistChunks {
                        size: 3,
                        blending: Some(Blending { offset: None }),
                    }),
                },
                entities: Entities { cull: false },
                dimension: HashMap::new(),
            }
        );

        assert_eq!(
            Config::from_str(indoc! { "
                [players.out-of-bounds.persist-chunks]
                size = 3
            " })?,
            Config {
                players: Players {
                    out_of_bounds: Some(OutOfBounds::PersistChunks {
                        size: 3,
                        blending: None,
                    }),
                },
                entities: Entities { cull: false },
                dimension: HashMap::new(),
            }
        );

        assert_eq!(
            Config::from_str(indoc! { r#"
                [players.out-of-bounds.relocate]
                dimension = "overworld"
                position = { x = -20.5, y = 70, z = 21.5 }

                [entities]
                cull = true

                [[dimension.overworld.persistent]]
                top-left = { x = -31, z = -31 }
                bottom-right = { x = 31, z = 31 }
                blending.offset = 10

                [[dimension.overworld.persistent]]
                top-left = { x = 100, z = 100 }
                bottom-right = { x = 101, z = 101 }
            "# })?,
            Config {
                players: Players {
                    out_of_bounds: Some(OutOfBounds::Relocate(Relocate {
                        dimension: dimension::Kind::Overworld,
                        position: Coord3 {
                            x: -20.5,
                            y: 70.0,
                            z: 21.5
                        },
                    })),
                },
                entities: Entities { cull: true },
                dimension: HashMap::from_iter([(
                    dimension::Kind::Overworld,
                    Dimension {
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
                ),]),
            }
        );
    }

    #[test]
    #[culpa::throws]
    fn bad_persistent_area_coordinates() {
        assert_eq!(
            Config::from_str(indoc! { r#"
                [[dimension.overworld.persistent]]
                top-left = { x = 31, z = 31 }
                bottom-right = { x = -31, z = -31 }
            "# })
            .unwrap_err()
            .to_string(),
            indoc! { "
                TOML parse error at line 1, column 1
                  |
                1 | [[dimension.overworld.persistent]]
                  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                top-left is to the right of bottom-right
            " },
        );

        assert_eq!(
            Config::from_str(indoc! { r#"
                [[dimension.overworld.persistent]]
                top-left = { x = -31, z = 31 }
                bottom-right = { x = 31, z = -31 }
            "# })
            .unwrap_err()
            .to_string(),
            indoc! { "
                TOML parse error at line 1, column 1
                  |
                1 | [[dimension.overworld.persistent]]
                  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                top-left is below the bottom-right
            " },
        );

        assert_eq!(
            Config::from_str(indoc! { r#"
                [[dimension.overworld.persistent]]
                top-left = { x = 31, z = 31 }
                bottom-right = { x = 31, z = 31 }
            "# })
            .unwrap_err()
            .to_string(),
            indoc! { "
                TOML parse error at line 1, column 1
                  |
                1 | [[dimension.overworld.persistent]]
                  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                area is empty
            " },
        );
    }
}
