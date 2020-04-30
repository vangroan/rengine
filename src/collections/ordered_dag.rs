//!
//! # Note
//!
//! Currently we rely on "unstable" slotmap so nodes don't have to be copyable.
use slotmap::SlotMap;
use std::cmp::Ord;
use std::collections::HashMap;
use std::error;
use std::fmt;

/// Directed acyclic graph, where node children are kept sorted.
pub struct OrderedDag<N, E: Ord> {
    nodes: SlotMap<NodeId, Node<N, Edge<E>>>,
}

impl<N, E> OrderedDag<N, E>
where
    E: Ord + Default,
{
    pub fn new() -> Self {
        OrderedDag {
            nodes: SlotMap::with_key(),
        }
    }

    /// Inserts a new node into the graph.
    ///
    /// The node will not be connected to any other node. No cycle check
    /// will occur.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    ///
    /// let mut graph: OrderedDag<i64, i64> = OrderedDag::new();
    ///
    /// let node_1_id = graph.insert(1);
    /// let node_2_id = graph.insert(2);
    /// let node_3_id = graph.insert(3);
    /// ```
    pub fn insert(&mut self, node_value: N) -> NodeId {
        self.nodes.insert(Node {
            value: node_value,
            edges: vec![],
        })
    }

    /// Inserts a new node as a child of the given parent..
    ///
    /// If the parent node is not specified, the node will implicitly be
    /// another root.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    ///
    /// let mut graph: OrderedDag<i64, i64> = OrderedDag::new();
    ///
    /// let node_1_id = graph.insert(1);
    /// let node_2_id = graph.insert(2);
    /// let node_3_id = graph.insert(3);
    /// ```
    pub fn insert_at(&mut self, node_value: N, parent_id: Option<NodeId>) -> NodeId {
        let node_id = self.nodes.insert(Node {
            value: node_value,
            edges: vec![],
        });

        if let Some(pid) = parent_id {
            // Won't return error because no outgoing edges exist yet.
            self.set_edge(pid, node_id, E::default()).unwrap();
        }

        node_id
    }

    /// Add an edge between two nodes.
    ///
    /// Does nothing if an edge already exists.
    pub fn add_edge(&mut self, _source_id: NodeId, _target_id: NodeId, _edge_value: E) {
        unimplemented!();
    }

    /// Add or update an edge netween two nodes.
    ///
    /// # Errors
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::{OrderedDag, OrderedGraphError};
    ///
    /// let mut graph: OrderedDag<i64, i64> = OrderedDag::new();
    ///
    /// let node_1 = graph.insert(1);
    /// let node_2 = graph.insert(2);
    /// let result = graph.set_edge(node_1, node_2, 0);
    /// assert_eq!(result, Ok(()));
    /// assert_eq!(graph.out_edge_count(node_1), Some(1));
    ///
    /// // Set edge fails when a cycle is detected.
    /// let result = graph.set_edge(node_2, node_1, 0);
    /// assert_eq!(result, Err(OrderedGraphError::Cycle));
    /// ```
    pub fn set_edge(
        &mut self,
        source_id: NodeId,
        target_id: NodeId,
        edge_value: E,
    ) -> Result<(), OrderedGraphError> {
        let index: usize;

        if let Some(node) = self.nodes.get_mut(source_id) {
            if let Some(idx) = node.edges.iter().position(|e| e.child == target_id) {
                // Edge exists. Replace value.
                node.edges.get_mut(idx).unwrap().value = edge_value;
                index = idx;
            } else {
                node.edges.push(Edge {
                    value: edge_value,
                    child: target_id,
                });
                index = node.edges.len() - 1;
            };
        } else {
            return Err(OrderedGraphError::NodeDoesNotExist);
        }

        if let Some(_in_node) = self.check_cycle(source_id) {
            // Cycle detected, remove newly inserted edge.
            let _ = self.nodes.get_mut(source_id).unwrap().edges.remove(index);
            Err(OrderedGraphError::Cycle)
        } else {
            Ok(())
        }
    }

    /// Borrows a reference the a node value.
    ///
    /// Returns None if it does not exist.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    ///
    /// let mut graph: OrderedDag<i64, i64> = OrderedDag::new();
    ///
    /// let node_1 = graph.insert(1);
    /// assert_eq!(graph.node(node_1), Some(&1));
    /// ```
    pub fn node(&self, node_id: NodeId) -> Option<&N> {
        self.nodes.get(node_id).map(|n| &n.value)
    }

    /// Borrows a mutable reference the a node value.
    ///
    /// Returns None if it does not exist.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    ///
    /// let mut graph: OrderedDag<i64, i64> = OrderedDag::new();
    ///
    /// let node_1 = graph.insert(1);
    /// *graph.node_mut(node_1).unwrap() = 2;
    /// assert_eq!(graph.node(node_1), Some(&2));
    /// ```
    pub fn node_mut(&mut self, node_id: NodeId) -> Option<&mut N> {
        self.nodes.get_mut(node_id).map(|n| &mut n.value)
    }

    /// The number of edges going out of the given node.
    ///
    /// Returns None if the node does not exist.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    ///
    /// let mut graph: OrderedDag<i64, i64> = OrderedDag::new();
    ///
    /// let node_1 = graph.insert(1);
    /// let node_2 = graph.insert(2);
    /// let node_3 = graph.insert(3);
    ///
    /// graph.set_edge(node_1, node_2, 0).unwrap();
    /// graph.set_edge(node_1, node_3, 0).unwrap();
    ///
    /// assert_eq!(graph.out_edge_count(node_1), Some(2));
    /// assert_eq!(graph.out_edge_count(node_2), Some(0));
    /// assert_eq!(graph.out_edge_count(node_3), Some(0));
    /// ```
    pub fn out_edge_count(&self, node_id: NodeId) -> Option<usize> {
        self.nodes.get(node_id).map(|n| n.edges.len())
    }

    /// The number of nodes in the graph.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    ///
    /// let mut graph: OrderedDag<i64, i64> = OrderedDag::new();
    ///
    /// let node_1 = graph.insert(1);
    /// let node_2 = graph.insert(2);
    /// let node_3 = graph.insert(3);
    /// assert_eq!(graph.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    fn check_cycle(&self, start_node_id: NodeId) -> Option<NodeId> {
        let mut state: HashMap<NodeId, VisitColor> = HashMap::new();

        fn dfs_visit<N, E: Ord>(
            g: &OrderedDag<N, E>,
            u: NodeId,
            s: &mut HashMap<NodeId, VisitColor>,
        ) -> Option<NodeId> {
            let child_iter = g.nodes.get(u).unwrap().edges.iter().map(|e| e.child);
            for v in child_iter {
                let color = *s.get(&v).unwrap_or(&VisitColor::White);
                if color == VisitColor::Grey {
                    // Cycle detected. Return parent node.
                    return Some(u);
                }

                if color != VisitColor::Black {
                    return None;
                }

                s.insert(v, VisitColor::Grey);
                let result = dfs_visit(g, v, s);
                if result.is_some() {
                    // Cycle found
                    return result;
                }
            }

            let c = s.entry(u).or_insert(VisitColor::Black);
            *c = VisitColor::Black;

            return None;
        }

        state.insert(start_node_id, VisitColor::Grey);
        return dfs_visit(self, start_node_id, &mut state);
    }
}

impl<N, E> Default for OrderedDag<N, E>
where
    E: Ord + Default,
{
    fn default() -> Self {
        OrderedDag::new()
    }
}

/// Wrapper for node data.
struct Node<N, E: Ord> {
    value: N,
    edges: Vec<E>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge<E: Ord> {
    value: E,
    child: NodeId,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OrderedGraphError {
    /// The graph has been changed in a way that introduced a cycle.
    Cycle,

    /// Attempt to create edge between two nodes, at least one didn't exist.
    NodeDoesNotExist,
}

impl error::Error for OrderedGraphError {
    fn description(&self) -> &str {
        match self {
            OrderedGraphError::Cycle => "graph detected cycle between nodes",
            OrderedGraphError::NodeDoesNotExist => {
                "attempt to create edge between nodes that don't exist"
            }
        }
    }
}

impl fmt::Display for OrderedGraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderedGraphError::Cycle => write!(f, "Ordered graph cycle error"),
            OrderedGraphError::NodeDoesNotExist => {
                write!(f, "Ordered graph node does not exist error")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitColor {
    White,
    Grey,
    Black,
}

new_key_type! { pub struct NodeId; }
