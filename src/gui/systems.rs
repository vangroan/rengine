use super::{GuiGraph, WidgetBounds};
use crate::comp::Transform;
use crate::option::lift2;
use glutin::{Event, WindowEvent};
use specs::{Entities, Join, Read, ReadExpect, ReadStorage, System};

pub struct GuiMouseMoveSystem;

impl<'a> System<'a> for GuiMouseMoveSystem {
    type SystemData = (
        Read<'a, Vec<Event>>,
        Entities<'a>,
        ReadExpect<'a, GuiGraph>,
        ReadStorage<'a, WidgetBounds>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (events, entities, gui_graph, aabbs, transforms) = data;

        for ev in events.iter() {
            if let Event::WindowEvent { event, .. } = ev {
                if let WindowEvent::CursorMoved { position, .. } = event {
                    let (mouse_x, mouse_y) = (position.x as f32, position.y as f32);
                    let (world_x, world_y) = (mouse_x / 1000.0, -mouse_y / 1000.0);
                    let mut walker = gui_graph.walk_dfs_post_order(gui_graph.root_id());

                    while let Some(node_id) = walker.next(&gui_graph) {
                        if let Some(entity) = gui_graph.get_entity(node_id) {
                            let maybe_components = lift2(aabbs.get(entity), transforms.get(entity));
                            if let Some((bounds, transform)) = maybe_components {
                                // Bounds are in the widget's local space.
                                if bounds.intersect_point([mouse_x, mouse_y]) {
                                    println!(
                                        "intersect ({}, {}) ({}, {}) {:?}",
                                        mouse_x, mouse_y, world_x, world_y, entity
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
