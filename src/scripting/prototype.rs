//! Entity prototype definitions.
use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::{hash_map, HashMap},
    marker::PhantomData,
    mem,
};

use serde::Deserialize;

fn from_value<'lua>(value: rlua::Value<'lua>) {
    let deserializer = rlua_serde::de::Deserializer { value };
}

pub trait PrototypeMarker {
    fn type_name<'a>() -> Cow<'a, str>;
}

pub trait Prototype {}

impl<T> Prototype for T where T: PrototypeMarker {}

#[derive(Deserialize)]
struct Foo {
    name: String,
}

impl PrototypeMarker for Foo {
    fn type_name<'a>() -> Cow<'a, str> {
        "foo".into()
    }
}

pub trait PrototypeStorage {
    // fn insert<T: Prototype>(&mut self, proto: T);
    // fn get<T: Prototype>(&self) -> &dyn T;
}

pub struct PrototypeVecStorage<T: Prototype> {
    data: Vec<T>,
}

impl<T> PrototypeVecStorage<T>
where
    T: Prototype,
{
    fn new() -> Self {
        PrototypeVecStorage { data: vec![] }
    }

    fn insert(&mut self, proto: T) {
        self.data.push(proto);
    }
}

impl<T> PrototypeStorage for PrototypeVecStorage<T>
where
    T: Prototype,
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
    fn insert_value<'de>(&mut self, key: String, value: rlua::Value<'de>);
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
    fn insert_value<'lua>(&mut self, key: String, value: rlua::Value<'lua>) {
        let deserializer = rlua_serde::de::Deserializer { value };
        let proto = T::deserialize(deserializer).unwrap();

        // TODO: Error on existing?
        self.data.insert(key, proto);
    }
}

/// Mapping of prototype names to types.
pub struct PrototypeTable {
    prototypes: HashMap<TypeId, Box<dyn Any>>,
    factories: HashMap<TypeId, Box<dyn Factory>>,
    types: HashMap<String, TypeId>,
}

impl PrototypeTable {
    pub fn new() -> Self {
        PrototypeTable {
            prototypes: HashMap::new(),
            factories: HashMap::new(),
            types: HashMap::new(),
        }
    }

    pub fn register<'de, T>(&mut self)
    where
        T: 'static + Prototype + PrototypeMarker + Deserialize<'de>,
    {
        let type_id = TypeId::of::<T>();

        let type_name = match T::type_name() {
            Cow::Borrowed(s) => s.to_owned(),
            Cow::Owned(s) => s,
        };

        self.types.insert(type_name, type_id);

        let proto_factory: PrototypeFactory<T> = PrototypeFactory::new();
        let factory: Box<dyn Factory> = Box::new(proto_factory);
        self.factories.insert(type_id, factory);

        let proto_storage: PrototypeVecStorage<T> = PrototypeVecStorage::new();
        self.prototypes.insert(type_id, Box::new(proto_storage));
    }

    pub fn insert<'lua>(&mut self, type_name: &str, value: rlua::Value<'lua>) {
        let type_id = self.types.get(type_name).unwrap();
        let factory = Box::as_mut(self.factories.get_mut(&type_id).unwrap());
        // let proto_storage = Box::as_mut(self.prototypes.get_mut(&type_id).unwrap());

        // TODO: Smarter way to extract key from value
        let key = if let rlua::Value::Table(ref t) = value {
            t.get::<_, String>("name").unwrap()
        } else {
            panic!("Unsupported Lua value type");
        };

        // Dynamic dispatch to concrete storage type, which would
        // have the concrete type of the target prototype.
        factory.insert_value(key, value);
    }

    // TODO: Figure out how to get rid of Deserialize<'de>
    pub fn get<'de, T>(&self, type_name: &str, proto_name: &str) -> Option<&T>
    where
        T: 'static + Prototype + Deserialize<'de>,
    {
        self.types
            .get(type_name)
            .and_then(|type_id| self.factories.get(&type_id))
            .map(|factory_box| Box::as_ref(factory_box))
            .and_then(|factory| factory.downcast_ref::<PrototypeFactory<T>>())
            .and_then(|proto_factory| proto_factory.data.get(proto_name))
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

    impl PrototypeMarker for Foo {
        fn type_name<'a>() -> Cow<'a, str> {
            "foo".into()
        }
    }

    #[test]
    fn test_add() {
        let mut table = PrototypeTable::new();
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

            table.insert("foo", value);

            Ok(())
        });
        result.unwrap();

        let prototype = table.get::<Foo>("foo", "prototype_1").unwrap();
        assert_eq!(prototype.position, [1, 2]);
    }
}
