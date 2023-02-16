use crate::graph::{EdgeIndex, VertexIndex};

#[derive(Debug)]
pub enum GraphError {
    NotConnected,
    VertexDoesNotExist(VertexIndex),
    EdgeDoesNotExist(EdgeIndex),
    DiffAlreadyApplied,
}
