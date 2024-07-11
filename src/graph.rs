use core::{fmt::Debug, mem, ops};

use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use snafu::OptionExt;

use crate::{
    errors::GraphError,
    gen_vec::{Element, GenVec, Index},
    graph_diff::{
        AddEdge, AddVertex, GraphDiff, RemoveEdge, RemoveVertex, UpdateEdgeData, UpdateVertexData,
    },
    EdgeDoesNotExistSnafu, VertexDoesNotExistSnafu,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VertexIndex(pub(crate) Index);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EdgeIndex(pub(crate) Index);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "js_names", serde(rename_all = "camelCase"))]
pub struct Vertex<T> {
    connections_from: Vec<(VertexIndex, EdgeIndex)>,
    connections_to: Vec<(VertexIndex, EdgeIndex)>,
    data: T,
}

impl<T> Vertex<T> {
    fn new(data: T) -> Vertex<T> {
        Vertex {
            connections_from: Vec::new(),
            connections_to: Vec::new(),
            data,
        }
    }

    pub fn get_connections_from(&self) -> &Vec<(VertexIndex, EdgeIndex)> {
        &self.connections_from
    }

    pub fn get_connections_to(&self) -> &Vec<(VertexIndex, EdgeIndex)> {
        &self.connections_to
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    fn add_from_unchecked(&mut self, from: VertexIndex, edge: EdgeIndex) {
        self.connections_from.push((from, edge));
    }

    fn add_to_unchecked(&mut self, to: VertexIndex, edge: EdgeIndex) {
        self.connections_to.push((to, edge));
    }

    fn remove_from(&mut self, edge_index: EdgeIndex) -> Result<(), ()> {
        let position = self
            .connections_from
            .iter()
            .position(|connection| connection.1 == edge_index)
            .ok_or(())?;

        self.connections_from.remove(position);

        Ok(())
    }

    fn remove_to(&mut self, edge_index: EdgeIndex) -> Result<(), ()> {
        let position = self
            .connections_to
            .iter()
            .position(|connection| connection.1 == edge_index)
            .ok_or(())?;

        self.connections_to.remove(position);

        Ok(())
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "js_names", serde(rename_all = "camelCase"))]
pub struct Edge<T> {
    from: VertexIndex,
    to: VertexIndex,
    data: T,
}

impl<T> Edge<T> {
    pub fn new(from: VertexIndex, to: VertexIndex, data: T) -> Edge<T> {
        Edge { from, to, data }
    }

    pub fn get_from(&self) -> VertexIndex {
        self.from
    }

