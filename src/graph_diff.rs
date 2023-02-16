use alloc::vec::Vec;

use crate::graph::{Edge, EdgeIndex, Vertex, VertexIndex};

#[derive(Debug, Clone)]
pub struct AddVertex<V> {
    pub(crate) vertex_index: VertexIndex,
    pub(crate) vertex_data: V,
}

#[derive(Debug, Clone)]
pub struct AddEdge<E> {
    pub(crate) edge_index: EdgeIndex,
    pub(crate) from: VertexIndex,
    pub(crate) to: VertexIndex,
    pub(crate) edge_data: E,
}

#[derive(Debug, Clone)]
pub struct RemoveEdge<E> {
    pub(crate) edge_index: EdgeIndex,
    pub(crate) edge: Edge<E>,
}

#[derive(Debug, Clone)]
pub struct RemoveVertex<V, E> {
    pub(crate) vertex_index: VertexIndex,
    pub(crate) vertex: Vertex<V>,
    pub(crate) removed_edges: Vec<RemoveEdge<E>>,
}

#[derive(Debug, Clone)]
pub enum GraphDiff<V, E> {
    AddVertex(AddVertex<V>),
    AddEdge(AddEdge<E>),
    RemoveEdge(RemoveEdge<E>),
    RemoveVertex(RemoveVertex<V, E>),
}
