//! String interning registry based on the
//! code found here: https://github.com/Marwes/haskell-compiler/blob/master/src/interner.rs

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct InternedStr(usize);

#[derive(Default)]
pub struct Interner {
    indexes: HashMap<String, usize>,
    strings: Vec<String>,
}

impl Interner {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn intern(&mut self, s: &str) -> InternedStr {
        match self.indexes.get(s).copied() {
            Some(index) => InternedStr(index),
            None => {
                let index = self.strings.len();
                self.indexes.insert(s.to_string(), index);
                self.strings.push(s.to_string());
                InternedStr(index)
            }
        }
    }

    pub fn get_str(&self, InternedStr(i): InternedStr) -> &str {
        if i < self.strings.len() {
            &*self.strings[i]
        } else {
            panic!("Invalid InternedStr {:?}", i)
        }
    }
}

#[cfg(feature = "nightly-features")]
impl !Sync for InternedStr {}

/// An interened string is stored in a thread local collection, with an integer
/// id that is only relevant to that collection. Sending an interened string to
/// another thread, and attempting to retrieve a string from its local
/// collection would yield an unexpected string or an out-of-range error.
///
/// Currently unimplementing built-in traits is only supported in
/// nightly.
#[cfg(feature = "nightly-features")]
impl !Send for InternedStr {}

/// Returns a reference to the interner stored in TLD
pub fn get_local_interner() -> Rc<RefCell<Interner>> {
    thread_local!(static INTERNER: Rc<RefCell<Interner>> = Rc::new(RefCell::new(Interner::new())));
    INTERNER.with(|interner| interner.clone())
}

pub fn intern(s: &str) -> InternedStr {
    let i = get_local_interner();
    let mut i = i.borrow_mut();
    i.intern(s)
}

impl Deref for InternedStr {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_ref()
    }
}

impl AsRef<str> for InternedStr {
    fn as_ref(&self) -> &str {
        let interner = get_local_interner();
        let x = (*interner).borrow_mut();
        let r: &str = x.get_str(*self);
        //The interner is task local and will never remove a string so this is safe
        unsafe { ::std::mem::transmute(r) }
    }
}

impl fmt::Display for InternedStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_str_interned() {
        let s = "foo";
        let i = intern(s);

        let interner = get_local_interner();

        assert_eq!("foo", interner.borrow().get_str(i));
    }
}
