use super::{BoundsRect, GlobalPosition, GuiGraph, HoveredWidget};
use crate::comp::Transform;
use crate::option::lift2;
use glutin::{Event, WindowEvent};
use specs::{Entities, Read, ReadExpect, ReadStorage, System, Write};

pub struct GuiMouseMoveSystem;

impl<'a> System<'a> for GuiMouseMoveSystem {
    type SystemData = (
        Read<'a, Vec<Event>>,
        Entities<'a>,
        ReadExpect<'a, GuiGraph>,
        Write<'a, HoveredWidget>,
        ReadStorage<'a, BoundsRect>,
        ReadStorage<'a, GlobalPosition>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (events, entities, gui_graph, mut hovered, aabbs, global_positions, transforms) = data;

        for ev in events.iter() {
            if let Event::WindowEvent { event, .. } = ev {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        let (mouse_x, mouse_y) = (position.x as f32, position.y as f32);
                        let (world_x, world_y) = (mouse_x / 1000.0, -mouse_y / 1000.0);
                        let mut walker = gui_graph.walk_dfs_post_order(gui_graph.root_id());
                        let mut found = false;

                        // TODO: Unfocus and hover out when cursor leaves window

                        while let Some(node_id) = walker.next(&gui_graph) {
                            if let Some(entity) = gui_graph.get_entity(node_id) {
                                let maybe_components =
                                    lift2(aabbs.get(entity), global_positions.get(entity));
                                if let Some((bounds, global_pos)) = maybe_components {
                                    // Bounds are in the widget's local space.
                                    let global_point = global_pos.point();
                                    let local_point =
                                        [mouse_x - global_point.x, mouse_y - global_point.y];
                                    if bounds.intersect_point(local_point) {
                                        // println!(
                                        //     "intersect ({}, {}) ({}, {}) {:?}",
                                        //     mouse_x, mouse_y, world_x, world_y, entity
                                        // );
                                        if hovered.entity() != Some(entity) {
                                            println!("hover over {:?} {:?}", entity, node_id);
                                            hovered.set(entity, node_id);
                                        }
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if !found {
                            if let Some((e, n)) = hovered.clear() {
                                println!("hover out {:?} {:?}", e, n);
                            }
                        }
                    }
                    WindowEvent::MouseInput { .. } => {
                        // TODO: Focus on click
                        // TODO: Emit GUI event on click
                    }
                    WindowEvent::MouseWheel { .. } => {
                        // TODO: Emit GUI event on mouse wheel
                    }
                    _ => {}
                }
            }
        }
    }
}
