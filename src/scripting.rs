//! Scripting and user modification capabilities.
use std::{
    borrow::Borrow,
    collections::HashSet,
    fs::{canonicalize, File},
    io::prelude::*,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

use log::{trace, warn, Level};
use regex::Regex;
use rlua::Lua;
use serde::Deserialize;

mod data_definer;
pub mod errors;
pub mod prelude;
pub mod prototype;

use data_definer::{LuaDataDefiner, LuaDataDefinerRc};
use errors::ModError;
use prototype::{Prototype, PrototypeTable};

const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_MOD_META_FILENAME: &str = "mod.toml";
pub const DEFAULT_DATA_FILENAME: &str = "data.lua";
pub const DEFAULT_MOD_NAME_REGEX: &str = "^[a-zA-Z][a-zA-Z0-9_]+$";
pub const DEFAULT_PROTO_KEY_FIELD: &str = "name";

/// Container for mod data, event subscription registry and
/// scripting virtual machines.
pub struct Mods {
    mods: Vec<ModBundle>,
    prototypes: PrototypeTable<()>,
    settings: ModSettings,
}

impl Mods {
    /// Creates a new [`Mods`] instance that points to the given directory path.
    pub fn from_path<P>(mod_path: P) -> self::errors::Result<Self>
    where
        P: AsRef<Path>,
    {
        let mod_path = match canonicalize(mod_path.as_ref()) {
            Ok(p) => p,
            Err(io_err) => {
                return Err(ModError::ModDirectory(
                    mod_path.as_ref().to_path_buf(),
                    io_err,
                ))
            }
        };

        Ok(Mods {
            mods: vec![],
            prototypes: PrototypeTable::new(),
            settings: ModSettings {
                mod_path,
                max_search_depth: 2,
                mod_meta_filename: DEFAULT_MOD_META_FILENAME.to_string(),
                mod_data_filename: DEFAULT_DATA_FILENAME.to_string(),
                mod_name_re: Regex::new(DEFAULT_MOD_NAME_REGEX).unwrap(),
                prototype_key_field: DEFAULT_PROTO_KEY_FIELD.to_string(),
            },
        })
    }

    pub fn register_prototype<'de, T>(&mut self)
    where
        T: 'static + Prototype + Deserialize<'de>,
    {
        self.prototypes.register::<T>();
    }

    /// Walks the mod path and loads all mods discovered metadata files.
    ///
    /// Instantiates a Lua VM for each registered mod. Does not execute
    /// any scripts yet. See [`Mods::data_stage`].
    pub fn load_mods(&mut self) -> self::errors::Result<()> {
        if log::max_level() >= Level::Trace {
            trace!("Loading mods {}", self.settings.mod_path.to_string_lossy());
        }

        // Search for mod definition files.
        let walker =
            WalkDir::new(&self.settings.mod_path).max_depth(self.settings.max_search_depth);

        // Buffer to read file contents into.
        let mut buf = vec![];

        // Mod names need to be kept unique.
        let mut seen_names: HashSet<String> = HashSet::new();

        // Temporary buffer to hold loaded mods, before being ordered according to dependency.
        let mut mods: Vec<ModBundle> = vec![];

        for entry in walker {
            let entry = entry.unwrap();

            if Mods::directory_is_hidden(&entry) {
                continue;
            }

            if entry.path().file_name().unwrap() == self.settings.mod_meta_filename.as_str() {
                let file_path = canonicalize(entry.path()).unwrap();
                let dir_path = file_path.parent().unwrap();

                if !file_path.is_file() {
                    warn!("Mod {:?} is not a file", dir_path);
                    continue;
                }

                if log::max_level() >= Level::Trace {
                    trace!("Discovered mod at {}", dir_path.to_string_lossy());
                }

                // Load metadata
                let mut file = File::open(&file_path).unwrap();
                buf.clear();
                file.read_to_end(&mut buf).unwrap();

                // Load Definition
                let meta: ModMetaModel = toml::from_slice(&buf).unwrap();

                // Validations
                if !self.validate_name(&meta.name) {
                    return Err(ModError::ModNameInvalid(meta.name));
                }

                if seen_names.contains(&meta.name) {
                    return Err(ModError::ModNameTaken(meta.name));
                }
                seen_names.insert(meta.name.clone());

                mods.push(ModBundle {
                    meta: ModMeta {
                        id: ModId::none(),
                        name: meta.name,
                        path: dir_path.to_path_buf(),
                    },
                    lua: Mods::create_lua(),
                    prototypes: prototype::PrototypeTable::new(),
                });
            }
        }

        // TODO: Load order and IDs by dependencies.
        for (index, mod_bundle) in mods.iter_mut().enumerate() {
            mod_bundle.meta.id = ModId(index);
        }

        self.mods = mods;

        trace!("Loading mods Done");

        Ok(())
    }

    /// Retrieve a reference to a mod.
    pub fn get<K>(&self, id: &K) -> Option<&ModBundle>
    where
        K: Borrow<ModId>,
    {
        self.mods.get(id.borrow().inner())
    }

    /// Retrieve a mutable reference to a mod.
    pub fn get_mut<K>(&mut self, id: &K) -> Option<&mut ModBundle>
    where
        K: Borrow<ModId>,
    {
        self.mods.get_mut(id.borrow().inner())
    }

    /// Execute the data definition stage on all registered mods.
    ///
    /// # Errors
    ///
    /// Returns [`ModError::LuaError`](enum.ModError.html) if a script fails. Since there are
    /// multiple scripts being executed from multiple mods, a failure could
    /// leave the passed in `data_definer` in an inconsistent state.
    ///
    /// It's best to discard any partial definitions on error.
    pub fn data_stage(&mut self) -> self::errors::Result<()> {
        trace!("Mod data define stage pass start");
        let lua = Mods::create_lua();
        Mods::load_builtins(&lua)?;

        // Buffer for file contents.
        let mut buf = vec![];

        let mut data_definer_rc = LuaDataDefinerRc::new(LuaDataDefiner::new(
            &lua,
            self.settings.prototype_key_field.clone(),
        )?);

        let result: rlua::Result<()> = lua.context(|lua_ctx| {
            lua_ctx.scope(|scope| {
                let globals = lua_ctx.globals();
                let user_data = scope.create_nonstatic_userdata(data_definer_rc.clone())?;
                globals.set("data", user_data)?;

                for mod_bundle in &self.mods {
                    let walker = WalkDir::new(&mod_bundle.meta.path);
                    for entry in walker {
                        let entry = entry.unwrap();
                        let file_path = canonicalize(entry.path()).unwrap();

                        if file_path.file_name().unwrap()
                            != self.settings.mod_data_filename.as_str()
                        {
                            continue;
                        }

                        // TODO: Handle file error
                        let mut file = File::open(&file_path).unwrap();
                        buf.clear();
                        file.read_to_end(&mut buf).unwrap();

                        if log::max_level() >= Level::Trace {
                            trace!(
                                "Executing data definitions at {}",
                                file_path.to_string_lossy()
                            );
                        }

                        data_definer_rc.borrow_mut().prime_mod(&mod_bundle.meta);
                        lua_ctx.load(&buf).exec()?;
                    }
                }

                // Extract definitions
                let mod_table: rlua::Table =
                    lua_ctx.registry_value(&data_definer_rc.borrow().table_key)?;

                for pair in mod_table.pairs() {
                    let (mod_name, categories): (String, rlua::Table) = pair?;
                    println!("mod_name {}", mod_name);
                    for pair in categories.pairs() {
                        let (category_name, proto_definitions): (String, rlua::Table) = pair?;
                        println!("category_name {}", category_name);

                        for pair in proto_definitions.pairs() {
                            let (proto_name, proto_value): (String, rlua::Value) = pair?;
                            // println!(
                            //     "definition {} {} {}",
                            //     mod_name,
                            //     def_name,
                            //     def.get::<_, String>("name")?
                            // );

                            let key = format!("{}:{}:{}", mod_name, category_name, proto_name);
                            println!("Registering prototype {}", key);
                            self.prototypes.insert(
                                category_name.as_str(),
                                key.as_str(),
                                proto_value,
                            );
                        }
                    }
                }

                Ok(())
            })
        });
        result?;

        trace!("Mod data define stage pass done");

        Ok(())
    }

    /// Unload all mods in this registry.
    pub fn clear(&mut self) {
        self.mods.clear();
    }

    /// Utility to check whether a directory is intended to be hidden.
    fn directory_is_hidden(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }

    /// Checks whether a mod name is valid.
    ///
    /// The name should be usable by Lua as an identifier.
    fn validate_name<S>(&self, name: S) -> bool
    where
        S: AsRef<str>,
    {
        self.settings.mod_name_re.is_match(name.as_ref())
    }

    /// Creates a new Lua state instance.
    fn create_lua() -> Lua {
        rlua::Lua::new()
    }

    pub fn load_builtins(lua: &rlua::Lua) -> rlua::Result<()> {
        lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();
            globals.set("ENGINE_VERSION", ENGINE_VERSION)?;

            let deep_copy_src: &[u8] = include_bytes!("scripting/builtins/deepcopy.lua");
            lua_ctx.load(&deep_copy_src).exec()?;

            Ok(())
        })
    }
}

