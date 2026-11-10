//! Proofgoal subproof [`Subcontext`](crate::context::Subcontext) structure.

/// [`Subcontext`](crate::context::Subcontext) for proofgoal subproofs.
#[derive(Debug, Default)]
pub struct ProofgoalContext {
    /// Start constraint ID of the subproof.
    pub subproof_start_id: usize,
}

impl ProofgoalContext {
    /// Constructs a new proofgoal context.
    #[inline]
    pub fn new(start_id: usize) -> Self {
        ProofgoalContext {
            subproof_start_id: start_id,
        }
    }
}
