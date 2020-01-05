pub enum ModCmd {
    /// Initialise all loaded mods, spawning script runners and
    /// runs the entry script for each mod.
    Init,

    /// Gracefully shuts down the thread running the Lua state.
    Shutdown,

    /// Scene lifetime hook.
    Scene(SceneHook),

    /// Placeholder for custom game input commands
    Game,
}

pub enum SceneHook {
    /// Scene has started.
    AfterStart,
}
