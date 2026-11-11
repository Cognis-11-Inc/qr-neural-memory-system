//! Initial binary state representation; no learning or propagation rule yet.

/// Width and height of the initial square substrate.
pub const SIDE: usize = 64;
/// Number of computational cells in the initial substrate.
pub const CELL_COUNT: usize = SIDE * SIDE;

/// A binary matrix in row-major order, indexed as `cells[row][column]`.
pub type Matrix = [[bool; SIDE]; SIDE];

/// Construct a matrix with all 4,096 cells inactive.
#[must_use]
pub const fn empty_matrix() -> Matrix {
    [[false; SIDE]; SIDE]
}

#[cfg(test)]
mod tests {
    use super::{CELL_COUNT, empty_matrix};

    #[test]
    fn initial_state_has_4096_inactive_independent_cells() {
        let mut matrix = empty_matrix();
        assert_eq!(matrix.iter().flatten().count(), 4096);
        assert_eq!(CELL_COUNT, 4096);
        assert!(matrix.iter().flatten().all(|cell| !cell));
        matrix[0][0] = true;
        matrix[63][63] = true;
        assert_eq!(matrix.iter().flatten().filter(|&&cell| cell).count(), 2);
        assert!(!empty_matrix()[0][0]);
    }
}
