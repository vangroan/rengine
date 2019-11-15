use crate::intern::{intern, InternedStr};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_LIB_NAME: &str = "core";
pub const DEFAULT_MOD_PATH: &str = "./mods";

/// World level resource that contains a mapping of
/// mod keys to mod meta objects.
#[derive(Debug)]
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

#[derive(Debug)]
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

    /// List of mod names that must be
    /// executed before this mod executes.
    depends_on: Vec<InternedStr>,
}

impl Mods {
    pub fn new(lib_name: &'static str, mod_path: &Path) -> Self {
        Mods {
            mods: BTreeMap::new(),
            order: Vec::new(),
            lib_name: intern(lib_name),
            mod_path: mod_path.to_owned(),
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
