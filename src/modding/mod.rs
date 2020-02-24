use crate::errors;
use crate::intern::{intern, InternedStr};
use crate::sync::ChannelPair;
use crossbeam::{channel, channel::SendError};
use log::{error, trace, warn};
use rlua::Lua;
use serde::Deserialize;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{canonicalize, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::thread;
use toml;
use walkdir::{DirEntry, WalkDir};
// use std::clone::Clone;

mod cmd;
mod runner;
mod validate;

pub use cmd::*;

pub const DEFAULT_LIB_NAME: &str = "core";
pub const DEFAULT_MOD_PATH: &str = "./mods";
pub const DEFAULT_MOD_DEF: &str = "mod.toml";
pub const DEFAULT_ENTRY_FILE: &str = "init.lua";

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// World level resource that contains a mapping of
/// mod keys to mod meta objects.
#[allow(dead_code)]
pub struct Mods {
    mods: BTreeMap<InternedStr, ModMeta>,

    /// Execution order of mods
    order: Vec<InternedStr>,

    /// Name of the library table provided to
    /// loaded mod's Lua code.
    lib_name: InternedStr,

    /// Path to the mod folder.
    mod_path: PathBuf,
}

#[allow(dead_code)]
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
    website: Option<InternedStr>,

    /// Entry point Lua filename.
    entry: InternedStr,

    /// List of mod names that must be
    /// executed before this mod executes.
    depends_on: Vec<InternedStr>,

    /// Indicates that the user intends to initialise
    /// the mod when starting up the main game scene.
    enabled: bool,

    /// Channel for sending and receiving a command buffer, to
    /// and from the script runner.
    ///
    /// Use this to comminucate with the script runner.
    hub: ChannelPair<Vec<ModCmd>>,

    /// Clone this to a script runner when one is spawned.
    chan: ChannelPair<Vec<ModCmd>>,

    /// Handle to the script runner thread, which can be joined on when
    /// shutting down gracefully.
    ///
    /// Optional because threads are only spawned during mod initialisation,
    /// after mod loading.
    join: Option<ScriptRunnerHandle>,

    /// Stream of errors that occurred inside a script runner's thread.
    errors: (
        channel::Sender<errors::Error>,
        channel::Receiver<errors::Error>,
    ),

    /// Stream of outgoing commands that have been sent by script runners
    /// during script execution.
    script_cmds: (channel::Sender<u32>, channel::Receiver<u32>),
}

/// Meta file found at the top level of a mod's folder.
#[derive(Deserialize)]
struct ModMetaModel {
    name: String,
    version: String,
    author: String,
    email: Option<String>,
    website: Option<String>,
}

type ScriptRunnerHandle = thread::JoinHandle<errors::Result<()>>;

impl Mods {
    pub fn new(lib_name: &'static str, mod_path: &Path) -> Self {
        Mods {
            mods: BTreeMap::new(),
            order: Vec::new(),
            lib_name: intern(lib_name),
            mod_path: mod_path.to_path_buf(),
        }
    }

    /// Walks the target mod path and loads the metadata files.
    ///
    /// Fails if the mod folder does not exist, if a mod meta data
    /// file is misformed, or fails to load.
    pub fn load_mods(&mut self) -> errors::Result<()> {
        trace!("Loading mods");

        // Search for mod definition file
        let walker = WalkDir::new(&self.mod_path).max_depth(2);

        for entry in walker {
            let entry = entry.unwrap();

            if is_hidden(&entry) {
                continue;
            }

            if entry.path().file_name().unwrap() == DEFAULT_MOD_DEF {
                let file_path = canonicalize(entry.path()).unwrap();
                let dir_path = file_path.parent().unwrap();
                let mod_name = intern(dir_path.iter().last().unwrap().to_str().unwrap());

                // TODO: Validate string values
                if !validate::mod_name(mod_name.as_ref()) {
                    error!("Invalid mod name '{}'", mod_name.as_ref());
                    return Err(errors::ErrorKind::ModLoad.into());
                }

                if !file_path.is_file() {
                    warn!("Mod {:?} is not a file", dir_path);
                    continue;
                }
                trace!("Found mod in {:?}", dir_path);

                // Load Data
                let mut file = File::open(&file_path)?;
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)?;

                // Load Definition
                let meta: ModMetaModel = toml::from_slice(&contents)?;

                // Construct Key
                let id = intern(&format!("{}:{}", mod_name.as_ref(), meta.version));

                if let Entry::Vacant(e) = self.mods.entry(id) {
                    let (hub_chan, mod_chan) = ChannelPair::create();
                    let error_chan = channel::unbounded();
                    let script_cmds_chan = channel::unbounded();

                    e.insert(ModMeta {
                        id,
                        path: dir_path.to_path_buf(),
                        name: intern(&meta.name),
                        version: intern(&meta.version),
                        author: intern(&meta.author),
                        email: meta.email.map(|ref s| intern(s)),
                        website: meta.website.map(|ref s| intern(s)),
                        entry: intern(DEFAULT_ENTRY_FILE),
                        depends_on: Vec::new(),
                        enabled: false,
                        hub: hub_chan,
                        chan: mod_chan,
                        join: None,
                        errors: error_chan,
                        script_cmds: script_cmds_chan,
                    });
                }
            }
        }

