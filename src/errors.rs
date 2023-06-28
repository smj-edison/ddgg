use snafu::Snafu;

use crate::graph::{EdgeIndex, VertexIndex};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum GraphError {
    #[snafu(display("Vertex `{index:?}` does not exist"))]
    VertexDoesNotExist { index: VertexIndex },
    #[snafu(display("Edge `{index:?}` does not exist"))]
    EdgeDoesNotExist { index: EdgeIndex },
    #[snafu(display("Invalid diff"))]
    InvalidDiff,
}
