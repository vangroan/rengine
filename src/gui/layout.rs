//! Layout engine.
use super::{create_gui_proj_matrix, GlobalPosition, GuiGraph, Placement, WidgetBounds};
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
        println!(
            "{:?} suggested position {:?} {:?}",
            entity,
            parent_measure.suggested_pos,
            (parent_measure.suggested_pos / 1000.0).to_homogeneous()
        );

        if let Some(global_pos) = data.global_positions.get_mut(entity) {
            global_pos.set_point(parent_measure.suggested_pos);
        }

        // Convert logical pixel position to graphics position.
        // TODO: Pixel scale from GUI settings resource.
        let mut render_position = (parent_measure.suggested_pos / 1000.0).to_homogeneous();

        // GUI y increases downwards, graphics y increases upwards.
        render_position.y *= -1.;

        // Transform's position is subordinate to the layout engine.
        data.transforms
            .get_mut(entity)
            .unwrap_or_else(|| panic!("{:?} {:?} has no transform for layout", node_id, entity))
            .set_position(render_position);

        // Using Walker because an iterator borrows the graph.
        let mut walker = data.gui_graph.walk_children(node_id);

        // Accumulated value of the widths and heights of the previous children, in logical pixels.
        let mut acc_pack = [0.0, 0.0];

        while let Some(child_node_id) = walker.next(&data.gui_graph) {
            println!("child node id {:?}", child_node_id);

            // Parent suggesting a new position for the child.
            let mut pos = Point2::new(0., 0.);

            // Suggeted available space that the child may take up.
            let bounds = data.bounds.get(entity).unwrap().clone();

            if let Some(pack) = data.packs.get(entity) {
                match pack.mode {
                    PackMode::Frame => {
                        // TODO: Offset from anchor
                        pos = Point2::new(0., 0.);
                    }
                    PackMode::Horizontal => {
                        pos = Point2::new(acc_pack[0], 0.);

                        // Add bounds of current child to accumulator so the
                        // next child can be positioned by it.
                        acc_pack[0] += pack.margin[0]
                            + data
                                .bounds
                                .get(data.gui_graph.get_entity(child_node_id).unwrap())
                                .map(|b| b.width)
                                .unwrap_or_default();
                    }
                    PackMode::Vertical => {
                        pos = Point2::new(0., acc_pack[1]);

                        // Add bounds of current child to accumulator so the
                        // next child can be positioned by it.
                        acc_pack[1] += pack.margin[1]
                            + data
                                .bounds
                                .get(data.gui_graph.get_entity(child_node_id).unwrap())
                                .map(|b| b.height)
                                .unwrap_or_default();
                    }
                    PackMode::Grid { .. } => unimplemented!(),
                }
            }

            let pm = ParentMeasurements {
                // TODO: new bounds rect from pack mode
                bounds: bounds,
                // TODO: suggested position from pack mode
                suggested_pos: pos,
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
    global_positions: WriteStorage<'a, GlobalPosition>,
    packs: ReadStorage<'a, Pack>,
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

/// Indicates that the children of a Widget must be arranged in some way.
#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Pack {
    pub mode: PackMode,
    /// The vertical and horizontal spacing between child widgets in logical pixels.
    pub margin: [f32; 2],
}

impl Pack {
    pub fn new(mode: PackMode) -> Self {
        Pack {
            mode,
            margin: [0.0, 0.0],
        }
    }
}

#[derive(Debug)]
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
