//! Entity prototype definitions.
//!
//! # Examples
//!
//! ```
//! use std::borrow::Cow;
//! use serde::Deserialize;
//! use rlua;
//! use rengine::scripting::prelude::*;
//!
//! #[derive(Deserialize)]
//! struct GameActor {
//!     name: String,
//!     position: [f32; 2],
//!     health: i32,
//!     can_jump: bool,
//! }
//!
//! impl Prototype for GameActor {
//!     fn type_name<'a>() -> Cow<'a, str> {
//!         "game_actor".into()
//!     }
//! }
//!
//! let mut prototype_table: PrototypeTable = PrototypeTable::new();
//!
//! // Importantly: Register type first!
//! prototype_table.register::<GameActor>();
//!
//! let lua = rlua::Lua::new();
//! let result: rlua::Result<()> = lua.context(|lua_ctx| {
//!     // The definition is retrieved as `rlua::Value`
//!     let soldier_table = lua_ctx.load(r#"
//!         {
//!             name = 'foot_soldier',
//!             position = {100.0, 200.0},
//!             health = 100,
//!             can_jump = true,
//!         }
//!         "#).eval::<rlua::Value>()?;
//!
//!     // Note: The `type_name` argument must match the `type_name` in
//!     //       the struct `GameActor`;
//!     prototype_table.insert("game_actor", "my_mod:game_actor:foot_soldier", soldier_table);
//!
//!     Ok(())
//! });
//! # result.unwrap();
//! ```
//!
//! # Implementation
//!
//! Prototypes are stored in a separate boxed storage so the `Deserialized`
//! trait isn't needed for [`PrototypeTable::get`].
use std::{any::TypeId, borrow::Cow, collections::HashMap, marker::PhantomData};

use serde::Deserialize;

/// Trait for prototypes that can be declared in Rust and defined in Lua.
///
/// # Examples
///
/// ```
/// use std::borrow::Cow;
/// use serde::Deserialize;
/// use rengine::scripting::prelude::*;
///
/// // Define a type that is both `Prototype` and `Deserialize`.
/// #[derive(Deserialize)]
/// struct GameActor {
///     position: [f32; 2],
///     sprite: String,
/// }
///
/// impl Prototype for GameActor {
///     /// A string key for mod scripts to refer to this type.
///     fn type_name<'a>() -> Cow<'a, str> {
///         "game_actor".into()
///     }
/// }
/// ```
pub trait Prototype {
    fn type_name<'a>() -> Cow<'a, str>;
}

/// Trait for a container that maps prototype keys to definition intances.
///
/// Used for upcasting and boxing a concrete storage type in the [`PrototypeTable`](struct.PrototypeTable.html).
trait Storage: mopa::Any {}
mopafy!(Storage);

/// Concrete storage implementation of prototype storage.
///
/// Backed by a map of prototype keys to prototype definition instances.
///
/// Definitions from scripts live here.
pub struct PrototypeMapStorage<T: Prototype> {
    data: HashMap<String, T>,
}

impl<T> PrototypeMapStorage<T>
where
    T: Prototype,
{
    fn new() -> Self {
        PrototypeMapStorage {
            data: HashMap::new(),
        }
    }

    fn insert(&mut self, key: String, proto: T) {
        self.data.insert(key, proto);
    }

    fn get<S>(&self, key: S) -> Option<&T>
    where
        S: AsRef<str>,
    {
        self.data.get(key.as_ref())
    }
}

impl<T> Storage for PrototypeMapStorage<T> where T: 'static + Prototype {}

/// Trait for a factory that creates a concrete prototype from a dynamically typed Lua value.
///
/// When [`PrototypeTable::insert`](struct.PrototypeTable.html) is called from a context with no
/// knowledge of the prototype's concrete type, the factory dynamically dispatches to a concrete
/// implementation which can create an instance of the concrete type via deserialization.
///
/// Used for upcasting and boxing a concrete storage type in the [`PrototypeTable`](struct.PrototypeTable.html).
trait Factory: mopa::Any {
    fn insert_value<'lua>(&self, storage: &mut dyn Storage, key: String, value: rlua::Value<'lua>);
}
mopafy!(Factory);

/// Concrete factory implementation for creating prototypes from Lua values.
struct PrototypeFactory<T: Prototype> {
    _marker: PhantomData<T>,
}

impl<T> PrototypeFactory<T>
where
    T: Prototype,
{
    fn new() -> Self {
        PrototypeFactory {
            _marker: PhantomData,
        }
    }
}

impl<'de, T> Factory for PrototypeFactory<T>
where
    T: 'static + Prototype + Deserialize<'de>,
{
    /// Magic happens here.
    ///
    /// This factory knows the static type needed for deserialization.
    fn insert_value<'lua>(&self, storage: &mut dyn Storage, key: String, value: rlua::Value<'lua>) {
        let deserializer = rlua_serde::de::Deserializer { value };
        let proto = T::deserialize(deserializer).unwrap();

        // TODO: Error on existing?
        storage
            .downcast_mut::<PrototypeMapStorage<T>>()
            .expect("Unexpected storage type during downcast")
            .insert(key, proto);
    }
}

