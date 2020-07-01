//! Scripting and user modification capabilities.
use std::{
    borrow::Borrow,
    collections::HashSet,
    fs::{canonicalize, File},
    io::prelude::*,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

use log::{max_level, trace, warn, Level};
use regex::Regex;
use rlua::Lua;
use serde::Deserialize;

const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_MOD_META_FILENAME: &str = "mod.toml";
pub const DEFAULT_MOD_NAME_REGEX: &str = "^[a-zA-Z][a-zA-Z0-9_]+$";

/// Container for mod data, event subscription registry and
/// scripting virtual machines.
pub struct Mods {
    mods: Vec<ModBundle>,
    settings: ModSettings,
}

impl Mods {
    /// Creates a new [`Mods`] instance that points to the given directory path.
    pub fn from_path<P>(mod_path: P) -> self::Result<Self>
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
            settings: ModSettings {
                mod_path,
                max_search_depth: 2,
                mod_meta_filename: DEFAULT_MOD_META_FILENAME.to_string(),
                mod_name_re: Regex::new(DEFAULT_MOD_NAME_REGEX).unwrap(),
            },
        })
    }

    /// Walks the mod path and loads all mods discovered metadata files.
    ///
    /// Instantiates a Lua VM for each registered mod. Does not execute
    /// any scripts yet. See [`Mods::data_stage`].
    pub fn load_mods(&mut self) -> self::Result<()> {
        if max_level() >= Level::Trace {
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

                if max_level() >= Level::Trace {
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

                // TODO: Load order and IDs by dependencies.
                self.mods.push(ModBundle {
                    meta: ModMeta {
                        id: ModId::none(),
                        name: meta.name,
                        path: dir_path.to_path_buf(),
                    },
                    lua: Mods::create_lua(),
                });
            }
        }

        trace!("Loading mods Done");

        Ok(())
    }

    /// Retrieve a reference to a mod.
    pub fn get<K>(&self, id: &K) -> Option<&ModBundle>
    where
        K: Borrow<ModId>,
    {
        unimplemented!()
    }

    /// Retrieve a mutable reference to a mod.
    pub fn get_mut<K>(&self, id: &K) -> Option<&mut ModBundle>
    where
        K: Borrow<ModId>,
    {
        unimplemented!()
    }

    /// Execute the data definition stage on all registered mods.
    pub fn data_stage(&self) {
        unimplemented!()
    }

    /// Unload all mods in this registry.
    pub fn clear(&mut self) {
        unimplemented!()
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

    fn create_lua() -> Lua {
        rlua::Lua::new()
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

    /// Regular expression used for validating mod names.
    pub mod_name_re: Regex,
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
struct ModMetaModel {
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
    fn none() -> Self {
        ModId(::std::usize::MAX)
    }

    pub fn inner(&self) -> usize {
        self.0
    }
}

impl Into<usize> for ModId {
    fn into(self) -> usize {
        self.0
    }
}

pub type Result<T> = std::result::Result<T, ModError>;

#[derive(Debug)]
pub enum ModError {
    /// IO error accessing mod directory.
    ModDirectory(PathBuf, std::io::Error),

    /// Mod name is not unique in the mod registry.
    ModNameTaken(String),

    /// Mod name failed validation check.
    ModNameInvalid(String),
}

impl ::std::fmt::Display for ModError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> ::std::fmt::Result {
        use ModError::*;
        match self {
            ModDirectory(path, _) => {
                write!(f, "error accessing mod folder {}", path.to_string_lossy())
            }
            ModNameTaken(name) => write!(f, "mod with name '{}' already exists", name),
            ModNameInvalid(name) => write!(f, "mod name '{}' is invalid", name),
        }
    }
}

impl std::error::Error for ModError {
    fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
        use ModError::*;
        match self {
            ModDirectory(_, src) => Some(src),
            _ => None,
        }
    }
}