    pub fn get_to(&self) -> VertexIndex {
        self.to
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

/// Main graph structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "js_names", serde(rename_all = "camelCase"))]
pub struct Graph<V, E> {
    verticies: GenVec<Vertex<V>>,
    edges: GenVec<Edge<E>>,
}

impl<V, E> Graph<V, E> {
    pub fn from_constraints() -> Graph<V, E> {
        Graph {
            verticies: GenVec::new(),
            edges: GenVec::new(),
        }
    }
}

impl<V: Clone, E: Clone> Graph<V, E> {
    pub fn new() -> Graph<V, E> {
        Graph {
            verticies: GenVec::new(),
            edges: GenVec::new(),
        }
    }

    pub fn add_vertex(&mut self, vertex_data: V) -> (VertexIndex, GraphDiff<V, E>) {
        let vertex_index = VertexIndex(self.verticies.add(Vertex::new(vertex_data.clone())));

        let diff = AddVertex {
            vertex_index,
            vertex_data,
        };

        (vertex_index, GraphDiff::AddVertex(diff))
    }

    pub fn add_edge(
        &mut self,
        from_index: VertexIndex,
        to_index: VertexIndex,
        edge_data: E,
    ) -> Result<(EdgeIndex, GraphDiff<V, E>), GraphError> {
        self.assert_vertex_exists(from_index)?;
        self.assert_vertex_exists(to_index)?;

        // create the edge and link everything up
        let edge_index = EdgeIndex(self.edges.add(Edge::new(
            from_index,
            to_index,
            edge_data.clone(),
        )));

        // connect the verticies (all vertex lookups use unwraps here to preserve the invariant of two-way connections)
        self[from_index].add_to_unchecked(to_index, edge_index);
        self[to_index].add_from_unchecked(from_index, edge_index);

        let diff = AddEdge {
            edge_index: edge_index,
            from: from_index,
            to: to_index,
            edge_data,
        };

        Ok((edge_index, GraphDiff::AddEdge(diff)))
    }

    pub fn update_vertex(
        &mut self,
        index: VertexIndex,
        value: V,
    ) -> Result<(V, GraphDiff<V, E>), GraphError> {
        let vertex = self
            .get_vertex_mut(index)
            .context(VertexDoesNotExistSnafu { index })?;

        let old_value = mem::replace(&mut vertex.data, value.clone());

        Ok((
            old_value.clone(),
            GraphDiff::UpdateVertexData(UpdateVertexData {
                index,
                before: old_value,
                after: value,
            }),
        ))
    }

    pub fn update_edge(
        &mut self,
        index: EdgeIndex,
        value: E,
    ) -> Result<(E, GraphDiff<V, E>), GraphError> {
        let edge = self
            .get_edge_mut(index)
            .context(EdgeDoesNotExistSnafu { index })?;

        let old_value = mem::replace(&mut edge.data, value.clone());

        Ok((
            old_value.clone(),
            GraphDiff::UpdateEdgeData(UpdateEdgeData {
                index,
                before: old_value,
                after: value,
            }),
        ))
    }

    pub fn remove_edge(
        &mut self,
        edge_index: EdgeIndex,
    ) -> Result<(E, GraphDiff<V, E>), GraphError> {
        self.remove_edge_internal(edge_index)
            .map(|(edge, diff)| (edge, GraphDiff::RemoveEdge(diff)))
    }

    fn remove_edge_internal(
        &mut self,
        edge_index: EdgeIndex,
    ) -> Result<(E, RemoveEdge<E>), GraphError> {
        let edge = self
            .get_edge(edge_index)
            .with_context(|| EdgeDoesNotExistSnafu { index: edge_index })?;

        let from_index = edge.from;
        let to_index = edge.to;

        // remove the edge (all vertex lookups use unwraps here to preserve the invariant of two-way connections)
        self[from_index].remove_to(edge_index).unwrap();
        self[to_index].remove_from(edge_index).unwrap();

        let edge = self.edges.remove(edge_index.0).unwrap();
        let edge_data = edge.data.clone();

        let diff = RemoveEdge {
            edge_index,
            edge: edge,
        };

        Ok((edge_data, diff))
    }

    pub fn remove_vertex(
        &mut self,
        vertex_index: VertexIndex,
    ) -> Result<(V, GraphDiff<V, E>), GraphError> {
        // check that everything is in proper order
        let vertex = self
            .get_vertex(vertex_index)
            .with_context(|| VertexDoesNotExistSnafu {
                index: vertex_index,
            })?;

        // remove all connections to the vertex
        let mut connections = vertex.get_connections_from().clone();
        connections.extend(vertex.get_connections_to().clone());

        let edge_diffs: Vec<RemoveEdge<E>> = connections
            .iter()
            .map(|(_, connection_index)| self.remove_edge_internal(*connection_index).unwrap().1)
            .collect();

        // finally remove the vertex
        let vertex = self.verticies.remove(vertex_index.0).unwrap();
        let vertex_data = vertex.data.clone();

        let diff = GraphDiff::RemoveVertex(RemoveVertex {
            vertex_index,
            vertex,
            removed_edges: edge_diffs,
        });

        Ok((vertex_data, diff))
    }

    pub fn apply_diff(&mut self, diff: GraphDiff<V, E>) -> Result<(), GraphError> {
        match diff {
            GraphDiff::AddVertex(diff) => self.apply_add_vertex_diff(diff),
            GraphDiff::AddEdge(diff) => self.apply_add_edge_diff(diff),
            GraphDiff::UpdateVertexData(diff) => self.apply_update_vertex_diff(diff),
            GraphDiff::UpdateEdgeData(diff) => self.apply_update_edge_diff(diff),
            GraphDiff::RemoveEdge(diff) => self.apply_remove_edge_diff(diff),
            GraphDiff::RemoveVertex(diff) => self.apply_remove_vertex_diff(diff),
        }
    }

    pub fn rollback_diff(&mut self, diff: GraphDiff<V, E>) -> Result<(), GraphError> {
        match diff {
            GraphDiff::AddVertex(add_vertex) => self.rollback_add_vertex_diff(add_vertex),
            GraphDiff::AddEdge(add_edge) => self.rollback_add_edge_diff(add_edge),
            GraphDiff::UpdateVertexData(diff) => self.rollback_update_vertex_diff(diff),
            GraphDiff::UpdateEdgeData(diff) => self.rollback_update_edge_diff(diff),
            GraphDiff::RemoveEdge(remove_edge) => self.rollback_remove_edge_diff(remove_edge),
            GraphDiff::RemoveVertex(remove_vertex) => {
                self.rollback_remove_vertex_diff(remove_vertex)
            }
        }
    }
}

// Utility functions
impl<V: Clone, E: Clone> Graph<V, E> {
    pub fn get_vertex(&self, index: VertexIndex) -> Option<&Vertex<V>> {
        self.verticies.get(index.0)
    }

    pub fn get_vertex_data(&self, index: VertexIndex) -> Option<&V> {
        Some(&self.get_vertex(index)?.data)
    }

    fn get_vertex_mut(&mut self, index: VertexIndex) -> Option<&mut Vertex<V>> {
        self.verticies.get_mut(index.0)
    }

    /// Note: changes here will *not* be tracked. It's usually better to use
    /// [`Graph::update_vertex`]
    pub fn get_vertex_data_mut(&mut self, index: VertexIndex) -> Option<&mut V> {
        Some(&mut self.get_vertex_mut(index)?.data)
    }

    pub fn get_edge(&self, index: EdgeIndex) -> Option<&Edge<E>> {
        self.edges.get(index.0)
    }

    pub fn get_edge_data(&self, index: EdgeIndex) -> Option<&E> {
        Some(&self.get_edge(index)?.data)
    }

    /// Note: changes here will *not* be tracked. It's usually better to use
    /// [`Graph::update_edge`]
    pub fn get_edge_data_mut(&mut self, index: EdgeIndex) -> Option<&mut E> {
        Some(&mut self.get_edge_mut(index)?.data)
    }

    fn get_edge_mut(&mut self, index: EdgeIndex) -> Option<&mut Edge<E>> {
        self.edges.get_mut(index.0)
    }

    /// Check that a vertex exists. Returns a result containing `VertexDoesNotExist` if it doesn't exist
    pub fn assert_vertex_exists(&self, index: VertexIndex) -> Result<(), GraphError> {
        if self.verticies.get(index.0).is_none() {
            Err(GraphError::VertexDoesNotExist { index })
        } else {
            Ok(())
        }
    }

    // Check that an edge exists. Returns a result containing `EdgeDoesNotExist` if it doesn't exist
    pub fn assert_edge_exists(&self, index: EdgeIndex) -> Result<(), GraphError> {
        if self.edges.get(index.0).is_none() {
            Err(GraphError::EdgeDoesNotExist { index })
        } else {
            Ok(())
        }
    }

    /// Get a list of edges between two nodes. This only returns connections in one direction.
    pub fn shared_edges(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<impl Iterator<Item = EdgeIndex> + '_, GraphError> {
        Ok(self
            .get_vertex(from)
            .with_context(|| VertexDoesNotExistSnafu { index: from })?
            .get_connections_to()
            .iter()
            .filter(move |connection| connection.0 == to)
            .map(|connection| connection.1))
    }

    pub fn get_verticies(&self) -> &GenVec<Vertex<V>> {
        &self.verticies
    }

    pub fn get_edges(&self) -> &GenVec<Edge<E>> {
        &self.edges
    }

    pub fn vertex_indexes(&self) -> impl Iterator<Item = VertexIndex> + '_ {
        self.verticies.indexes().map(|index| VertexIndex(index))
    }

    pub fn edge_indexes(&self) -> impl Iterator<Item = EdgeIndex> + '_ {
        self.edges.indexes().map(|index| EdgeIndex(index))
    }