/// Bundle of a factory and storage that are scoped to a prototype category.
type ProtoBundle = (Box<dyn Factory>, Box<dyn Storage>);

/// Mapping of prototype names to types.
pub struct PrototypeTable {
    prototypes2: HashMap<TypeId, ProtoBundle>,
    types: HashMap<String, TypeId>,
}

impl PrototypeTable {
    pub fn new() -> Self {
        Default::default()
    }

    /// Registers a prototype in the table.
    ///
    /// Required in order to add definition instances later.
    pub fn register<'de, T>(&mut self)
    where
        T: 'static + Prototype + Deserialize<'de>,
    {
        // Prototype type id
        let type_id = TypeId::of::<T>();

        // Name can be either defined as a static string, or
        // dynamically in script at runtime.
        let type_name: String = T::type_name().to_string();
        self.types.insert(type_name, type_id);

        let proto_factory: PrototypeFactory<T> = PrototypeFactory::new();
        let factory: Box<dyn Factory> = Box::new(proto_factory);

        let proto_storage: PrototypeMapStorage<T> = PrototypeMapStorage::new();

        self.prototypes2
            .insert(type_id, (factory, Box::new(proto_storage)));
    }

    /// Inserts a prototype definition into the table.
    ///
    /// The type name is required for the table to determine which concrete Rust
    /// type to use when deserializing the Lua value.
    ///
    /// The key can be any string that uniquely identifies the instance.
    ///
    /// Types must first be registered using [`PrototypeTable::register`](struct.PrototypeTable.html#method.register).
    ///
    /// This method is intended to be used in contexts where the Rust type for the
    /// prototype is not available. This is mostly Rust functions called by Lua inside
    /// a context, scope or user data method.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::borrow::Cow;
    /// # use serde::Deserialize;
    /// # use rlua;
    /// # use rengine::scripting::prelude::*;
    /// #
    /// # #[derive(Deserialize)]
    /// # struct GameActor {}
    /// #
    /// # impl Prototype for GameActor {
    /// #    fn type_name<'a>() -> Cow<'a, str> {
    /// #        "game_actor".into()
    /// #    }
    /// # }
    /// #
    /// # let mut prototype_table: PrototypeTable = PrototypeTable::new();
    /// // Importantly: Register type first!
    /// prototype_table.register::<GameActor>();
    ///
    /// # let lua = rlua::Lua::new();
    /// # let result: rlua::Result<()> = lua.context(|lua_ctx| {
    /// # let soldier_table = lua_ctx.load(r#"{}"#).eval::<rlua::Value>()?;
    /// #
    /// // Note: The `type_name` argument must match the `type_name` in
    /// //       the struct `GameActor`;
    /// prototype_table.insert("game_actor", "my_mod:game_actor:foot_soldier", soldier_table);
    /// #
    /// #    Ok(())
    /// # });
    /// # result.unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the type name has not been registered.
    ///
    /// Deserialization errors occur when the given Lua value cannot be deserialized into
    /// the registered Rust type.
    pub fn insert<'lua>(&mut self, type_name: &str, key: &str, value: rlua::Value<'lua>) {
        let type_id = self.types.get(type_name).unwrap();
        let (factory, storage) = self
            .prototypes2
            .get_mut(type_id)
            .map(|bundle| (Box::as_ref(&bundle.0), Box::as_mut(&mut bundle.1)))
            .unwrap();

        // Dynamic dispatch to concrete storage type, which would
        // have the concrete type of the target prototype as a generic
        // type parameter.
        factory.insert_value(storage, key.to_string(), value);
    }

    /// Retrieve an immutable reference to a prototype if it exists.
    pub fn get<T>(&self, key: &str) -> Option<&T>
    where
        T: 'static + Prototype,
    {
        let type_name = T::type_name();
        self.types
            .get(type_name.as_ref())
            .and_then(|type_id| self.prototypes2.get(&type_id))
            .map(|(_, storage)| Box::as_ref(storage))
            .and_then(|storage| storage.downcast_ref::<PrototypeMapStorage<T>>())
            .and_then(|proto_factory| proto_factory.get(key))
    }
}

impl Default for PrototypeTable {
    fn default() -> Self {
        PrototypeTable {
            prototypes2: HashMap::new(),
            types: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Foo {
        name: String,
        position: [i32; 2],
    }

    impl Prototype for Foo {
        fn type_name<'a>() -> Cow<'a, str> {
            "foo".into()
        }
    }

    #[test]
    fn test_table() {
        let mut table: PrototypeTable = PrototypeTable::new();
        let lua = rlua::Lua::new();

        table.register::<Foo>();

        let result: rlua::Result<()> = lua.context(|lua_ctx| {
            let value: rlua::Value = lua_ctx
                .load(
                    r#"
                    {
                        name = 'prototype_1',
                        position = { 1, 2 },
                    }
                    "#,
                )
                .eval()?;

            table.insert(Foo::type_name().as_ref(), "test:foo:prototype_1", value);

            Ok(())
        });
        result.unwrap();

        let prototype = table.get::<Foo>("test:foo:prototype_1").unwrap();
        assert_eq!(prototype.name, "prototype_1");
        assert_eq!(prototype.position, [1, 2]);
    }
}
