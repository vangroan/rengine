use crate::comp::Tag;
use std::sync::RwLock;

lazy_static! {
    static ref WIDGET_COUNTER: RwLock<WidgetCounter> = RwLock::new(WidgetCounter::default());
}

#[derive(Debug, Default)]
struct WidgetCounter(u128);

impl WidgetCounter {
    fn incr(&mut self) {
        self.0 += 1;
    }

    fn inner(&self) -> u128 {
        self.0
    }
}

/// Creates a new name tag for a widget.
pub fn next_widget_tag() -> Tag {
    let mut counter = WIDGET_COUNTER.write().unwrap();
    counter.incr();
    Tag::new(format!("Widget {}", counter.inner()))
}
