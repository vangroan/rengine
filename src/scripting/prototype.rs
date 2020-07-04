//! Entity prototype definitions.
//!
//! # Implementation
//!
//! Prototypes are stored in a separate boxed storage so the `Deserialized`
//! trait isn't needed for [`PrototypeTable::get`].
use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::{hash_map, HashMap},
    marker::PhantomData,
    mem,
};

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
///     /// Game context for spawning an entity, like
///     /// the world database and graphics device.
///     type Context = ();
///
///     /// The return value when spawning an entity
///     /// from this prototype.
///     type Spawned = Option<()>;
///
///     /// A string key for mod scripts to refer to this type.
///     fn type_name<'a>() -> Cow<'a, str> {
///         "game_actor".into()
///     }
///
///     /// Create an entity using the data in this prototype.
///     fn spawn(&self, ctx: &mut Self::Context) -> Self::Spawned {
///         Some(())
///     }
/// }
/// ```
pub trait Prototype {
    type Context;
    type Spawned;

    fn type_name<'a>() -> Cow<'a, str>;
    fn spawn(&self, ctx: &mut Self::Context) -> Self::Spawned;
}

trait Storage: mopa::Any {
    // fn insert<T: Prototype>(&mut self, proto: T);
    // fn get<T: Prototype>(&self) -> &dyn T;
}
mopafy!(Storage);

pub struct PrototypeVecStorage<T: Prototype> {
    data: HashMap<String, T>,
}

impl<T> PrototypeVecStorage<T>
where
    T: Prototype,
{
    fn new() -> Self {
        PrototypeVecStorage {
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

impl<T> Storage for PrototypeVecStorage<T>
where
    T: 'static + Prototype,
{
    // fn insert<P>(&mut self, proto: P)
    // where
    //     T: Prototype,
    // {
    // if TypeId::of::<T>() == TypeId::of::<P>() {
    //     mem::transmute(e: T)
    // } else {
    //     panic!("mismatched types");
    // }
    // }
}

trait Factory: mopa::Any {
    fn insert_value<'lua>(&self, storage: &mut Storage, key: String, value: rlua::Value<'lua>);
}

mopafy!(Factory);

struct PrototypeFactory<T: Prototype> {
    data: HashMap<String, T>,
    _marker: PhantomData<T>,
}

impl<T> PrototypeFactory<T>
where
    T: Prototype,
{
    fn new() -> Self {
        PrototypeFactory {
            data: HashMap::new(),
            _marker: PhantomData,
        }
    }
}

// impl<T> Factory for PrototypeFactory<T>
// where
//     T: Prototype,
// {
//     fn from_value<'lua>(&self, storage: &mut dyn PrototypeStorage, value: rlua::Value<'lua>) {
//         // let deserializer = rlua_serde::de::Deserializer { value };
//     }
// }

impl<'de, T> Factory for PrototypeFactory<T>
where
    T: 'static + Prototype + Deserialize<'de>,
{
    fn insert_value<'lua>(&self, storage: &mut Storage, key: String, value: rlua::Value<'lua>) {
        let deserializer = rlua_serde::de::Deserializer { value };
        let proto = T::deserialize(deserializer).unwrap();

        // TODO: Error on existing?
        storage
            .downcast_mut::<PrototypeVecStorage<T>>()
            .expect("Unexpected storage type during downcast")
            .insert(key, proto);
    }
}

type ProtoBundle = (Box<dyn Factory>, Box<dyn Storage>);

/// Mapping of prototype names to types.
pub struct PrototypeTable<Ctx> {
    prototypes2: HashMap<TypeId, ProtoBundle>,
    prototypes: HashMap<TypeId, Box<dyn Any>>,
    factories: HashMap<TypeId, Box<dyn Factory>>,
    types: HashMap<String, TypeId>,
    _marker: PhantomData<Ctx>,
}

impl<C> PrototypeTable<C> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register<'de, T>(&mut self)
    where
        T: 'static + Prototype + Deserialize<'de>,
    {
        // Prototype type id
        let type_id = TypeId::of::<T>();

        // Name can be either defined as a static string, or
        // dynamically in script at runtime.
        let type_name = match T::type_name() {
            Cow::Borrowed(s) => s.to_owned(),
            Cow::Owned(s) => s,
        };

        self.types.insert(type_name, type_id);

        let proto_factory: PrototypeFactory<T> = PrototypeFactory::new();
        let factory: Box<dyn Factory> = Box::new(proto_factory);
        // self.factories.insert(type_id, factory);

        let proto_storage: PrototypeVecStorage<T> = PrototypeVecStorage::new();
        // self.prototypes.insert(type_id, Box::new(proto_storage));

        self.prototypes2
            .insert(type_id, (factory, Box::new(proto_storage)));
    }

    pub fn insert<'lua>(&mut self, type_name: &str, key: &str, value: rlua::Value<'lua>) {
        let type_id = self.types.get(type_name).unwrap();
        let (factory, storage) = self
            .prototypes2
            .get_mut(type_id)
            .map(|bundle| (Box::as_ref(&bundle.0), Box::as_mut(&mut bundle.1)))
            .unwrap();

        // Dynamic dispatch to concrete storage type, which would
        // have the concrete type of the target prototype.
        factory.insert_value(storage, key.to_string(), value);
    }

    pub fn get<T>(&self, key: &str) -> Option<&T>
    where
        T: 'static + Prototype,
    {
        let type_name = T::type_name();
        self.types
            .get(type_name.as_ref())
            .and_then(|type_id| self.prototypes2.get(&type_id))
            .map(|(_, storage)| Box::as_ref(storage))
            .and_then(|storage| storage.downcast_ref::<PrototypeVecStorage<T>>())
            .and_then(|proto_factory| proto_factory.data.get(key))
    }
}

impl<C> Default for PrototypeTable<C> {
    fn default() -> Self {
        PrototypeTable {
            prototypes2: HashMap::new(),
            prototypes: HashMap::new(),
            factories: HashMap::new(),
            types: HashMap::new(),
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    struct World(usize);
    type Entity = usize;

    #[derive(Deserialize)]
    struct Foo {
        name: String,
        position: [i32; 2],
    }

    impl Prototype for Foo {
        type Context = World;
        type Spawned = Entity;

        fn type_name<'a>() -> Cow<'a, str> {
            "foo".into()
        }

        fn spawn(&self, ctx: &mut Self::Context) -> Self::Spawned {
            ctx.0 += 1;
            ctx.0
        }
    }

    #[test]
    fn test_table() {
        let mut table: PrototypeTable<World> = PrototypeTable::new();
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
        assert_eq!(prototype.position, [1, 2]);
    }
}
