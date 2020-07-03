//! Entity prototype definitions.
use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::HashMap,
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

trait Factory {
    fn from_value<'de>(&self, storage: &mut dyn Any, value: rlua::Value<'de>);
}

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
    fn from_value<'lua>(&self, storage: &mut dyn Any, value: rlua::Value<'lua>) {
        // let proto: Option<T> = self.from_value(value);
        // let proto: Option<T> = rlua_serde::from_value(value).unwrap();
        let deserializer = rlua_serde::de::Deserializer { value };
        let proto = T::deserialize(deserializer).unwrap();

        if let Some(concrete_storage) = storage.downcast_mut::<PrototypeVecStorage<T>>() {
            concrete_storage.insert(proto);
        } else {
            panic!("failed to downcast storage to concrete type");
        }
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
        let factory = Box::as_ref(self.factories.get(&type_id).unwrap());
        let proto_storage = Box::as_mut(self.prototypes.get_mut(&type_id).unwrap());

        factory.from_value(proto_storage, value);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Foo {
        name: String,
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
                        name = 'Prototype 1',
                    }
                    "#,
                )
                .eval()?;

            table.insert("foo", value);

            Ok(())
        });
        result.unwrap();
    }
}
