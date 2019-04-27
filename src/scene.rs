use crate::graphics::GraphicContext;
use specs::World;
use std::error::Error;
use std::fmt;

pub trait Scene {
    fn on_start(&mut self, _ctx: &mut Context<'_>) -> Option<Trans> {
        None
    }

    fn on_stop(&mut self, _ctx: &mut Context<'_>) -> Option<Trans> {
        None
    }

    fn on_resume(&mut self) {}
    fn on_pause(&mut self) {}

    fn on_event(&mut self, _ctx: &mut Context<'_>, _ev: &glutin::Event) -> Option<Trans> {
        None
    }

    fn on_update(&mut self, _ctx: &mut Context<'_>) -> Option<Trans> {
        None
    }

    fn on_message(&mut self) {}
}

pub struct Context<'a> {
    pub world: &'a mut World,
    pub graphics: &'a mut GraphicContext,
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
    pub fn pop(&mut self) -> bool {
        if self.request.is_some() {
            false
        } else {
            self.request = Some(Trans::Pop);
            true
        }
    }

    /// Removes the current scene at the top of the
    /// stack, if any. A new instance of the given
    /// scene type will be pushed to the top of the
    /// stack.
    pub fn replace<S>(&mut self, scene: S) -> bool
    where
        S: 'static + Scene,
    {
        if self.request.is_some() {
            false
        } else {
            self.request = Some(Trans::Replace(Box::new(scene)));
            true
        }
    }
}

/// Methods for applying a stack change from
/// a request, during maintain
impl SceneStack {
    pub fn maintain(&mut self, world: &mut World, graphics: &mut GraphicContext) -> SceneResult {
        if let Some(request) = self.request.take() {
            use Trans::*;

            match request {
                Push(scene_box) => {
                    self.apply_push(scene_box, world, graphics);
                    Ok(())
                }
                Pop => {
                    self.apply_pop(world, graphics);
                    Ok(())
                }
                Replace(scene_box) => {
                    self.apply_replace(scene_box, world, graphics);
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    /// Pops each scene off the stack, calling `on_stop` for each.
    pub fn clear(&mut self, world: &mut World, graphics: &mut GraphicContext) -> SceneResult {
        while let Some(mut s) = self.scenes.pop() {
            let mut ctx = Context { world, graphics };
            let trans = s.on_stop(&mut ctx);
            if !trans.is_none() {
                self.request = trans;
            }
        }

        Ok(())
    }

    fn apply_push(
        &mut self,
        scene_box: Box<dyn Scene>,
        world: &mut World,
        graphics: &mut GraphicContext,
    ) {
        if let Some(ref mut s) = self.current_mut() {
            s.on_pause();
        }

        self.scenes.push(scene_box);

        if let Some(ref mut s) = self.current_mut() {
            let mut ctx = Context { world, graphics };
            let trans = s.on_start(&mut ctx);
            if !trans.is_none() {
                self.request = trans;
            }
        }
    }

    fn apply_pop(&mut self, world: &mut World, graphics: &mut GraphicContext) {
        if let Some(ref mut s) = self.current_mut() {
            let mut ctx = Context { world, graphics };
            let trans = s.on_stop(&mut ctx);
            if !trans.is_none() {
                self.request = trans;
            }
        }

        self.scenes.pop();

        if let Some(ref mut s) = self.current_mut() {
            s.on_resume();
        }
    }

    fn apply_replace(
        &mut self,
        scene_box: Box<dyn Scene>,
        world: &mut World,
        graphics: &mut GraphicContext,
    ) {
        if let Some(ref mut s) = self.current_mut() {
            let mut ctx = Context { world, graphics };
            let trans = s.on_stop(&mut ctx);
            if !trans.is_none() {
                self.request = trans;
            }
        }

        self.scenes.pop();
        self.scenes.push(scene_box);

        if let Some(ref mut s) = self.current_mut() {
            let mut ctx = Context { world, graphics };
            let trans = s.on_start(&mut ctx);
            if !trans.is_none() {
                self.request = trans;
            }
        }
    }
}

/// Methods for dispatching main loop events
impl SceneStack {
    pub fn dispatch_update(&mut self, world: &mut World, graphics: &mut GraphicContext) {
        if let Some(ref mut scene) = self.current_mut() {
            let mut ctx = Context { world, graphics };
            let trans = scene.on_update(&mut ctx);
            if !trans.is_none() {
                self.request = trans;
            }
        }
    }

    pub fn dispatch_event(
        &mut self,
        world: &mut World,
        graphics: &mut GraphicContext,
        event: &glutin::Event,
    ) {
        if let Some(ref mut scene) = self.current_mut() {
            let mut ctx = Context { world, graphics };
            let trans = scene.on_event(&mut ctx, event);
            if !trans.is_none() {
                self.request = trans;
            }
        }
    }
}

pub enum Trans {
    Push(Box<dyn Scene>),
    Pop,
    Replace(Box<dyn Scene>),
}

impl Trans {
    pub fn push<S>(scene: S) -> Option<Trans>
    where
        S: 'static + Scene,
    {
        Some(Trans::Push(Box::new(scene)))
    }

    pub fn replace<S>(scene: S) -> Option<Trans>
    where
        S: 'static + Scene,
    {
        Some(Trans::Replace(Box::new(scene)))
    }
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
