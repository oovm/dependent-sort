/// Error type for topological sort
#[derive(Debug, Copy, Clone)]
pub enum TopologicalError<'i, T, G> {
    /// A task is missing from the list of tasks.
    MissingTask {
        /// Reference of the missing task.
        task: &'i T,
    },
    /// A group is missing from the list of groups.
    MissingGroup {
        /// Reference of the missing group.
        group: &'i G,
    },
}
