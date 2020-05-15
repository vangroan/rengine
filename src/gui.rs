use crate::collections::ordered_dag::{NodeId, OrderedDag};
use specs::Entity;

mod aabb;
mod builder;
mod draw;
mod drawable;
mod layout;
mod mesh;
mod placement;
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
pub use proj::*;
pub use systems::*;
pub use widget::*;

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

    pub fn insert_entity(&mut self, entity: Entity, parent: Option<NodeId>) -> NodeId {
        // When no parent is specified, add to root.
        let parent_index = parent.unwrap_or_else(|| self.root_id.clone());

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
}

/// Edge of the Widget parent -> child
/// relationship.
#[derive(Debug, Default, PartialOrd, Ord, PartialEq, Eq)]
struct Child {
    order_index: u16,
}
