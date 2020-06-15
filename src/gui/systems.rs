use super::{BoundsRect, GlobalPosition, GuiGraph, HoveredWidget};
use crate::comp::Tag;
use glutin::{Event, WindowEvent};
use specs::prelude::*;

pub struct GuiMouseMoveSystem;

impl<'a> System<'a> for GuiMouseMoveSystem {
    type SystemData = GuiMouseData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let GuiMouseData {
            events,
            gui_graph,
            mut hovered,
            clickables,
            bounds_rects,
            global_positions,
            tags,
        } = data;

        for ev in events.iter() {
            if let Event::WindowEvent { event, .. } = ev {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        let (mouse_x, mouse_y) = (position.x as f32, position.y as f32);
                        // let (world_x, world_y) = (mouse_x / 1000.0, -mouse_y / 1000.0);
                        let mut walker = gui_graph.walk_dfs_post_order(gui_graph.root_id());
                        let mut found = false;

                        // TODO: Unfocus and hover out when cursor leaves window
                        // TODO: This graph walk will be the same for all mouse events. Refactor into function.
                        while let Some(node_id) = walker.next(&gui_graph) {
                            if let Some(entity) = gui_graph.get_entity(node_id) {
                                let maybe_components = (
                                    bounds_rects.get(entity),
                                    global_positions.get(entity),
                                    clickables.get(entity),
                                );
                                if let (Some(bounds), Some(global_pos), Some(_)) = maybe_components
                                {
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
                                            let name: &str = tags
                                                .get(entity)
                                                .map(|tag| tag.as_ref())
                                                .unwrap_or("");
                                            println!(
                                                "hover over {:?} {:?} '{}'",
                                                entity, node_id, name
                                            );
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

#[derive(SystemData)]
pub struct GuiMouseData<'a> {
    events: Read<'a, Vec<Event>>,
    gui_graph: ReadExpect<'a, GuiGraph>,
    hovered: Write<'a, HoveredWidget>,
    clickables: ReadStorage<'a, Clickable>,
    bounds_rects: ReadStorage<'a, BoundsRect>,
    global_positions: ReadStorage<'a, GlobalPosition>,
    tags: ReadStorage<'a, Tag>,
}

// ---------- //
// Components //
// ---------- //

/// Marks a widget as interactable via user mouse input.
#[derive(Component)]
pub struct Clickable;
