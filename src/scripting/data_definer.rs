//! Interface for registering prototype definitions.
pub use std::cell::{Ref, RefMut};
use std::{cell::RefCell, rc::Rc};

use log::trace;
use rlua::{Context, Lua, RegistryKey, Table, UserData, UserDataMethods, Value};

use crate::scripting::{ModId, ModMeta};

pub struct LuaDataDefiner {
    /// Name of table field in the prototype to extract
    /// a value and use as an identifier.
    pub key_field: String,

    /// Name and id of the current mod in the data pass.
    ///
    /// Register calls will use this name as the key in the data table.
    pub current_mod: Option<(ModId, String)>,

    /// Table that contains all the definitions.
    pub table_key: rlua::RegistryKey,
}

impl LuaDataDefiner {
    pub fn new<S>(lua: &Lua, key_field: S) -> rlua::Result<Self>
    where
        S: ToString,
    {
        let table_key: RegistryKey = lua.context(|lua_ctx: Context<'_>| {
            let table: Table = lua_ctx.create_table()?;
            lua_ctx.create_registry_value(table)
        })?;

        Ok(LuaDataDefiner {
            key_field: key_field.to_string(),
            current_mod: None,
            table_key,
        })
    }

    /// Set the current mod to prime the definer for loading definitions for that specific mod.
    #[inline]
    pub fn prime_mod(&mut self, mod_meta: &ModMeta) {
        self.current_mod = Some((mod_meta.id, mod_meta.name.clone()));
    }
}

/// `UserData` reference to a [`LuaDataDefiner`](struct.LuaDataDefiner.html) allowing it
/// to be borrowed by a Lua context.
#[derive(Clone)]
pub struct LuaDataDefinerRc(Rc<RefCell<LuaDataDefiner>>);

impl LuaDataDefinerRc {
    pub fn new(data_definer: LuaDataDefiner) -> Self {
        LuaDataDefinerRc(Rc::new(RefCell::new(data_definer)))
    }

    pub fn borrow(&self) -> Ref<'_, LuaDataDefiner> {
        self.0.borrow()
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, LuaDataDefiner> {
        self.0.borrow_mut()
    }
}

impl UserData for LuaDataDefinerRc {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method(
            "extend",
            |lua_ctx, definer_rc, (type_name, definitions): (String, Table)| {
                let data_definer = definer_rc.borrow();
                let key_field = data_definer.key_field.as_str();
                let (mod_id, mod_name) = data_definer
                    .current_mod
                    .as_ref()
                    .map(|(id, name)| (id, name.as_str()))
                    .expect("data definer register called, but not primed with mod");
                let data_table = lua_ctx.registry_value::<Table>(&data_definer.table_key)?;

                // Sequence of definitions.
                for proto_table in definitions.sequence_values::<Table>() {
                    let proto_table = proto_table?;
                    let proto_name: String = proto_table.get(key_field)?;
                    trace!("mod_name {}", mod_name);

                    // Prototypes for the current mod
                    let mod_table = match data_table.get::<_, Value>(mod_name)? {
                        Value::Nil => {
                            // Lazily create a mod table.
                            let t = lua_ctx.create_table()?;

                            // TODO: Definition validation
                            data_table.set(mod_name, t.clone())?;
                            t
                        }
                        Value::Table(t) => t,
                        _ => {
                            /* unsupported */
                            panic!("mod table unsupported type");
                        }
                    };

                    // Prototype category table
                    let category_table = match mod_table.get(type_name.as_str())? {
                        Value::Nil => {
                            let t = lua_ctx.create_table()?;
                            mod_table.set(type_name.as_str(), t.clone())?;
                            t
                        }
                        Value::Table(t) => t,
                        _ => {
                            /* unsupported */
                            panic!("prototype definition table unsupported type");
                        }
                    };

                    if log::max_level() >= log::Level::Trace {
                        trace!(
                            "Data definition expanded with '{}:{}:{}'",
                            mod_name,
                            type_name,
                            proto_name
                        );
                    }
                    category_table.set(proto_name, proto_table)?;
                }

                Ok(())
            },
        );

        // Method for retrieving the global definition table.
        methods.add_method("table", |lua_ctx, definer_rc, ()| {
            let data_definer = definer_rc.borrow();
            let table: Table = lua_ctx.registry_value(&data_definer.table_key)?;
            Ok(table)
        });
    }
}