    pub fn vertex_iter(&self) -> impl Iterator<Item = (VertexIndex, &Vertex<V>)> + '_ {
        self.verticies
            .iter()
            .map(|(index, vertex)| (VertexIndex(index), vertex))
    }

    pub fn edge_iter(&self) -> impl Iterator<Item = (EdgeIndex, &Edge<E>)> + '_ {
        self.edges
            .iter()
            .map(|(index, edge)| (EdgeIndex(index), edge))
    }

    pub fn vertex_data_iter(&self) -> impl Iterator<Item = (VertexIndex, &V)> + '_ {
        self.verticies
            .iter()
            .map(|(index, vertex)| (VertexIndex(index), &vertex.data))
    }

    pub fn vertex_data_iter_mut(&mut self) -> impl Iterator<Item = (VertexIndex, &mut V)> + '_ {
        self.verticies
            .iter_mut()
            .map(|(index, vertex)| (VertexIndex(index), &mut vertex.data))
    }

    pub fn edge_data_iter(&self) -> impl Iterator<Item = (EdgeIndex, &E)> + '_ {
        self.edges
            .iter()
            .map(|(index, edge)| (EdgeIndex(index), &edge.data))
    }

    pub fn edge_data_iter_mut(&mut self) -> impl Iterator<Item = (EdgeIndex, &mut E)> + '_ {
        self.edges
            .iter_mut()
            .map(|(index, edge)| (EdgeIndex(index), &mut edge.data))
    }

    fn apply_add_vertex_diff(&mut self, diff: AddVertex<V>) -> Result<(), GraphError> {
        if !self
            .verticies
            .is_replaceable_by_index_apply(diff.vertex_index.0)
        {
            return Err(GraphError::InvalidDiff);
        }

        self.verticies.vec[diff.vertex_index.0.index] = Element::Occupied {
            value: Vertex::new(diff.vertex_data),
            generation: diff.vertex_index.0.generation,
        };

        Ok(())
    }

    fn apply_add_edge_diff(&mut self, diff: AddEdge<E>) -> Result<(), GraphError> {
        self.assert_vertex_exists(diff.from)?;
        self.assert_vertex_exists(diff.to)?;

        // check that this edge doesn't exist
        if !self.edges.is_replaceable_by_index_apply(diff.edge_index.0) {
            return Err(GraphError::InvalidDiff);
        }

        // apply the diff
        self.edges.vec[diff.edge_index.0.index] = Element::Occupied {
            value: Edge::new(diff.from, diff.to, diff.edge_data),
            generation: diff.edge_index.0.generation,
        };

        let from = self
            .get_vertex_mut(diff.from)
            .expect("Graph state has become corrupted before applying diff");
        from.add_to_unchecked(diff.to, diff.edge_index);

        let to = self
            .get_vertex_mut(diff.to)
            .expect("Graph state has become corrupted before applying diff");
        to.add_from_unchecked(diff.from, diff.edge_index);

        Ok(())
    }

    fn apply_update_vertex_diff(&mut self, diff: UpdateVertexData<V>) -> Result<(), GraphError> {
        let vertex = self
            .get_vertex_mut(diff.index)
            .context(VertexDoesNotExistSnafu { index: diff.index })?;

        vertex.data = diff.after;

        Ok(())
    }

    fn apply_update_edge_diff(&mut self, diff: UpdateEdgeData<E>) -> Result<(), GraphError> {
        let edge = self
            .get_edge_mut(diff.index)
            .context(EdgeDoesNotExistSnafu { index: diff.index })?;

        edge.data = diff.after;

        Ok(())
    }

    fn apply_remove_edge_diff(&mut self, diff: RemoveEdge<E>) -> Result<(), GraphError> {
        self.assert_vertex_exists(diff.edge.from)?;
        self.assert_vertex_exists(diff.edge.to)?;
        self.assert_edge_exists(diff.edge_index)?;

        // remove the edge
        self.remove_edge(diff.edge_index)
            .expect("Graph state has become corrupted before applying diff");

        Ok(())
    }

    fn apply_remove_vertex_diff(&mut self, diff: RemoveVertex<V, E>) -> Result<(), GraphError> {
        self.assert_vertex_exists(diff.vertex_index)?;

        self.remove_vertex(diff.vertex_index)
            .expect("Graph state has become corrupted before applying diff");

        Ok(())
    }

    fn rollback_add_vertex_diff(&mut self, diff: AddVertex<V>) -> Result<(), GraphError> {
        // check that the vertex exists
        self.assert_vertex_exists(diff.vertex_index)?;

        self.remove_vertex_and_reset(diff.vertex_index)
            .expect("Graph state has become corrupted before applying diff");

        Ok(())
    }

    fn rollback_add_edge_diff(&mut self, diff: AddEdge<E>) -> Result<(), GraphError> {
        self.assert_vertex_exists(diff.from)?;
        self.assert_vertex_exists(diff.to)?;
        self.assert_edge_exists(diff.edge_index)?;

        // remove the edge
        self.remove_edge_and_reset(diff.edge_index)
            .expect("Graph state has become corrupted before applying diff");

        Ok(())
    }

    fn rollback_update_vertex_diff(&mut self, diff: UpdateVertexData<V>) -> Result<(), GraphError> {
        let vertex = self
            .get_vertex_mut(diff.index)
            .context(VertexDoesNotExistSnafu { index: diff.index })?;

        vertex.data = diff.before;

        Ok(())
    }

    fn rollback_update_edge_diff(&mut self, diff: UpdateEdgeData<E>) -> Result<(), GraphError> {
        let edge = self
            .get_edge_mut(diff.index)
            .context(EdgeDoesNotExistSnafu { index: diff.index })?;

        edge.data = diff.before;

        Ok(())
    }

    fn rollback_remove_edge_diff(&mut self, diff: RemoveEdge<E>) -> Result<(), GraphError> {
        let from_index = diff.edge.from;
        let to_index = diff.edge.to;

        self.assert_vertex_exists(from_index)?;
        self.assert_vertex_exists(to_index)?;

        // check that this edge doesn't exist
        if !self
            .edges
            .is_replaceable_by_index_rollback(diff.edge_index.0)
        {
            return Err(GraphError::InvalidDiff);
        }

        // apply the diff
        self.edges.vec[diff.edge_index.0.index] = Element::Occupied {
            value: diff.edge,
            generation: diff.edge_index.0.generation,
        };

        let from = self
            .get_vertex_mut(from_index)
            .expect("Graph state has become corrupted before applying diff");
        from.add_to_unchecked(to_index, diff.edge_index);

        let to = self
            .get_vertex_mut(to_index)
            .expect("Graph state has become corrupted before applying diff");
        to.add_from_unchecked(from_index, diff.edge_index);

        Ok(())
    }

    fn rollback_remove_vertex_diff(&mut self, diff: RemoveVertex<V, E>) -> Result<(), GraphError> {
        if !self
            .verticies
            .is_replaceable_by_index_rollback(diff.vertex_index.0)
        {
            return Err(GraphError::InvalidDiff);
        }

        // check that all edges are replaceable
        for removed_edge in diff.removed_edges.iter() {
            if !self
                .edges
                .is_replaceable_by_index_rollback(removed_edge.edge_index.0)
            {
                return Err(GraphError::InvalidDiff);
            }
        }

        self.verticies.vec[diff.vertex_index.0.index] = Element::Occupied {
            value: diff.vertex,
            generation: diff.vertex_index.0.generation,
        };

        for removed_edge in diff.removed_edges {
            let from = self
                .get_vertex_mut(removed_edge.edge.from)
                .expect("Graph state has become corrupted before applying diff");
            from.add_to_unchecked(removed_edge.edge.to, removed_edge.edge_index);

            let to = self
                .get_vertex_mut(removed_edge.edge.to)
                .expect("Graph state has become corrupted before applying diff");
            to.add_from_unchecked(removed_edge.edge.from, removed_edge.edge_index);

            self.edges.vec[removed_edge.edge_index.0.index] = Element::Occupied {
                value: removed_edge.edge,
                generation: removed_edge.edge_index.0.generation,
            };
        }

        Ok(())
    }

    fn remove_vertex_and_reset(&mut self, index: VertexIndex) -> Result<V, GraphError> {
        // check that everything is in proper order
        let vertex = self
            .get_vertex(index)
            .with_context(|| VertexDoesNotExistSnafu { index })?;

        // remove all connections to the vertex
        let mut connections = vertex.get_connections_from().clone();
        connections.extend(vertex.get_connections_to().clone());

        for (_, edge_index) in connections {
            self.remove_edge_and_reset(edge_index).unwrap();
        }

        // finally remove the vertex
        let vertex = self
            .verticies
            .remove_but_maintain_generation(index.0)
            .unwrap();
        let vertex_data = vertex.data.clone();

        Ok(vertex_data)
    }

    fn remove_edge_and_reset(&mut self, edge_index: EdgeIndex) -> Result<E, GraphError> {
        let edge = self
            .get_edge(edge_index)
            .with_context(|| EdgeDoesNotExistSnafu { index: edge_index })?;
        let from_index = edge.from;
        let to_index = edge.to;

        // remove the edge (all vertex lookups use unwraps here to preserve the invariant of two-way connections)
        let from = self.get_vertex_mut(from_index).unwrap();
        from.remove_to(edge_index).unwrap();

        let to = self.get_vertex_mut(to_index).unwrap();
        to.remove_from(edge_index).unwrap();

        let edge = self
            .edges
            .remove_but_maintain_generation(edge_index.0)
            .unwrap();

        Ok(edge.data)
    }
}

impl<V: Clone, E: Clone> Default for Graph<V, E> {
    fn default() -> Self {
        Graph::new()
    }
}

impl<V: Clone, E: Clone> ops::Index<VertexIndex> for Graph<V, E> {
    type Output = Vertex<V>;

    fn index(&self, index: VertexIndex) -> &Self::Output {
        self.get_vertex(index).unwrap()
    }
}

impl<V: Clone, E: Clone> ops::IndexMut<VertexIndex> for Graph<V, E> {
    fn index_mut(&mut self, index: VertexIndex) -> &mut Self::Output {
        self.get_vertex_mut(index).unwrap()
    }
}

impl<V: Clone, E: Clone> ops::Index<EdgeIndex> for Graph<V, E> {
    type Output = Edge<E>;

    fn index(&self, index: EdgeIndex) -> &Self::Output {
        self.get_edge(index).unwrap()
    }
}

impl<V: Clone, E: Clone> ops::IndexMut<EdgeIndex> for Graph<V, E> {
    fn index_mut(&mut self, index: EdgeIndex) -> &mut Self::Output {
        self.get_edge_mut(index).unwrap()
    }
}
