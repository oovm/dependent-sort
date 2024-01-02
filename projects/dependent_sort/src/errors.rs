#[derive(Debug, Copy, Clone)]
pub enum TopologicalError {
    UnknownError
}

pub type Result<T> = std::result::Result<T, TopologicalError>;
