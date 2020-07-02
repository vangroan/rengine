//! Mod and scripting errors.
use std::path::PathBuf;

/// Specialised `Result` for the scripting API.
pub type Result<T> = std::result::Result<T, ModError>;

#[derive(Debug)]
pub enum ModError {
    /// IO error accessing mod directory.
    ModDirectory(PathBuf, std::io::Error),

    /// Mod name is not unique in the mod registry.
    ModNameTaken(String),

    /// Mod name failed validation check.
    ModNameInvalid(String),

    /// Failure operating on file.
    IoError(std::io::Error),

    /// Error in Lua state or script.
    LuaError(rlua::Error),
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
            IoError(_) => write!(f, "mod file error"),
            LuaError(_) => write!(f, "error in Lua script"),
        }
    }
}

impl std::error::Error for ModError {
    fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
        use ModError::*;
        match self {
            ModDirectory(_, err) => Some(err),
            IoError(err) => Some(err),
            LuaError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<rlua::Error> for ModError {
    fn from(lua_err: rlua::Error) -> Self {
        ModError::LuaError(lua_err)
    }
}
