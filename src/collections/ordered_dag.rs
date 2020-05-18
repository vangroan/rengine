//!
//! # Note
//!
//! Currently we rely on "unstable" slotmap so nodes don't have to be copyable.
use slotmap::SlotMap;
use std::cmp::Ord;
use std::collections::HashMap;
use std::error;
use std::fmt::{self, Debug};
use std::iter::Iterator;
use std::marker::PhantomData;

pub mod prelude {
    pub use super::{NodeId, OrderedDag, Walker};
}

/// Directed acyclic graph, where node children are kept sorted.
pub struct OrderedDag<N, E: Ord> {
    nodes: SlotMap<NodeId, Node<N, Edge<E>>>,
}

impl<N, E> OrderedDag<N, E>
where
    E: Ord,
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
    pub fn insert_at(&mut self, node_value: N, parent_id: Option<NodeId>) -> NodeId
    where
        E: Default,
    {
        let node_id = self.nodes.insert(Node {
            value: node_value,
            edges: vec![],
        });

        if let Some(pid) = parent_id {
            // Won't return error because no outgoing edges exist yet.
            self.set_edge_unchecked(pid, node_id, E::default()).unwrap();
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
    /// Returns errors when the source node id does not exist, or when the edge
    /// would create a cycle.
    ///
    /// When a cycle is detected, the edge will not be created.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::{OrderedDag, ordered_dag::OrderedGraphError};    
    ///
    /// let mut graph: OrderedDag<i64, i64> = OrderedDag::new();
    ///
    /// let node_1 = graph.insert(1);
    /// let node_2 = graph.insert(2);
    /// let result = graph.set_edge(node_1, node_2, 0);
    /// assert_eq!(result, Ok(()));
    /// assert_eq!(graph.out_edge_len(node_1), Some(1));
    ///
    /// // Set edge fails when a cycle is detected.
    /// let result = graph.set_edge(node_2, node_1, 0);
    /// assert_eq!(result, Err(OrderedGraphError::Cycle));
    /// assert_eq!(graph.out_edge_len(node_2), Some(0));
    /// ```
    pub fn set_edge(
        &mut self,
        source_id: NodeId,
        target_id: NodeId,
        edge_value: E,
    ) -> Result<(), OrderedGraphError> {
        if let Some(index) = self.set_edge_unchecked(source_id, target_id, edge_value) {
            if let Some(_in_node) = self.check_cycle(source_id) {
                // Cycle detected, remove newly inserted edge.
                let _ = self.nodes.get_mut(source_id).unwrap().edges.remove(index);
                Err(OrderedGraphError::Cycle)
            } else {
                Ok(())
            }
        } else {
            Err(OrderedGraphError::NodeDoesNotExist)
        }
    }

    /// Add or update an edge without checking for cycles.
    ///
    /// Returns the index of the inserted edge on success, or None
    /// if the target node doesn't exist.
    fn set_edge_unchecked(
        &mut self,
        source_id: NodeId,
        target_id: NodeId,
        edge_value: E,
    ) -> Option<usize> {
        if let Some(node) = self.nodes.get_mut(source_id) {
            if let Some(idx) = node.edges.iter().position(|e| e.child == target_id) {
                // Edge exists. Replace value.
                node.edges.get_mut(idx).unwrap().value = edge_value;
                Some(idx)
            } else {
                node.edges.push(Edge {
                    value: edge_value,
                    child: target_id,
                });
                Some(node.edges.len() - 1)
            }
        } else {
            None
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
    /// assert_eq!(graph.out_edge_len(node_1), Some(2));
    /// assert_eq!(graph.out_edge_len(node_2), Some(0));
    /// assert_eq!(graph.out_edge_len(node_3), Some(0));
    /// ```
    pub fn out_edge_len(&self, node_id: NodeId) -> Option<usize> {
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

    /// Returns true if the graph is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    ///
    /// let mut graph: OrderedDag<i64, i64> = OrderedDag::new();
    ///
    /// assert!(graph.is_empty());
    /// graph.insert(1);
    /// assert!(!graph.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Sorts the edges of all nodes.
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    /// use rengine::collections::ordered_dag::{NodeId, Walker};
    ///
    /// fn make_string(g: &OrderedDag<&str, i64>, node_id: NodeId) -> String {
    ///     g.walk_pre_order(node_id)
    ///         .iter(&g)
    ///         .map(|(_node_id, node_val)| *node_val)
    ///         .collect::<Vec<&str>>()
    ///         .join("")
    /// }
    ///
    /// let mut graph: OrderedDag<&'static str, i64> = OrderedDag::new();
    ///
    /// //        a
    /// //       /|\
    /// //      / | \
    /// //     /  |  \
    /// //    23  11  7
    /// //   /    |    \
    /// //  b     c     d
    ///
    /// let node_1 = graph.insert("a");
    /// let node_2 = graph.insert("b");
    /// let node_3 = graph.insert("c");
    /// let node_4 = graph.insert("d");
    /// graph.set_edge(node_1, node_2, 23);
    /// graph.set_edge(node_1, node_3, 11);
    /// graph.set_edge(node_1, node_4, 7);
    ///
    /// graph.sort();
    /// assert_eq!(make_string(&graph, node_1), "adcb");
    ///
    /// // Move "b" to the front
    /// graph.set_edge(node_1, node_2, 6);
    /// graph.sort();
    /// assert_eq!(make_string(&graph, node_1), "abdc");
    /// ```
    pub fn sort(&mut self) {
        for (_, node) in self.nodes.iter_mut() {
            node.edges.sort();
        }
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
                let color = *s.entry(v).or_insert(VisitColor::White);
                if color == VisitColor::Grey {
                    // Cycle detected. Return parent node.
                    return Some(u);
                }

                if color == VisitColor::Black {
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

            None
        }

        state.insert(start_node_id, VisitColor::Grey);
        dfs_visit(self, start_node_id, &mut state)
    }

    /// Builds a string representation of the whole graph.
    pub fn string(&self) -> String
    where
        N: Debug,
        E: Ord + Debug,
    {
        let mut sb = String::new();

        for (node_id, node) in self.nodes.iter() {
            if node.edges.is_empty() {
                sb.push_str(&format!("{:?} ->\n", node_id));
            } else if node.edges.len() == 1 {
                let child_id = node.edges.iter().next().unwrap().child;
                sb.push_str(&format!("{:?} -> {:?}\n", node_id, child_id));
            } else {
                sb.push_str(&format!("{:?}\n", node_id));
                for edge_id in node.edges.iter().map(|e| e.child) {
                    sb.push_str(&format!("  -> {:?}\n", edge_id));
                }
            }
        }

        sb
    }
}

impl<N, E> OrderedDag<N, E>
where
    E: Ord,
{
    /// Traverse the graph depth-first in pre-order.
    ///
    /// If the graph does not contain the given node id, the walker
    /// will do nothing.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    /// use rengine::collections::ordered_dag::Walker;
    ///
    /// //       a
    /// //      / \
    /// //     /   \
    /// //    b     c
    /// //   / \
    /// //  /   \
    /// // d     e
    /// //
    /// // pre_order = a, b, d, e, c
    ///
    /// let mut graph: OrderedDag<&'static str, i64> = OrderedDag::new();
    /// let node_1 = graph.insert("a");
    /// let node_2 = graph.insert("b");
    /// let node_3 = graph.insert("c");
    /// let node_4 = graph.insert("d");
    /// let node_5 = graph.insert("e");
    /// graph.set_edge(node_1, node_2, 0);
    /// graph.set_edge(node_1, node_3, 0);
    /// graph.set_edge(node_2, node_4, 0);
    /// graph.set_edge(node_2, node_5, 0);
    ///
    /// let mut walker = graph.walk_pre_order(node_1);
    /// let mut result = String::new();
    ///
    /// while let Some(node_id) = walker.next(&graph) {
    ///     result.push_str(graph.node(node_id).unwrap());
    /// }
    ///
    /// assert_eq!(result.as_str(), "abdec");
    /// ```
    pub fn walk_pre_order(&self, start_node: NodeId) -> PreOrderWalk<N, E> {
        PreOrderWalk {
            stack: if self.nodes.contains_key(start_node) {
                vec![start_node]
            } else {
                vec![]
            },
            _marker: PhantomData,
        }
    }

    /// Traverse the graph depth-first in post-order.
    ///
    /// If the graph does not contain the given node id, the walker
    /// will do nothing.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    /// use rengine::collections::ordered_dag::Walker;
    ///
    /// //       a
    /// //      / \
    /// //     /   \
    /// //    b     c
    /// //   / \
    /// //  /   \
    /// // d     e
    /// //
    /// // post_order = d, e, b, c, a
    ///
    /// let mut graph: OrderedDag<&'static str, i64> = OrderedDag::new();
    /// let node_1 = graph.insert("a");
    /// let node_2 = graph.insert("b");
    /// let node_3 = graph.insert("c");
    /// let node_4 = graph.insert("d");
    /// let node_5 = graph.insert("e");
    /// graph.set_edge(node_1, node_2, 0);
    /// graph.set_edge(node_1, node_3, 0);
    /// graph.set_edge(node_2, node_4, 0);
    /// graph.set_edge(node_2, node_5, 0);
    ///
    /// let mut walker = graph.walk_post_order(node_1);
    /// let mut result = String::new();
    ///
    /// while let Some(node_id) = walker.next(&graph) {
    ///     result.push_str(graph.node(node_id).unwrap());
    /// }
    ///
    /// assert_eq!(result.as_str(), "debca");
    /// ```
    pub fn walk_post_order(&self, start_node: NodeId) -> PostOrderWalk<N, E> {
        PostOrderWalk {
            stack: if self.nodes.contains_key(start_node) {
                vec![start_node]
            } else {
                vec![]
            },
            out: vec![],
            _marker: PhantomData,
        }
    }

    /// Walk the immediate children of the given node.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::collections::OrderedDag;
    /// use rengine::collections::ordered_dag::Walker;
    ///
    /// //       a
    /// //      / \
    /// //     /   \
    /// //    b     c
    /// //   / \
    /// //  /   \
    /// // d     e
    ///
    /// let mut graph: OrderedDag<&'static str, i64> = OrderedDag::new();
    /// let node_1 = graph.insert("a");
    /// let node_2 = graph.insert("b");
    /// let node_3 = graph.insert("c");
    /// let node_4 = graph.insert("d");
    /// let node_5 = graph.insert("e");
    /// graph.set_edge(node_1, node_2, 0);
    /// graph.set_edge(node_1, node_3, 0);
    /// graph.set_edge(node_2, node_4, 0);
    /// graph.set_edge(node_2, node_5, 0);
    ///
    /// let mut walker = graph.walk_children(node_1);
    /// let mut result = String::new();
    ///
    /// while let Some(node_id) = walker.next(&graph) {
    ///     result.push_str(graph.node(node_id).unwrap());
    /// }
    ///
    /// assert_eq!(result.as_str(), "bc");
    /// ```
    pub fn walk_children(&self, node_id: NodeId) -> ChildrenWalk<N, E> {
        ChildrenWalk {
            node_id,
            cursor: 0,
            _marker: PhantomData,
        }
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

// -------------------- //
// Traversal Algorithms //
// -------------------- //

pub trait Walker {
    type Node;
    type Edge: Ord;

    fn next(&mut self, graph: &OrderedDag<Self::Node, Self::Edge>) -> Option<NodeId>;

    fn iter<'a>(
        self,
        graph: &'a OrderedDag<Self::Node, Self::Edge>,
    ) -> WalkerIter<'a, Self::Node, Self::Edge, Self>
    where
        Self: Sized,
    {
        WalkerIter {
            walker: self,
            graph,
        }
    }
}

pub struct WalkerIter<'a, N, E: Ord, W: Walker<Node = N, Edge = E>> {
    walker: W,
    graph: &'a OrderedDag<N, E>,
}

impl<'a, N, E, W> Iterator for WalkerIter<'a, N, E, W>
where
    E: Ord,
    W: Walker<Node = N, Edge = E>,
{
    type Item = (NodeId, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node_id) = self.walker.next(&self.graph) {
            self.graph.node(node_id).map(|node_val| (node_id, node_val))
        } else {
            None
        }
    }
}

pub struct PreOrderWalk<N, E> {
    stack: Vec<NodeId>,
    _marker: PhantomData<(N, E)>,
}

impl<N, E> Walker for PreOrderWalk<N, E>
where
    E: Ord,
{
    type Node = N;
    type Edge = E;
    fn next(&mut self, graph: &OrderedDag<Self::Node, Self::Edge>) -> Option<NodeId> {
        if let Some(node_id) = self.stack.pop() {
            let mut iter = graph
                .nodes
                .get(node_id)
                .unwrap()
                .edges
                .iter()
                .map(|e| e.child);
            // The stack pops from the back, so the first child to
            // be visited must be added last.
            while let Some(child_id) = iter.next_back() {
                self.stack.push(child_id);
            }

            Some(node_id)
        } else {
            None
        }
    }
}

pub struct PostOrderWalk<N, E> {
    stack: Vec<NodeId>,
    out: Vec<NodeId>,
    _marker: PhantomData<(N, E)>,
}

impl<N, E> Walker for PostOrderWalk<N, E>
where
    E: Ord,
{
    type Node = N;
    type Edge = E;
    fn next(&mut self, graph: &OrderedDag<Self::Node, Self::Edge>) -> Option<NodeId> {
        while let Some(node_id) = self.stack.pop() {
            self.out.push(node_id);
            let iter = graph
                .nodes
                .get(node_id)
                .unwrap()
                .edges
                .iter()
                .map(|e| e.child);

            for child_id in iter {
                self.stack.push(child_id);
            }
        }

        self.out.pop()
    }
}

pub struct ChildrenWalk<N, E> {
    node_id: NodeId,
    cursor: usize,
    _marker: PhantomData<(N, E)>,
}

impl<N, E> Walker for ChildrenWalk<N, E>
where
    E: Ord,
{
    type Node = N;
    type Edge = E;
    fn next(&mut self, graph: &OrderedDag<Self::Node, Self::Edge>) -> Option<NodeId> {
        if let Some(edge) = graph
            .nodes
            .get(self.node_id)
            .and_then(|n| n.edges.get(self.cursor))
        {
            self.cursor += 1;
            Some(edge.child)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_check_cycle() {
        let mut graph: OrderedDag<i64, i64> = OrderedDag::new();

        let node_1 = graph.insert(1);
        let node_2 = graph.insert(2);
        graph.set_edge(node_1, node_2, 0).unwrap();
        assert_eq!(graph.check_cycle(node_1), None);

        graph.set_edge_unchecked(node_2, node_1, 0).unwrap();
        println!("{}", graph.string());
        assert!(graph.check_cycle(node_1).is_some());
    }
}
