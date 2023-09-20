use super::Coord;

#[derive(Copy, Clone, PartialEq, Debug, serde::Deserialize)]
pub(crate) struct Coord3 {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl Coord3 {
    pub(crate) fn to_coord(self) -> Coord<i64> {
        #[allow(clippy::as_conversions)] // no alternative yet
        Coord {
            x: self.x as i64,
            z: self.z as i64,
        }
    }
}

impl std::fmt::Display for Coord3 {
    #[culpa::throws(std::fmt::Error)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) {
        let Self { x, y, z } = self;
        write!(f, "{x},{y},{z}")?;
    }
}