        Ok(())
    }

    /// Initialise loaded mods.
    ///
    /// Runs the initial Lua file of each mod or modpack.
    pub fn init_mods(&mut self, api: fn(&mut rlua::Lua, ScriptChannel)) -> errors::Result<()> {
        for (_id, meta) in self.mods.iter_mut() {
            // TODO: Avoid string copy.
            let lib_name = self.lib_name.as_ref().to_owned();
            let chan = meta.chan.clone();
            let init_script = meta.path.join(meta.entry.as_ref());
            let error_sender = meta.errors.0.clone();
            let cmds_sender = meta.script_cmds.0.clone();

            meta.join = Some(
                thread::Builder::new()
                    .name("mod:0.0.0".to_string())
                    .spawn(move || {
                        // Engine scripting interface
                        let mut lua = create_interface(lib_name.as_ref())?;

                        // Game scripting interface
                        api(&mut lua, ScriptChannel(cmds_sender));

                        let mut runner = self::runner::ScriptRunner {
                            lua,
                            chan,
                            init_script,
                            lib_name: lib_name.clone(),
                            errors: error_sender,
                        };

                        // Run until shutdown
                        runner.run();

                        Ok(())
                    })
                    .unwrap(),
            );
        }

        let (_in_cmds, out_cmds) = self.dispatch(vec![cmd::ModCmd::Init])?;
        if out_cmds.is_some() {
            warn!("Dispatching commands during initialization is not supported.");
            if let Some(mut cmds) = out_cmds {
                for cmd in cmds.drain(..) {
                    println!("  {:?}", cmd);
                }
            }
        }

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
    pub fn define_entity<F>(&mut self, _f: F)
    where
        F: Fn(),
    {
        unimplemented!()
    }

    /// Gracefully shuts down all script runners.
    ///
    /// Will return an error if any of the script
    /// runners returns an error its during shutdown.
    ///
    /// Can also return an error if one or more threads
    /// panic.
    pub fn shutdown(&mut self) -> errors::Result<()> {
        let result = self.dispatch(vec![cmd::ModCmd::Shutdown]);
        if let Ok((_, Some(_))) = result {
            warn!("Dispatching commands during shutdown is not supported.");
        }

        let mut errors: Option<Vec<errors::Error>> = None;

        for (_, meta) in self.mods.iter_mut() {
            if let Some(handle) = meta.join.take() {
                match handle.join() {
                    Ok(r) => {
                        if let Err(e) = r {
                            errors.get_or_insert_with(|| vec![]).push(e);
                        }
                    }
                    Err(_) => errors
                        .get_or_insert_with(|| vec![])
                        .push(errors::ErrorKind::ModScriptThread.into()),
                }
            }
        }

        // Return script errors after threads are shutdown.
        result?;

        Ok(())
    }

    /// Dispatches a scene lifetime hook to all mods.
    pub fn scene_hook(&mut self, hook: SceneHook) -> errors::Result<Option<Vec<u32>>> {
        self.dispatch(vec![ModCmd::Scene(hook)])
            // Discard in command buffer.
            .map(|(_in_cmds, out_cmds)| out_cmds)
    }

    /// Executes all mods, passing the given command buffer
    /// to all script runners. Blocks on each script runner
    /// waiting for the buffer to be returned.
    fn dispatch(
        &mut self,
        mut in_cmds: Vec<cmd::ModCmd>,
    ) -> errors::Result<(Vec<cmd::ModCmd>, Option<Vec<u32>>)> {
        // Lazy instantiated vectors
        let mut errors: Option<Vec<errors::Error>> = None;
        let mut out_cmds: Option<Vec<u32>> = None;

        for (_id, meta) in self.mods.iter_mut() {
            // Ownership of the command buffer is passed
            // to script runner thread and returned on
            // each iteration.
            in_cmds = match meta.hub.send(in_cmds) {
                Ok(_) => match meta.hub.receive() {
                    Ok(v) => v,
                    // If the receiver is closed, then
                    // we've lost the command buffer.
                    Err(_) => return Err(errors::ErrorKind::ModDispatch.into()),
                },
                // If the channel is full, the command
                // buffer is returned.
                Err(SendError(v)) => v,
            };

            // Gather possible errors
            while let Ok(err) = meta.errors.1.try_recv() {
                errors.get_or_insert_with(|| vec![]).push(err);
            }

            // Gather outgoing commands
            while let Ok(cmd) = meta.script_cmds.1.try_recv() {
                out_cmds.get_or_insert_with(|| vec![]).push(cmd);
            }
        }

        in_cmds.clear();

        if let Some(e) = errors {
            Err(errors::ErrorKind::ModComposite(e).into())
        } else {
            Ok((in_cmds, out_cmds))
        }
    }
}

impl Default for Mods {
    fn default() -> Self {
        Mods {
            mods: BTreeMap::new(),
            order: Vec::new(),
            lib_name: intern(DEFAULT_LIB_NAME),
            mod_path: PathBuf::from(DEFAULT_MOD_PATH),
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

#[derive(Clone)]
pub struct ScriptChannel(pub(crate) channel::Sender<u32>);

impl ScriptChannel {
    pub fn send(&mut self, message: u32) {
        // TODO: Properly handle error.
        self.0.send(message).expect("Script channel send failure");
    }
}

fn create_interface(lib_name: &str) -> errors::Result<Lua> {
    use rlua::Table;
    let lua = Lua::new();

    let result = lua.context(|lua_ctx| {
        let register_entity =
            lua_ctx.create_function(|_, (_name, _t): (rlua::String, Table)| {
                println!("register_entity()");

                Ok(())
            })?;

        let lib = lua_ctx.create_table()?;
        lib.set("version", VERSION)?;
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

/// Avoid hidden unix files
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}
