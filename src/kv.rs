use std::{borrow::Cow, collections::HashMap};

use paste::paste;

const DEFAULT_CAPACITY: usize = 10; // maybe enough for most cases?

macro_rules! set_impl {
    ($name:ident) => {
        paste! {
            pub fn [<set_ $name>]<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
                &mut self,
                key: K,
                value: V,
            ) {
                if self.$name.is_none() {
                    self.$name = Some(HashMap::with_capacity(DEFAULT_CAPACITY));
                }
                self.$name.as_mut().unwrap().insert(key.into(), value.into());
            }
        }
    };
}

macro_rules! del_impl {
    ($name:ident) => {
        paste! {
            pub fn [<del_ $name>]<K: AsRef<str>>(&mut self, key: K) {
                let key = key.as_ref();
                if let Some(v) = self.$name.as_mut() {
                    v.remove(key);
                }
            }
        }
    };
}

macro_rules! get_impl {
    ($name:ident) => {
        paste! {
            pub fn [<get_ $name>]<K: AsRef<str>>(&self, key: K) -> Option<&str> {
                let key = key.as_ref();
                match self.$name.as_ref() {
                    Some(v) => {
                        v.get(key).map(|v| v.as_ref())
                    }
                    None => None,
                }
            }
        }
    };
}

macro_rules! get_all_impl {
    ($name:ident) => {
        paste! {
            pub fn [<get_all_ $name s>](&self) -> Option<&HashMap<Cow<'static, str>, Cow<'static, str>>> {
                self.$name.as_ref()
            }
        }
    };
}

#[derive(Debug, Default, Clone)]
pub struct Node {
    persistent: Option<HashMap<Cow<'static, str>, Cow<'static, str>>>,
    transient: Option<HashMap<Cow<'static, str>, Cow<'static, str>>>,
    // this is called stale because upstream and downstream all use this.
    stale: Option<HashMap<Cow<'static, str>, Cow<'static, str>>>,
}

impl Node {
    set_impl!(persistent);
    set_impl!(transient);
    set_impl!(stale);

    del_impl!(persistent);
    del_impl!(transient);
    del_impl!(stale);

    get_impl!(persistent);
    get_impl!(transient);
    get_impl!(stale);

    get_all_impl!(persistent);
    get_all_impl!(transient);
    get_all_impl!(stale);

    pub fn extend(&mut self, other: Self) {
        if let Some(v) = other.persistent {
            if self.persistent.is_none() {
                self.persistent = Some(v);
            } else {
                self.persistent.as_mut().unwrap().extend(v);
            }
        }

        if let Some(v) = other.transient {
            if self.transient.is_none() {
                self.transient = Some(v);
            } else {
                self.transient.as_mut().unwrap().extend(v);
            }
        }

        if let Some(v) = other.stale {
            if self.stale.is_none() {
                self.stale = Some(v);
            } else {
                self.stale.as_mut().unwrap().extend(v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_stale() {
        let mut node = Node::default();
        node.set_stale("key", "value");
        println!("{:?}", node);
    }
}
