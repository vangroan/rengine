use daggy::Dag;
use specs::Entity;

mod builder;
mod drawable;
mod draw;
mod widget;

pub use builder::*;
pub use drawable::*;
pub use draw::*;
pub use widget::*;

// TODO: Layout
// TODO: Cleaning up Widgets when scene is stopped
// TODO: GUI renderer

pub struct GuiGraph {
    root_id: WidgetId,
    graph: Dag<Entity, Child, WidgetIndexType>,
}

impl GuiGraph {
    /// Create a new `GuiGraph` instance with
    /// a root Entity.
    pub fn with_root(root_entity: Entity) -> Self {
        let mut graph = Dag::new();
        let root_id = graph.add_node(root_entity).into();

        GuiGraph { root_id, graph }
    }

    pub fn insert_entity(&mut self, entity: Entity, parent: Option<WidgetId>) -> WidgetId {
        // When no parent is specified, add to root.
        let parent_index = parent.unwrap_or(self.root_id.clone()).node_index();

        // TODO: Do we need to keep the Edge index?
        let (_edge_index, node_index) = self.graph.add_child(parent_index, Child, entity).into();

        node_index.into()
    }

    /// Remove all widgets in the GUI that are associated
    /// with the given entities.
    pub fn delete_entities(&mut self, _entities: &[Entity]) {
        unimplemented!()
    }
}

/// Edge of the Widget parent -> child
/// relationship.
struct Child;
