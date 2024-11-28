
/// An enum that represents a failure in graph or automaton traversal.
pub enum Fail {
    /// The callee is unable to accept or reject the input.
    /// Caller should propagate this error.
    Incomplete,

    /// The callee rejected the input and mark it as recoverable.
    /// Caller should try the next candidate.
    Backtrack,

    /// The callee rejected the input and mark it as unrecoverable.
    /// Caller should propagate this error.
    Cut,
}

