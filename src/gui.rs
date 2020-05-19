use crate::collections::ordered_dag::prelude::*;
use crate::collections::ordered_dag::{ChildrenWalk, PostOrderWalk};
use specs::Entity;

mod aabb;
mod builder;
mod draw;
mod drawable;
mod layout;
mod mesh;
mod placement;
mod pos;
mod proj;
mod systems;
mod widget;
pub mod widgets;

pub use aabb::*;
pub use builder::*;
pub use draw::*;
pub use drawable::*;
pub use layout::*;
pub use mesh::*;
pub use placement::*;
pub use pos::*;
pub use proj::*;
pub use systems::*;
pub use widget::*;

/// General global settings for configuring the GUI.
pub struct GuiSettings {
    /// The width and height, in logical pixel size, of a 1 by 1 quad when
    /// rendered to the window.
    ///
    /// Also important for transforming mouse coordinates from the screen
    /// to 3D world coordinates.
    pub pixel_scale: f32,
}

// TODO: Layout
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
        self.graph.node(node_id).map(|n| *n)
    }

    /// Remove all widgets in the GUI that are associated
    /// with the given entities.
    pub fn delete_entities(&mut self, _entities: &[Entity]) {
        unimplemented!()
    }

    pub fn walk_dfs_post_order(&self, node_id: NodeId) -> WidgetDfsPostOrderWalk {
        WidgetDfsPostOrderWalk(self.graph.walk_post_order(node_id))
    }

    pub fn walk_children(&self, node_id: NodeId) -> WidgetChildrenWalk {
        WidgetChildrenWalk(self.graph.walk_children(node_id))
    }

    pub fn debug_print(&self) {
        println!("{}", self.graph.string());
    }
}

/// Edge of the Widget parent -> child
/// relationship.
#[derive(Debug, Default, PartialOrd, Ord, PartialEq, Eq)]
struct Child {
    order_index: u16,
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
