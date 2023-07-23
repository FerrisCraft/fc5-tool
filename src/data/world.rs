use camino::{Utf8Path, Utf8PathBuf};
use eyre::{Context, ContextCompat, Error, Result};
use uuid::Uuid;

use super::{dimension, read_compound, write_compound, Compound, Dimension, Player};

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

    pub(crate) fn dimension(&self, kind: dimension::Kind) -> Dimension {
        Dimension::new(kind, &self.directory)
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory))]
    pub(crate) fn level(&self) -> Compound {
        read_compound(&self.directory.join("level.dat"))?
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory))]
    pub(crate) fn save_level(&self, data: &Compound) {
        write_compound(&self.directory.join("level.dat"), data)?;
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory, uuid = %uuid))]
    pub(crate) fn player(&self, uuid: Uuid) -> Player {
        Player {
            uuid,
            data: read_compound(
                &self
                    .directory
                    .join("playerdata")
                    .join(format!("{uuid}.dat")),
            )?,
        }
    }

    #[culpa::throws]
    #[tracing::instrument(skip_all, fields(world.directory = %self.directory, player.uuid = %player.uuid))]
    pub(crate) fn save_player(&self, player: &Player) {
        let Player { uuid, data } = player;
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
