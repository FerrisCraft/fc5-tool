use eyre::{bail, ContextCompat, Error};
use uuid::Uuid;

use super::{dimension, Compound, Coord3};

pub(crate) struct Player {
    pub(crate) uuid: Uuid,
    pub(crate) data: Compound,
}

impl Player {
    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(player.uuid = %self.uuid))]
    pub(crate) fn position(&self) -> Coord3 {
        let Some(fastnbt::Value::List(position)) = self.data.get("Pos") else {
            bail!("bad Pos")
        };
        let [fastnbt::Value::Double(x), fastnbt::Value::Double(y), fastnbt::Value::Double(z)] =
            position[..]
        else {
            bail!("bad Pos")
        };

        Coord3 { x, y, z }
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(player.uuid = %self.uuid, position = %position))]
    pub(crate) fn set_position(&mut self, position: Coord3) {
        self.data.insert(
            "Pos".into(),
            fastnbt::nbt!([position.x, position.y, position.z]),
        );
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(player.uuid = %self.uuid))]
    pub(crate) fn dimension(&self) -> dimension::Kind {
        let dimension = self.data.get("Dimension").context("missing Dimension")?;
        dimension::Kind::from_nbt(dimension)?
    }

    #[culpa::throws]
    #[tracing::instrument(skip(self), fields(player.uuid = %self.uuid, dimension.kind = %dimension))]
    pub(crate) fn set_dimension(&mut self, dimension: dimension::Kind) {
        self.data.insert("Dimension".into(), dimension.nbt());
    }
}
