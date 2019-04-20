use specs::World;

pub trait Scene {}

/// Wrapper for Scene world and dispatchers.
pub struct SceneState {
    world: World,
}

pub struct SceneStack {
    scenes: Vec<SceneState>,
}

impl SceneStack {
    pub fn new() -> Self {
        SceneStack { scenes: vec![] }
    }

    /// Retrieves the scene at the top of the stack.
    ///
    /// Returns `None` when the stack is empty.
    pub fn current(&self) -> Option<&SceneState> {
        unimplemented!()
    }

    /// Retrieves the scene at the top of the stack.
    ///
    /// Returns `None` when the stack is empty.
    pub fn current_mut(&mut self) -> Option<&mut SceneState> {
        unimplemented!()
    }

    /// Instantiates a new instance of the given
    /// scene type on the top of the stack.
    pub fn push<S>(&mut self)
    where
        S: Scene,
    {
        unimplemented!()
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
    pub fn replace<S>(&mut self)
    where
        S: Scene,
    {
        unimplemented!()
    }
}
