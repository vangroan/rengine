use super::WidgetBounds;
use glutin::{Event, WindowEvent};
use specs::{Entities, Join, Read, ReadStorage, System};

pub struct GuiMouseMoveSystem;

impl<'a> System<'a> for GuiMouseMoveSystem {
    type SystemData = (
        Read<'a, Vec<Event>>,
        Entities<'a>,
        ReadStorage<'a, WidgetBounds>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (events, entities, aabbs) = data;

        for ev in events.iter() {
            if let Event::WindowEvent { event, .. } = ev {
                if let WindowEvent::CursorMoved { position, .. } = event {
                    let (x, y) = (position.x as f32, position.y as f32);
                    let (world_x, world_y) = (x / 1000.0, -y / 1000.0);
                    for (entity, aabb) in (&entities, &aabbs).join() {
                        if aabb.intersect_point([x, y]) {
                            println!(
                                "intersect ({}, {}) ({}, {}) {:?}",
                                x, y, world_x, world_y, entity
                            );
                            break;
                        }
                    }
                }
            }
        }
    }
}
