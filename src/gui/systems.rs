use super::{BoundsRect, GlobalPosition, GuiGraph, HoveredWidget, NodeId};
use crate::comp::Tag;
use glutin::{Event, WindowEvent};
use shrev::EventChannel;
use specs::prelude::*;

pub struct GuiMouseMoveSystem;

impl<'a> System<'a> for GuiMouseMoveSystem {
    type SystemData = GuiMouseData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let GuiMouseData {
            events,
            mut gui_events,
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
                        // TODO: Unfocus and hover out when cursor leaves window

                        if let Some((entity, node_id)) = find_widget(
                            FindWidgetData {
                                gui_graph: &gui_graph,
                                global_positions: &global_positions,
                                bounds_rects: &bounds_rects,
                                clickables: &clickables,
                            },
                            [position.x as f32, position.y as f32],
                        ) {
                            if hovered.entity() != Some(entity) {
                                let name: &str =
                                    tags.get(entity).map(|tag| tag.as_ref()).unwrap_or("");
                                println!("hover over {:?} {:?} '{}'", entity, node_id, name);
                                hovered.set(entity, node_id);
                                gui_events.single_write(WidgetEvent {
                                    entity,
                                    node_id,
                                    kind: WidgetEventKind::HoverOver,
                                    window_event: event.clone(),
                                });
                            }
                        } else if let Some((entity, node_id)) = hovered.clear() {
                            println!("hover out {:?} {:?}", entity, node_id);
                            gui_events.single_write(WidgetEvent {
                                entity,
                                node_id,
                                kind: WidgetEventKind::HoverOut,
                                window_event: event.clone(),
                            });
                        }
                    }
                    WindowEvent::MouseInput { .. } => {
                        // TODO: Focus on click
                        // TODO: Emit GUI event on click
                    }
                    WindowEvent::MouseWheel { .. } => {
                        // TODO: Emit GUI event on mouse wheel
                    }
                    WindowEvent::KeyboardInput { .. } => {
                        // TODO: Focussed widget receives keyboard events
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
    gui_events: Write<'a, EventChannel<WidgetEvent>>,
    gui_graph: ReadExpect<'a, GuiGraph>,
    hovered: Write<'a, HoveredWidget>,
    clickables: ReadStorage<'a, Clickable>,
    bounds_rects: ReadStorage<'a, BoundsRect>,
    global_positions: ReadStorage<'a, GlobalPosition>,
    tags: ReadStorage<'a, Tag>,
}

#[derive(SystemData)]
struct FindWidgetData<'run, 'res: 'run> {
    gui_graph: &'run ReadExpect<'res, GuiGraph>,
    global_positions: &'run ReadStorage<'res, GlobalPosition>,
    bounds_rects: &'run ReadStorage<'res, BoundsRect>,
    clickables: &'run ReadStorage<'res, Clickable>,
}

fn find_widget(data: FindWidgetData, mouse_position: [f32; 2]) -> Option<(Entity, NodeId)> {
    let FindWidgetData {
        gui_graph,
        global_positions,
        bounds_rects,
        clickables,
    } = data;
    let [mouse_x, mouse_y] = mouse_position;

    let mut walker = gui_graph.walk_dfs_post_order(gui_graph.root_id());
    while let Some(node_id) = walker.next(&gui_graph) {
        if let Some(entity) = gui_graph.get_entity(node_id) {
            let maybe_components = (
                bounds_rects.get(entity),
                global_positions.get(entity),
                clickables.get(entity),
            );

            if let (Some(bounds), Some(global_pos), Some(_)) = maybe_components {
                // Bounds are in the widget's local space.
                let global_point = global_pos.point();
                let local_point = [mouse_x - global_point.x, mouse_y - global_point.y];
                if bounds.intersect_point(local_point) {
                    return Some((entity, node_id));
                }
            }
        }
    }
    None
}

// ---------- //
// Components //
// ---------- //

/// Marks a widget as interactable via user mouse input.
#[derive(Component)]
pub struct Clickable;

// -------------- //
// Event Messages //
// -------------- //

pub type WidgetEvents = EventChannel<WidgetEvent>;

#[derive(Debug)]
pub struct WidgetEvent {
    /// Entity id of the widget that handled the event.
    entity: specs::Entity,
    /// Node id in the GUI graph for the widget.
    node_id: crate::gui::NodeId,
    /// GUI event kind.
    kind: WidgetEventKind,
    /// Window event that caused this GUI event.
    window_event: glutin::WindowEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetEventKind {
    HoverOver,
    HoverOut,
    Pressed,
    Released,
}
