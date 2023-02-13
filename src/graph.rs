use core::{fmt::Debug, ops};

use alloc::vec::Vec;

use crate::{
    errors::GraphError,
    gen_vec::{GenVec, Index},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexIndex(Index);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeIndex(Index);

pub struct Vertex<T> {
    connections_from: Vec<(VertexIndex, EdgeIndex)>,
    connections_to: Vec<(VertexIndex, EdgeIndex)>,
    value: T,
}

impl<T> Vertex<T> {
    fn new(value: T) -> Vertex<T> {
        Vertex {
            connections_from: Vec::new(),
            connections_to: Vec::new(),
            value,
        }
    }

    pub fn get_connections_from(&self) -> &Vec<(VertexIndex, EdgeIndex)> {
        &self.connections_from
    }

    pub fn get_connections_to(&self) -> &Vec<(VertexIndex, EdgeIndex)> {
        &self.connections_to
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

pub struct Edge<T> {
    from: VertexIndex,
    to: VertexIndex,
    pub value: T,
}

impl<T> Edge<T> {
    pub fn new(from: VertexIndex, to: VertexIndex, value: T) -> Edge<T> {
        Edge { from, to, value }
    }

    pub fn get_from(&self) -> VertexIndex {
        self.from
    }

    pub fn get_to(&self) -> VertexIndex {
        self.to
    }
}

/// Main graph structure
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

impl<V, E> Graph<V, E> {
    pub fn add_vertex(&mut self, vertex: V) -> Result<VertexIndex, GraphError> {
        Ok(VertexIndex(self.verticies.add(Vertex::new(vertex))))
    }

    pub fn add_edge(
        &mut self,
        from_index: VertexIndex,
        to_index: VertexIndex,
        edge_value: E,
    ) -> Result<EdgeIndex, GraphError> {
        // check that everything is in proper order
        self.assert_vertex_exists(from_index)?;
        self.assert_vertex_exists(to_index)?;

        // create the edge and link everything up
        let edge_index = EdgeIndex(self.edges.add(Edge::new(from_index, to_index, edge_value)));

        // already checked that vertex exists, so we're unwrapping
        let from = self.get_vertex_mut(from_index).unwrap();
        from.add_to_unchecked(to_index, edge_index);

        // already checked that vertex exists, so we're unwrapping (also, we've already added
        // the "from->to" link, and now we can't add the "to->from" link. This is a panic
        // condition)
        let to = self.get_vertex_mut(to_index).unwrap();
        to.add_from_unchecked(from_index, edge_index);

        Ok(edge_index)
    }

    pub fn remove_edge(&mut self, edge_index: EdgeIndex) -> Result<Edge<E>, GraphError> {
        // check that everything is in proper order
        let edge = self.get_edge(edge_index)?;
        let from_index = edge.from;
        let to_index = edge.to;

        // remove the edge (all node lookups use unwraps here to preserve the invariant of two-way connections)
        let from = self.get_vertex_mut(from_index).unwrap();
        from.remove_to(edge_index).unwrap();

        let to = self.get_vertex_mut(to_index).unwrap();
        to.remove_from(edge_index).unwrap();

        Ok(self.edges.remove(edge_index.0).unwrap())
    }

    pub fn remove_vertex(&mut self, vertex_index: VertexIndex) -> Result<V, GraphError> {
        // check that everything is in proper order
        let vertex = self.get_vertex(vertex_index)?;

        // remove all connections to the vertex
        let mut connections = vertex.get_connections_from().clone();
        connections.extend(vertex.get_connections_to().clone());

        for (_, connection_index) in connections {
            self.remove_edge(connection_index).unwrap();
        }

        // finally remove the vertex
        Ok(self.verticies.remove(vertex_index.0).unwrap().value)
    }

    pub fn get_vertex(&self, index: VertexIndex) -> Result<&Vertex<V>, GraphError> {
        match self.verticies.get(index.0) {
            Some(vertex) => Ok(vertex),
            None => Err(GraphError::VertexDoesNotExist(index)),
        }
    }

    pub fn get_vertex_mut(&mut self, index: VertexIndex) -> Result<&mut Vertex<V>, GraphError> {
        match self.verticies.get_mut(index.0) {
            Some(vertex) => Ok(vertex),
            None => Err(GraphError::VertexDoesNotExist(index)),
        }
    }

    pub fn get_edge(&self, index: EdgeIndex) -> Result<&Edge<E>, GraphError> {
        match self.edges.get(index.0) {
            Some(edge) => Ok(edge),
            None => Err(GraphError::EdgeDoesNotExist(index)),
        }
    }

    pub fn get_edge_mut(&mut self, index: EdgeIndex) -> Result<&mut Edge<E>, GraphError> {
        match self.edges.get_mut(index.0) {
            Some(edge) => Ok(edge),
            None => Err(GraphError::EdgeDoesNotExist(index)),
        }
    }

    /// Check that a vertex exists. Returns a result containing `VertexDoesNotExist` if it doesn't exist
    pub fn assert_vertex_exists(&self, index: VertexIndex) -> Result<(), GraphError> {
        if self.verticies.get(index.0).is_none() {
            Err(GraphError::VertexDoesNotExist(index))
        } else {
            Ok(())
        }
    }

    // Check that an edge exists. Returns a result containing `EdgeDoesNotExist` if it doesn't exist
    pub fn assert_edge_exists(&self, index: EdgeIndex) -> Result<(), GraphError> {
        if self.edges.get(index.0).is_none() {
            Err(GraphError::EdgeDoesNotExist(index))
        } else {
            Ok(())
        }
    }
}

impl<V, E> ops::Index<VertexIndex> for Graph<V, E> {
    type Output = Vertex<V>;

    fn index(&self, index: VertexIndex) -> &Self::Output {
        self.get_vertex(index).unwrap()
    }
}

impl<V, E> ops::IndexMut<VertexIndex> for Graph<V, E> {
    fn index_mut(&mut self, index: VertexIndex) -> &mut Self::Output {
        self.get_vertex_mut(index).unwrap()
    }
}

impl<V, E> ops::Index<EdgeIndex> for Graph<V, E> {
    type Output = Edge<E>;

    fn index(&self, index: EdgeIndex) -> &Self::Output {
        self.get_edge(index).unwrap()
    }
}

impl<V, E> ops::IndexMut<EdgeIndex> for Graph<V, E> {
    fn index_mut(&mut self, index: EdgeIndex) -> &mut Self::Output {
        self.get_edge_mut(index).unwrap()
    }
}
