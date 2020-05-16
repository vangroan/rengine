//! Layout engine.
use super::{create_gui_proj_matrix, GuiGraph, Placement, WidgetBounds};
use crate::collections::ordered_dag::prelude::*;
use crate::comp::Transform;
use crate::res::DeviceDimensions;
use glutin::dpi::LogicalSize;
use log::warn;
use nalgebra::{Matrix4, Point2, Vector2, Vector3};
use specs::{Component, DenseVecStorage, ReadExpect, ReadStorage, System, Write, WriteStorage};

pub struct GuiLayoutSystem;

impl<'a> System<'a> for GuiLayoutSystem {
    type SystemData = LayoutData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if let Some(node_id) = data.layout_dirty.take_node_id() {
            println!("processing layout");

            // Set the root widget's dimensions to match the device to be rendered to.
            let LogicalSize { width, height } = *data.device_dim.logical_size();
            data.bounds
                .get_mut(data.gui_graph.root_entity())
                .expect("GUI root entity has no bounds")
                .set_size([width as f32, height as f32]);
            // TODO: Pixel scale from a configurable resource
            let proj_matrix = create_gui_proj_matrix(
                *data.device_dim.physical_size(),
                1000.0,
                data.device_dim.dpi_factor() as f32,
            );

            // TODO: Is it reasonable to use a node id in the dirty flag to start
            //       the recalc from an arbitrary node?
            let parent_measure = ParentMeasurements {
                bounds: WidgetBounds::new(width as f32, height as f32),
                suggested_pos: Point2::new(0.0, 0.0),
            };
            process_layout(&mut data, node_id, parent_measure, proj_matrix);
        }
    }
}

/// Layout pass of the GUI graph.
///
/// Recursive call to change a Widget's Transform according to its layout rules.
pub fn process_layout(
    data: &mut LayoutData,
    node_id: NodeId,
    parent_measure: ParentMeasurements,
    proj: Matrix4<f32>,
) {
    if let Some(entity) = data.gui_graph.get_entity(node_id) {
        // New local position of the current widget, relative to its parent.
        //
        // All calculations are in logical pixels.
        let pos: Vector3<f32>;

        // If the Widget has an optional Placement, its new position is set
        // relative the view.
        if let Some(placement) = data.placements.get(entity) {
            println!("{:?} {:?}", entity, placement);
            let offset = placement.offset();
            let parent_size = parent_measure.bounds.size();
            pos = Vector3::new(parent_size[0] * offset.x, parent_size[1] * offset.y, 0.0);
        } else {
            // Use the suggested transform from the parent Widget.
            pos = parent_measure.suggested_pos.to_homogeneous();
        }

        println!("{:?} new position {:?}", entity, pos);

        if let Some(pack_mode) = data.pack_modes.get(entity) {
            match pack_mode {
                PackMode::Frame => {}
                _ => unimplemented!(),
            }
        }

        // Convert logical pixel position to graphics position.
        // let render_position = proj.try_inverse().unwrap().transform_vector(&pos);
        // TODO: Create matrix that will transform logical pixel positions and sizes to render world space.
        // data.transforms
        //     .get_mut(entity)
        //     .unwrap_or_else(|| panic!("{:?} {:?} has no transform for layout", node_id, entity))
        //     .set_position(pos / 1000.0);

        let bounds = data.bounds.get(entity).unwrap().clone();

        let mut walker = data.gui_graph.walk_children(node_id);
        while let Some(child_node_id) = walker.next(&data.gui_graph) {
            println!("child {:?}", child_node_id);
            let pm = ParentMeasurements {
                // TODO: new bounds rect from pack mode
                bounds: bounds.clone(),
                // TODO: suggested position from pack mode
                suggested_pos: Point2::new(pos.x, pos.y),
            };
            process_layout(data, child_node_id, pm, proj);
        }
    } else {
        warn!("Entity for {:?} not found during layout pass.", node_id);
    }
}

/// Resources and components required to recalculate the GUI layout.
#[derive(SystemData)]
pub struct LayoutData<'a> {
    device_dim: ReadExpect<'a, DeviceDimensions>,
    gui_graph: ReadExpect<'a, GuiGraph>,
    layout_dirty: Write<'a, LayoutDirty>,
    bounds: WriteStorage<'a, WidgetBounds>,
    placements: ReadStorage<'a, Placement>,
    pack_modes: ReadStorage<'a, PackMode>,
    transforms: WriteStorage<'a, Transform>,
}

/// Measurements calculated by the parent widget and passed to the child during
/// a recursive layout pass.
pub struct ParentMeasurements {
    /// The parent widget's bounding box.
    bounds: WidgetBounds,

    /// A global world position the parent has calculated that child can
    /// optionally use to position itself.
    suggested_pos: Point2<f32>,
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

pub enum MeasurementMode {
    /// In Parent mode the Widget will conform to the space its
    /// parent assigns to it.
    Parent,

    /// In Content mode the Widget will attempt to wrap its
    /// content, and will be asked by the parent how much space
    /// it needs.
    Content,
}
