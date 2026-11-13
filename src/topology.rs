//! Explicit open-boundary topology, independent of physics and camera coordinates.

/// Number of independently addressable QR neurons.
pub const POPULATION: usize = 4096;

/// Registered topology policy; numeric codes are part of observation schema 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Topology {
    /// Six face-sharing neighbors in a 16-cubed volume.
    Cubic6 = 0,
    /// All 26 neighbors in the surrounding cube.
    Cubic26 = 1,
    /// Preserved planar eight-neighbor diagnostic condition.
    Grid2D8 = 2,
}

impl TryFrom<u8> for Topology {
    type Error = std::io::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Cubic6),
            1 => Ok(Self::Cubic26),
            2 => Ok(Self::Grid2D8),
            _ => Err(std::io::Error::other("unknown topology code")),
        }
    }
}

impl Topology {
    /// Stable policy name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Cubic6 => "Cubic6",
            Self::Cubic26 => "Cubic26",
            Self::Grid2D8 => "Grid2D8",
        }
    }
    /// Integer extents; viewer spacing never changes these dimensions.
    #[must_use]
    pub const fn dimensions(self) -> [usize; 3] {
        match self {
            Self::Grid2D8 => [64, 64, 1],
            _ => [16, 16, 16],
        }
    }
    /// Fixed maximum degree used for the external coupling divisor.
    #[must_use]
    pub const fn maximum_degree(self) -> usize {
        match self {
            Self::Cubic6 => 6,
            Self::Cubic26 => 26,
            Self::Grid2D8 => 8,
        }
    }
    /// Map in-bounds integer coordinates to their stable ID.
    #[must_use]
    pub fn id(self, p: [usize; 3]) -> Option<usize> {
        let [nx, ny, nz] = self.dimensions();
        if p[0] < nx && p[1] < ny && p[2] < nz {
            Some(p[0] + nx * (p[1] + ny * p[2]))
        } else {
            None
        }
    }
    /// Inverse stable-ID mapping.
    ///
    /// # Panics
    /// Panics when the ID is outside the 4,096-neuron population.
    #[must_use]
    pub fn coordinates(self, id: usize) -> [usize; 3] {
        assert!(id < POPULATION);
        let [nx, ny, _] = self.dimensions();
        [id % nx, (id / nx) % ny, id / (nx * ny)]
    }
    /// Unique neighbors in ascending ID order, with no wrapping.
    ///
    /// # Panics
    /// Panics for an out-of-bounds neuron ID.
    #[must_use]
    pub fn neighbors(self, id: usize) -> Vec<usize> {
        let p = self.coordinates(id);
        let mut result = Vec::with_capacity(self.maximum_degree());
        for dz in -1isize..=1 {
            for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    let length = dx.abs() + dy.abs() + dz.abs();
                    if length == 0 || (self == Self::Cubic6 && length != 1) {
                        continue;
                    }
                    let Some(x) = p[0].checked_add_signed(dx) else {
                        continue;
                    };
                    let Some(y) = p[1].checked_add_signed(dy) else {
                        continue;
                    };
                    let Some(z) = p[2].checked_add_signed(dz) else {
                        continue;
                    };
                    if let Some(j) = self.id([x, y, z]) {
                        result.push(j);
                    }
                }
            }
        }
        result.sort_unstable();
        result
    }
    /// Exact shortest external-graph hop distance.
    ///
    /// # Panics
    /// Panics for an out-of-bounds endpoint.
    #[must_use]
    pub fn hops(self, a: usize, b: usize) -> usize {
        let (a, b) = (self.coordinates(a), self.coordinates(b));
        let d = std::array::from_fn::<_, 3, _>(|i| a[i].abs_diff(b[i]));
        if self == Self::Cubic6 {
            d.iter().sum()
        } else {
            *d.iter().max().unwrap_or(&0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhaustive_mapping_degrees_and_reciprocal_open_edges() {
        for kind in [Topology::Cubic6, Topology::Cubic26, Topology::Grid2D8] {
            for id in 0..POPULATION {
                assert_eq!(kind.id(kind.coordinates(id)), Some(id));
                let links = kind.neighbors(id);
                assert!(!links.contains(&id));
                assert!(links.windows(2).all(|p| p[0] < p[1]));
                for j in links {
                    assert!(kind.neighbors(j).contains(&id));
                    assert_eq!(kind.hops(id, j), 1);
                }
            }
        }
        assert_eq!(Topology::Cubic6.neighbors(0).len(), 3);
        assert_eq!(Topology::Cubic26.neighbors(0).len(), 7);
        for (position, face, full) in [([8, 8, 8], 6, 26), ([0, 8, 8], 5, 17), ([0, 0, 8], 4, 11)] {
            assert_eq!(
                Topology::Cubic6
                    .neighbors(Topology::Cubic6.id(position).unwrap())
                    .len(),
                face
            );
            assert_eq!(
                Topology::Cubic26
                    .neighbors(Topology::Cubic26.id(position).unwrap())
                    .len(),
                full
            );
        }
        assert_eq!(Topology::Grid2D8.neighbors(0), vec![1, 64, 65]);
    }
}
