use specs::{RunNow, World};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::time;

/// Wrapper for Scene world and dispatchers.
pub struct SceneState<'a> {
    world: World,
    update: Vec<Box<dyn RunNow<'a>>>,
    signal: Vec<Box<dyn RunNow<'a>>>,
}

pub struct SceneBuilder<'a> {
    world: World,
    update: Vec<Box<dyn RunNow<'a>>>,
    signal: Vec<Box<dyn RunNow<'a>>>,
}

impl<'a> SceneBuilder<'a> {
    fn new() -> Self {
        SceneBuilder {
            world: World::new(),
            update: Vec::new(),
            signal: Vec::new(),
        }
    }

    pub fn update<T>(mut self, system: T) -> Self
    where
        T: 'static + RunNow<'a>,
    {
        self.update.push(Box::new(system));
        self
    }

    pub fn signal<T>(mut self, system: T) -> Self
    where
        T: 'static + RunNow<'a>,
    {
        self.signal.push(Box::new(system));
        self
    }

    pub(crate) fn build(self) -> SceneState<'a> {
        let SceneBuilder {
            world,
            update,
            signal,
        } = self;

        SceneState {
            world,
            update,
            signal,
        }
    }
}

pub struct SceneFactories(HashMap<&'static str, Box<dyn Fn(SceneBuilder) -> SceneBuilder>>);

impl SceneFactories {
    pub fn new() -> Self {
        SceneFactories(HashMap::new())
    }

    pub fn add<F>(&mut self, key: &'static str, factory: F)
    where
        F: 'static + Fn(SceneBuilder) -> SceneBuilder,
    {
        self.0.insert(key, Box::new(factory));
    }

    pub fn create<'a>(&self, key: &'static str) -> Option<SceneState<'a>> {
        let builder = SceneBuilder::new();
        match self.0.get(key) {
            Some(ref factory) => Some(factory(builder).build()),
            None => None,
        }
    }
}

pub trait SceneDispatch<'a> {
    fn dispatch_update(&mut self, delta_time: time::Duration);
}

pub struct SceneStack<'a> {
    factories: SceneFactories,
    scenes: Vec<SceneState<'a>>,
    request: Option<TransitionRequest>,
}

impl<'a> SceneStack<'a> {
    pub fn new(factories: SceneFactories) -> Self {
        SceneStack {
            factories,
            scenes: Vec::new(),
            request: None,
        }
    }

    /// Retrieves the scene at the top of the stack.
    ///
    /// Returns `None` when the stack is empty.
    pub fn current(&self) -> Option<&SceneDispatch> {
        unimplemented!()
    }

    /// Retrieves the scene at the top of the stack.
    ///
    /// Returns `None` when the stack is empty.
    pub fn current_mut(&mut self) -> Option<&mut SceneDispatch> {
        unimplemented!()
    }

    /// Instantiates a new instance of the given
    /// scene type on the top of the stack.
    pub fn push(&mut self, key: &'static str) -> bool {
        if self.request.is_some() {
            false
        } else {
            self.request = Some(TransitionRequest::Push(key));
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
impl<'a> SceneStack<'a> {
    pub fn maintain(&mut self) -> SceneResult {
        if let Some(request) = self.request.take() {
            use TransitionRequest::*;

            match request {
                Push(key) => {
                    if self.factories.0.contains_key(key) {
                        self.apply_push(key);
                        Ok(())
                    } else {
                        Err(SceneError::KeyNotRegistered(key))
                    }
                }
                _ => unimplemented!(),
            }
        } else {
            Ok(())
        }
    }

    fn apply_push(&mut self, key: &'static str) {
        match self.factories.create(key) {
            Some(scene) => self.scenes.push(scene),
            None => {}
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
impl<'a> SceneStack<'a> {
    pub fn dispatch_update(&mut self, delta_time: time::Duration) {
        if let Some(ref mut scene) = self.current_mut() {
            scene.dispatch_update(delta_time);
        }
    }
}

enum TransitionRequest {
    Push(&'static str),
    Pop,
    Replace(&'static str),
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
