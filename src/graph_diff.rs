use alloc::vec::Vec;

use crate::graph::{Edge, EdgeIndex, Vertex, VertexIndex};

#[derive(Debug, Clone)]
pub struct AddVertex<V> {
    pub(crate) vertex_index: VertexIndex,
    pub(crate) vertex_data: V,
}

impl<V> AddVertex<V> {
    pub fn get_vertex_index(&self) -> VertexIndex {
        self.vertex_index
    }

    pub fn get_vertex_data(&self) -> &V {
        &self.vertex_data
    }
}

#[derive(Debug, Clone)]
pub struct AddEdge<E> {
    pub(crate) edge_index: EdgeIndex,
    pub(crate) from: VertexIndex,
    pub(crate) to: VertexIndex,
    pub(crate) edge_data: E,
}

impl<E> AddEdge<E> {
    pub fn get_edge_index(&self) -> EdgeIndex {
        self.edge_index
    }

    pub fn get_from(&self) -> VertexIndex {
        self.from
    }

    pub fn get_to(&self) -> VertexIndex {
        self.to
    }

    pub fn get_edge_data(&self) -> &E {
        &self.edge_data
    }
}

#[derive(Debug, Clone)]
pub struct RemoveEdge<E> {
    pub(crate) edge_index: EdgeIndex,
    pub(crate) edge: Edge<E>,
}

impl<E> RemoveEdge<E> {
    pub fn get_edge_index(&self) -> EdgeIndex {
        self.edge_index
    }

    pub fn get_edge(&self) -> &Edge<E> {
        &self.edge
    }
}

#[derive(Debug, Clone)]
pub struct RemoveVertex<V, E> {
    pub(crate) vertex_index: VertexIndex,
    pub(crate) vertex: Vertex<V>,
    pub(crate) removed_edges: Vec<RemoveEdge<E>>,
}

impl<V, E> RemoveVertex<V, E> {
    pub fn get_vertex_index(&self) -> VertexIndex {
        self.vertex_index
    }

    pub fn get_vertex(&self) -> &Vertex<V> {
        &self.vertex
    }

    pub fn get_removed_edges(&self) -> &Vec<RemoveEdge<E>> {
        &self.removed_edges
    }
}

#[derive(Debug, Clone)]
pub enum GraphDiff<V, E> {
    AddVertex(AddVertex<V>),
    AddEdge(AddEdge<E>),
    RemoveEdge(RemoveEdge<E>),
    RemoveVertex(RemoveVertex<V, E>),
}
