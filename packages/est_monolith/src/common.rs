/// An enum that represents a failure in graph or automaton traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fail {
    /// The callee is unable to accept or reject the input.
    /// Caller should propagate this error.
    Incomplete,
    // /// The callee rejected the input and mark it as recoverable.
    // /// Caller should try the next candidate.
    // Backtrack,

    // /// The callee rejected the input and mark it as unrecoverable.
    // /// Caller should propagate this error.
    // Cut,
}

pub type Result<T> = std::result::Result<T, Fail>;
