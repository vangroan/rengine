use std::error::Error;
use std::fmt;

pub trait Scene {
    fn on_start(&mut self) {}
    fn on_event(&mut self) {}
    fn on_update(&mut self) {}
    fn on_message(&mut self) {}
}

pub struct SceneStack {
    scenes: Vec<Box<dyn Scene>>,
    request: Option<Trans>,
}

impl SceneStack {
    pub fn new() -> Self {
        SceneStack {
            scenes: Vec::new(),
            request: None,
        }
    }

    /// Retrieves the scene at the top of the stack.
    ///
    /// Returns `None` when the stack is empty.
    pub fn current(&self) -> Option<&dyn Scene> {
        self.scenes.last().map(|scene_box| &**scene_box)
    }

    /// Retrieves the scene at the top of the stack.
    ///
    /// Returns `None` when the stack is empty.
    pub fn current_mut(&mut self) -> Option<&mut (dyn Scene + 'static)> {
        self.scenes.last_mut().map(|scene_box| &mut **scene_box)
    }

    /// Schedules the given instance of a
    /// scene on the top of the stack.
    pub fn push<S>(&mut self, scene: S) -> bool
    where
        S: 'static + Scene,
    {
        if self.request.is_some() {
            false
        } else {
            self.request = Some(Trans::Push(Box::new(scene)));
            true
        }
    }

    /// Schedules the given instance of a
    /// scene on the top of the stack.
    pub fn push_box(&mut self, scene_box: Box<dyn Scene>) -> bool {
        if self.request.is_some() {
            false
        } else {
            self.request = Some(Trans::Push(scene_box));
            true
        }
    }

    /// Removes the current scene at the top of the
    /// stack.
    ///
    /// Does nothing when the stack is empty.
    pub fn pop(&mut self) {
        unimplemented!()
    }

    /// Removes the current scene at the top of the
    /// stack, if any. A new instance of the given
    /// scene type will be pushed to the top of the
    /// stack.
    pub fn replace<T>(&mut self) {
        unimplemented!()
    }
}

/// Methods for applying a stack change from
/// a request, during maintain
impl SceneStack {
    pub fn maintain(&mut self) -> SceneResult {
        if let Some(request) = self.request.take() {
            use Trans::*;

            match request {
                Push(scene) => {
                    self.apply_push(scene);
                    Ok(())
                }
                _ => unimplemented!(),
            }
        } else {
            Ok(())
        }
    }

    fn apply_push(&mut self, scene_box: Box<dyn Scene>) {
        self.scenes.push(scene_box);

        if let Some(ref mut s) = self.current_mut() {
            s.on_start();
        }
    }

    fn apply_pop(&mut self) {
        unimplemented!()
    }

    fn apply_replace(&mut self) {
        unimplemented!()
    }
}

/// Methods for dispatching main loop events
impl SceneStack {
    pub fn dispatch_update(&mut self) {
        if let Some(ref mut scene) = self.current_mut() {
            // scene.dispatch_update();
        }
    }
}

enum Trans {
    Push(Box<dyn Scene>),
    Pop,
    Replace(Box<dyn Scene>),
}

pub type SceneResult = Result<(), SceneError>;

#[derive(Debug)]
pub enum SceneError {
    KeyNotRegistered(&'static str),
}

impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use SceneError::*;

        write!(
            f,
            "Scene Error {}",
            match self {
                KeyNotRegistered(_) => "Scene key not registered",
            }
        )
    }
}

impl Error for SceneError {
    fn description(&self) -> &str {
        use SceneError::*;

        match self {
            KeyNotRegistered(_) => "Transition attempt to scene key which has not been registered",
        }
    }
}
