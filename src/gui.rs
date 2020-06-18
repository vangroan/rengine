use crate::collections::ordered_dag::prelude::*;
use crate::collections::ordered_dag::{ChildrenWalk, PostOrderWalk, PreOrderWalk};
use specs::Entity;

pub use crate::collections::ordered_dag::NodeId;

mod builder;
mod draw;
mod layout;
mod mesh;
mod proj;
mod systems;
pub mod text;
mod widget;
pub mod widgets;

pub use builder::*;
pub use draw::*;
pub use layout::*;
pub use mesh::*;
pub use proj::*;
pub use systems::*;
pub use widget::*;

// TODO: Cleaning up Widgets when scene is stopped

pub struct GuiGraph {
    root_id: NodeId,
    graph: OrderedDag<Entity, Child>,
}

impl GuiGraph {
    /// Create a new `GuiGraph` instance with
    /// a root Entity.
    pub fn with_root(root_entity: Entity) -> Self {
        let mut graph = OrderedDag::new();
        let root_id = graph.insert(root_entity);

        GuiGraph { root_id, graph }
    }

    #[inline]
    pub fn root_id(&self) -> NodeId {
        self.root_id
    }

    /// Retrieve the entity id at the root of the graph.
    ///
    /// # Panics
    ///
    /// If the root entity was removed from the graph.
    #[inline]
    pub fn root_entity(&self) -> Entity {
        *self
            .graph
            .node(self.root_id)
            .expect("GUI root entity not found in graph")
    }

    pub fn insert_entity(&mut self, entity: Entity, parent: Option<NodeId>) -> NodeId {
        // When no parent is specified, add to root.
        let parent_index = parent.unwrap_or_else(|| self.root_id);

        self.graph.insert_at(entity, Some(parent_index))
    }

    pub fn get_entity(&self, node_id: NodeId) -> Option<Entity> {
        self.graph.node(node_id).cloned()
    }

    /// Remove all widgets in the GUI that are associated
    /// with the given entities.
    pub fn delete_entities(&mut self, _entities: &[Entity]) {
        unimplemented!()
    }

    pub fn walk_dfs_pre_order(&self, node_id: NodeId) -> WidgetDfsPreOrderWalk {
        WidgetDfsPreOrderWalk(self.graph.walk_pre_order(node_id))
    }

    pub fn walk_dfs_post_order(&self, node_id: NodeId) -> WidgetDfsPostOrderWalk {
        WidgetDfsPostOrderWalk(self.graph.walk_post_order(node_id))
    }

    pub fn walk_children(&self, node_id: NodeId) -> WidgetChildrenWalk {
        WidgetChildrenWalk(self.graph.walk_children(node_id))
    }

    pub fn debug_print(&self) {
        pretty_print_gui(&self.graph, self.root_id, 0, false);
        // println!("{}", self.graph.string());
    }
}

/// Edge of the Widget parent -> child
/// relationship.
#[derive(Debug, Default, PartialOrd, Ord, PartialEq, Eq)]
struct Child {
    order_index: u16,
}

pub struct WidgetDfsPreOrderWalk(PreOrderWalk<Entity, Child>);

impl WidgetDfsPreOrderWalk {
    pub fn next(&mut self, gui_graph: &GuiGraph) -> Option<NodeId> {
        self.0.next(&gui_graph.graph)
    }
}

pub struct WidgetDfsPostOrderWalk(PostOrderWalk<Entity, Child>);

impl WidgetDfsPostOrderWalk {
    pub fn next(&mut self, gui_graph: &GuiGraph) -> Option<NodeId> {
        self.0.next(&gui_graph.graph)
    }
}

pub struct WidgetChildrenWalk(ChildrenWalk<Entity, Child>);

impl WidgetChildrenWalk {
    pub fn next(&mut self, gui_graph: &GuiGraph) -> Option<NodeId> {
        self.0.next(&gui_graph.graph)
    }
}

#[derive(Debug, Default)]
pub struct HoveredWidget(Option<(Entity, NodeId)>);

impl HoveredWidget {
    #[inline]
    pub fn entity(&self) -> Option<Entity> {
        self.0.map(|(e, _)| e)
    }

    #[inline]
    pub fn node_id(&self) -> Option<NodeId> {
        self.0.map(|(_, n)| n)
    }

    #[inline]
    pub fn set(&mut self, entity: Entity, node_id: NodeId) {
        self.0 = Some((entity, node_id))
    }

    #[inline]
    pub fn has_widget(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    pub fn clear(&mut self) -> Option<(Entity, NodeId)> {
        self.0.take()
    }
}

fn pretty_print_gui(graph: &OrderedDag<Entity, Child>, node_id: NodeId, level: i32, last: bool) {
    let mut indent = String::new();

    // First node and first level of children will have no pipes prepended.
    let pipe_count = i32::max(level - 2, 0);
    for _ in 0..pipe_count {
        indent.push('│');
    }

    // if level == 0 {
    //     // indent.push('┌');
    // } else if level == 1 {
    //     indent.push('├');
    // } else {
    //     indent.push_str("|├");
    // }
    if last {
        indent.push_str("└");
    } else {
        indent.push_str("├");
    }

    // for _ in 0..level {
    //     indent.push_str("─");
    // }

    println!("{}{:?}", indent, node_id);

    let mut walker = graph.walk_children(node_id);
    let mut cursor = 0;
    let count = graph.out_edge_len(node_id).unwrap();
    while let Some(child_id) = walker.next(&graph) {
        pretty_print_gui(graph, child_id, level + 1, cursor == count - 1);
        cursor += 1;
    }
}
