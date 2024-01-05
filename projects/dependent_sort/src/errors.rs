#[derive(Debug, Copy, Clone)]
pub enum TopologicalError<'i, T, G> {
    UnknownError,
    MissingTask { task: &'i T },
    MissingGroup { group: &'i G },
}
