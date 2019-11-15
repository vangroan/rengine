use crate::errors;
use crate::intern::{intern, InternedStr};
use crossbeam::{
    channel,
    channel::{RecvError, SendError},
};
use log::{trace, warn};
use rlua::Lua;
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::thread;

pub const DEFAULT_LIB_NAME: &str = "core";
pub const DEFAULT_MOD_PATH: &str = "./mods";
pub const DEFAULT_ENTRY_FILE: &str = "init.lua";

/// World level resource that contains a mapping of
/// mod keys to mod meta objects.
pub struct Mods {
    mods: BTreeMap<InternedStr, ModMeta>,

    /// Execution order of mods
    order: Vec<InternedStr>,

    /// Name of the library table provided to
    /// loaded mod's Lua code.
    lib_name: InternedStr,

    /// Path to the mod folder.
    mod_path: PathBuf,

    /// Communication channel to script runner threads.
    hub: ChannelPair,

    /// Copy of other end of channels. To be cloned
    /// and passed to script runner threads, whenever a
    /// new mod is initialized.
    mod_channel: ChannelPair,
}

pub struct ModMeta {
    /// Unique identifier for this mod, a combination
    /// of the directory name and version of the mod.
    id: InternedStr,

    /// Path to the directory where the mod was found.
    path: PathBuf,

    /// Human readable mod name, meant to
    /// be displayed in a UI.
    name: InternedStr,

    /// Semantic version of the mod.
    version: InternedStr,

    /// Name of mod author.
    author: InternedStr,

    /// Optional contact email address of mod author.
    email: Option<InternedStr>,

    /// Optional website address for mod.
    webstie: Option<InternedStr>,

    /// Entry point Lua filename.
    entry: InternedStr,

    /// List of mod names that must be
    /// executed before this mod executes.
    depends_on: Vec<InternedStr>,

    /// Indicates that the user intends to initialise
    /// the mod when starting up the main game scene.
    enabled: bool,

    /// Incoming command buffer.
    sender: channel::Sender<Vec<ModCmd>>,

    /// Outgoing result buffer.
    receiver: channel::Receiver<Vec<ModCmd>>,

    /// Handle to the script runner thread, which can be joined on when
    /// shutting down gracefully.
    ///
    /// Optional because threads are only spawned during mod initialisation,
    /// after mod loading.
    join: Option<ScriptRunnerHandle>,
    // Lua state is specific to each mod.
    // lua: Lua,
}

enum ModCmd {
    /// Gracefully shuts down the thread running the Lua state.
    Shutdown,
}

#[derive(Clone)]
struct ChannelPair {
    sender: channel::Sender<Vec<ModCmd>>,
    receiver: channel::Receiver<Vec<ModCmd>>,
}

type ModCmdChannel = (channel::Sender<Vec<ModCmd>>, channel::Receiver<Vec<ModCmd>>);

type ScriptRunnerHandle = thread::JoinHandle<errors::Result<()>>;

struct ScriptRunner {
    lua: Lua,
    chan: ChannelPair,
}

impl Mods {
    pub fn new(lib_name: &'static str, mod_path: &Path) -> Self {
        let (hub_chan, mod_chan) = ChannelPair::create();

        Mods {
            mods: BTreeMap::new(),
            order: Vec::new(),
            lib_name: intern(lib_name),
            mod_path: mod_path.to_owned(),
            hub: hub_chan,
            mod_channel: mod_chan,
        }
    }

    /// Walks the target mod path and loads the metadata files.
    ///
    /// Fails if the mod folder does not exist, if a mod meta data
    /// file is misformed, or fails to load.
    pub fn load_mods(&mut self) -> errors::Result<()> {
        trace!("Loading mods");
        // TODO: Load mods
        Ok(())
    }

    /// Initialise loaded mods.
    ///
    /// Runs the initial Lua file of each mod or modpack.
    pub fn init_mods(&self) -> errors::Result<()> {
        // TODO: Do this for each loaded mod.
        let lib_name = self.lib_name.as_ref().to_owned();
        let chan = self.mod_channel.clone();
        let join: ScriptRunnerHandle = thread::Builder::new()
            .name("mod:0.0.0".to_string())
            .spawn(move || {
                let lua = create_interface(lib_name)?;
                let mut runner = ScriptRunner { lua, chan };

                // Run until shutdown
                runner.run();

                Ok(())
            })
            .unwrap();

        Ok(())
    }