/// Global settings for mod system.
pub struct ModSettings {
    /// Absolute path to the mod folder.
    pub mod_path: PathBuf,

    /// Maximum directory depth to travel when searching for files.
    pub max_search_depth: usize,

    /// Filename for mod metadata file.
    pub mod_meta_filename: String,

    /// Filename for mod data definition script file.
    pub mod_data_filename: String,

    /// Regular expression used for validating mod names.
    pub mod_name_re: Regex,

    /// Name of the table field to use when extracting prototype identifiers.
    pub prototype_key_field: String,
}

/// Information describing a mod.
pub struct ModMeta {
    id: ModId,

    name: String,

    /// Path to the directory where the mod was found.
    path: PathBuf,
}

/// Meta file found at the top level of a mod's folder.
#[derive(Deserialize)]
pub struct ModMetaModel {
    name: String,
    version: String,
    author: String,
    email: Option<String>,
    website: Option<String>,
    dependencies: Vec<String>,
}

pub struct ModBundle {
    meta: ModMeta,
    lua: rlua::Lua,
    pub prototypes: prototype::PrototypeTable<()>,
    // TODO: event subscriptions
}

/// Identifier for registered mods.
///
/// Generated by [`Mods`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct ModId(usize);

impl ModId {
    /// Constructs a [`ModId`] with an invalid inner value.
    ///
    /// Used as metadata id while loading mods, before sorting.
    #[inline]
    fn none() -> Self {
        ModId(::std::usize::MAX)
    }

    #[inline]
    pub fn inner(&self) -> usize {
        self.0
    }
}

impl Into<usize> for ModId {
    fn into(self) -> usize {
        self.0
    }
}
