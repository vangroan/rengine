//! Interface for registering prototype definitions.
pub use std::cell::{Ref, RefMut};
use std::{cell::RefCell, ops::Deref, rc::Rc};

use rlua::{Lua, Table, UserData, UserDataMethods, Value};

use crate::scripting::ModMeta;

pub struct LuaDataDefiner {
    /// Name of the current mod in the data pass.
    ///
    /// Register calls will use this name as the key in the data table.
    pub current_mod_name: Option<String>,
    /// Table that contains all the definitions.
    pub table_key: rlua::RegistryKey,
}

impl LuaDataDefiner {
    pub fn new(lua: &Lua) -> rlua::Result<Self> {
        let table_key = lua.context(|lua_ctx| {
            let table = lua_ctx.create_table()?;
            Ok(lua_ctx.create_registry_value(table)?)
        })?;

        Ok(LuaDataDefiner {
            current_mod_name: None,
            table_key: table_key,
        })
    }

    /// Set the current mod to prime the definer for loading definitions for that specific mod.
    #[inline]
    pub fn prime_mod(&mut self, mod_meta: &ModMeta) {
        self.current_mod_name = Some(mod_meta.name.clone());
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
        methods.add_method("register", |lua_ctx, definer_rc, data: Table| {
            let data_definer = definer_rc.borrow();
            let mod_name = data_definer
                .current_mod_name
                .as_ref()
                .expect("data definer register called, but not primed with mod")
                .as_str();
            let data_table = lua_ctx.registry_value::<Table>(&data_definer.table_key)?;

            for pair in data.pairs() {
                let (_, value): (i32, rlua::Table) = pair?;
                println!("registering {}", mod_name);

                match data_table.get::<_, Value>(mod_name)? {
                    Value::Nil => {
                        println!("value nil");
                        let t = lua_ctx.create_table()?;
                        // TODO: Definition validation
                        t.set(value.get::<_, String>("name")?, value)?;
                    }
                    Value::Table(t) => {
                        println!("value table");
                        t.set(value.get::<_, String>("name")?, value)?;
                    }
                    _ => { /* unsupported */ }
                }
            }

            Ok(())
        });
    }
}
