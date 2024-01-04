#[derive(Debug, Copy, Clone)]
pub enum TopologicalError {
    UnknownError,
    MissingTask,
    MissingGroup,
}

pub type Result<T> = std::result::Result<T, TopologicalError>;
