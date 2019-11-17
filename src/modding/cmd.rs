
pub enum ModCmd {
    /// Initialise all loaded mods, spawning script runners and
    /// runs the entry script for each mod.
    Init,

    /// Gracefully shuts down the thread running the Lua state.
    Shutdown,
}