    /// Registers a closure that is called whenever
    /// a mod defines an entity.
    ///
    /// The closure must ensure that the entity definition is
    /// valid, according to the game's needs, and return an
    /// error when a mod attempts to define an entity with invalid
    /// components.
    ///
    /// ## Thoughts
    ///
    /// Maybe this should rather live in an entity definition trait.
    ///
    /// ```ignore
    /// struct CollectableDef {
    ///     sprite: ComponentDef<Billboard>,
    ///     item: ComponentDef<Item>,
    ///     transform: ComponentDef<Transform>,
    /// }
    /// ```
    pub fn define_entity<F>(&mut self, f: F)
    where
        F: Fn(),
    {
        unimplemented!()
    }

    pub fn shutdown(&mut self) {
        for (_, meta) in self.mods.iter_mut() {
            if let Some(handle) = meta.join.take() {
                handle.join().expect("script runner panic").unwrap();
            }
        }
    }

    /// Executes all mods, passing the given command buffer
    /// to all script runners. Blocks on each script runner
    /// waiting for the buffer to be returned.
    fn dispatch(&self, mut cmds: Vec<ModCmd>) -> Vec<ModCmd> {
        cmds.clear();
        cmds
    }
}

impl Default for Mods {
    fn default() -> Self {
        let (hub_chan, mod_chan) = ChannelPair::create();

        Mods {
            mods: BTreeMap::new(),
            order: Vec::new(),
            lib_name: intern(DEFAULT_LIB_NAME),
            mod_path: PathBuf::from(DEFAULT_MOD_PATH),
            hub: hub_chan,
            mod_channel: mod_chan,
        }
    }
}

impl fmt::Display for ModMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Mod({})", self.id)
    }
}

impl Drop for ModMeta {
    fn drop(&mut self) {
        if let Some(handle) = self.join.take() {
            warn!(
                "Mod {} dropped, but script runner not shutdown",
                self.id.as_ref()
            );

            handle.join().expect("script runner thread panic").unwrap();
        }
    }
}

impl ChannelPair {
    /// Creates two channel pairs, that are linked to eachother.
    fn create() -> (Self, Self) {
        // Unbounded channel will block on both
        // send and receive, until the other end
        // is ready.
        let (a_send, b_recv): ModCmdChannel = channel::bounded(0);
        let (b_send, a_recv): ModCmdChannel = channel::bounded(0);

        let a = ChannelPair {
            sender: a_send,
            receiver: a_recv,
        };
        let b = ChannelPair {
            sender: b_send,
            receiver: b_recv,
        };

        return (a, b);
    }

    fn send(&mut self, val: Vec<ModCmd>) -> Result<(), SendError<Vec<ModCmd>>> {
        self.sender.send(val)
    }

    fn receive(&mut self) -> Result<Vec<ModCmd>, RecvError> {
        self.receiver.recv()
    }
}

impl ScriptRunner {
    fn run(&mut self) {
        let mut running = true;

        'main: while running {
            // Receive command buffer from hub
            match self.chan.receive() {
                Ok(mut cmds) => {
                    running = self.on_receive(&mut cmds);

                    // Send back to hub
                    self.chan.send(cmds).unwrap();
                }
                _ => {}
            }
        }
    }

    fn on_receive(&mut self, cmds: &Vec<ModCmd>) -> bool {
        use ModCmd::*;

        for cmd in cmds.iter() {
            match cmd {
                Shutdown => return false,
                _ => {}
            }
        }

        return true;
    }
}

fn create_interface(lib_name: String) -> errors::Result<Lua> {
    use rlua::{self, Table};
    let lua = Lua::new();

    let result = lua.context(|lua_ctx| {
        let register_entity =
            lua_ctx.create_function(|_, (_name, _t): (rlua::String, Table)| {
                println!("register_entity()");

                Ok(())
            })?;

        let lib = lua_ctx.create_table()?;
        lib.set("version", "0.0.0")?;
        lib.set("register_entity", register_entity)?;

        let globals = lua_ctx.globals();
        globals.set(lib_name, lib)?;

        Ok(())
    });

    match result {
        Ok(_) => Ok(lua),
        Err(err) => Err(errors::ErrorKind::Lua(err).into()),
    }
}
