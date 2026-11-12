//! Initial activation-state scaffold; geometry and neuron internals remain unspecified.

/// Number of computational cells in the initial substrate.
pub const CELL_COUNT: usize = 4096;

/// One activation bit per cell, indexed by cell ID without spatial meaning.
///
/// This scaffold does not yet represent persistent learned internal neuron state.
pub type ActivationState = [bool; CELL_COUNT];

/// Construct an activation state with all 4,096 cells inactive.
#[must_use]
pub const fn empty_activation_state() -> ActivationState {
    [false; CELL_COUNT]
}

#[cfg(test)]
mod tests {
    use super::{CELL_COUNT, empty_activation_state};

    #[test]
    fn initial_state_has_4096_inactive_independent_cells() {
        let mut state = empty_activation_state();
        assert_eq!(state.len(), 4096);
        assert_eq!(CELL_COUNT, 4096);
        assert!(state.iter().all(|cell| !cell));
        state[0] = true;
        state[CELL_COUNT - 1] = true;
        assert_eq!(state.iter().filter(|&&cell| cell).count(), 2);
        assert!(!empty_activation_state()[0]);
    }
}
