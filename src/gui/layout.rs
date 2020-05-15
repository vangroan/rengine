//! Layout engine.
use super::{GuiGraph, Placement};
use crate::collections::ordered_dag::NodeId;
use specs::{Component, DenseVecStorage, ReadExpect, ReadStorage, System, Write};

pub struct GuiLoayoutSystem;

impl<'a> System<'a> for GuiLoayoutSystem {
    type SystemData = LayoutData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if let Some(node_id) = data.layout_dirty.take_node_id() {
            process_layout(data, node_id);
        }
    }
}

pub fn process_layout(data: LayoutData, node_id: NodeId) {
    if let Some(entity) = data.gui_graph.get_entity(node_id) {
        let LayoutData {
            gui_graph,
            placements,
            pack_modes,
            ..
        } = data;

        if let Some(pack_mode) = pack_modes.get(entity) {
            match pack_mode {
                PackMode::Frame => {}
                _ => unimplemented!(),
            }
        }
    }
}

/// Resources and components required to recalculate the GUI layout.
#[derive(SystemData)]
pub struct LayoutData<'a> {
    gui_graph: ReadExpect<'a, GuiGraph>,
    layout_dirty: Write<'a, LayoutDirty>,
    placements: ReadStorage<'a, Placement>,
    pack_modes: ReadStorage<'a, PackMode>,
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub enum PackMode {
    Vertical,
    Horizontal,
    Grid { columns: u16 },
    Frame,
}

/// Marks the GUI graph as dirty, starting at the given node.
#[derive(Debug, Default)]
pub struct LayoutDirty(Option<NodeId>);

impl LayoutDirty {
    pub fn with_node_id(node_id: NodeId) -> Self {
        LayoutDirty(Some(node_id))
    }

    pub fn set_node_id(&mut self, node_id: NodeId) {
        self.0 = Some(node_id);
    }

    pub fn node_id(&self) -> Option<NodeId> {
        self.0
    }

    pub fn take_node_id(&mut self) -> Option<NodeId> {
        self.0.take()
    }
}
