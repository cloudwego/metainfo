mod convert;
mod faststr_map;
mod kv;
mod type_map;

use std::{fmt, sync::Arc};

use ahash::AHashMap;
use convert::{Converter, HttpConverter, RpcConverter};
use faststr::FastStr;
pub use faststr_map::FastStrMap;
use kv::Node;
use paste::paste;
pub use type_map::TypeMap;

pub mod backward;
pub mod forward;
pub use backward::Backward;
pub use forward::Forward;

#[cfg(feature = "task_local")]
tokio::task_local! {
    pub static METAINFO: std::cell::RefCell<MetaInfo>;
}

/// Framework should all obey these prefixes.

pub const RPC_PREFIX_PERSISTENT: &str = "RPC_PERSIST_";
pub const RPC_PREFIX_TRANSIENT: &str = "RPC_TRANSIT_";
pub const RPC_PREFIX_BACKWARD: &str = "RPC_BACKWARD_";
pub const HTTP_PREFIX_PERSISTENT: &str = "rpc-persist-";
pub const HTTP_PREFIX_TRANSIENT: &str = "rpc-transit-";
pub const HTTP_PREFIX_BACKWARD: &str = "rpc-backward-";

const DEFAULT_MAP_SIZE: usize = 10;

/// `MetaInfo` is used to passthrough information between components and even client-server.
///
/// It supports two types of info: typed map and string k-v.
///
/// It is designed to be tree-like, which means you can share a `MetaInfo` with multiple children.
///
/// Note: only the current scope is mutable.
///
/// Examples:
/// ```rust
/// use metainfo::MetaInfo;
///
/// fn test() {
///     let mut m1 = MetaInfo::new();
///     m1.insert::<i8>(2);
///     assert_eq!(*m1.get::<i8>().unwrap(), 2);
///
///     let (mut m1, mut m2) = m1.derive();
///     assert_eq!(*m2.get::<i8>().unwrap(), 2);
///
///     m2.insert::<i8>(4);
///     assert_eq!(*m2.get::<i8>().unwrap(), 4);
///
///     m2.remove::<i8>();
///     assert_eq!(*m2.get::<i8>().unwrap(), 2);
/// }
/// ```
#[derive(Default)]
pub struct MetaInfo {
    /// Parent is read-only, if we can't find the specified key in the current,
    /// we search it in the parent scope.
    parent: Option<Arc<MetaInfo>>,
    tmap: Option<TypeMap>,
    smap: Option<AHashMap<FastStr, FastStr>>, // for str k-v
    faststr_tmap: Option<FastStrMap>,         // for newtype wrapper of FastStr

    /// for information transport through client and server.
    /// e.g. RPC
    forward_node: Option<kv::Node>,
    backward_node: Option<kv::Node>,
}

impl MetaInfo {
    /// Creates an empty `MetaInfo`.
    #[inline]
    pub fn new() -> MetaInfo {
        Default::default()
    }

    /// Creates an `MetaInfo` with the parent given.
    ///
    /// When the info is not found in the current scope, `MetaInfo` will try to get from parent.
    ///
    /// [`derive`] is more efficient than this. It is recommended to use [`derive`] instead of this.
    #[inline]
    pub fn from(parent: Arc<MetaInfo>) -> MetaInfo {
        let forward_node = parent.forward_node.clone();
        let backward_node = parent.backward_node.clone();
        MetaInfo {
            parent: Some(parent),
            tmap: None,
            smap: None,
            faststr_tmap: None,

            forward_node,
            backward_node,
        }
    }

    /// Derives the current [`MetaInfo`], returns two new equivalent `Metainfo`s.
    ///
    /// When the info is not found in the current scope, `MetaInfo` will try to get from parent.
    ///
    /// This is the recommended way.
    #[inline]
    pub fn derive(mut self) -> (MetaInfo, MetaInfo) {
        if self.tmap.is_none() && self.smap.is_none() && self.faststr_tmap.is_none() {
            // we can use the same parent as self to make the tree small
            let new = MetaInfo {
                parent: self.parent.clone(),
                tmap: None,
                smap: None,
                faststr_tmap: None,
                forward_node: self.forward_node.clone(),
                backward_node: self.backward_node.clone(),
            };
            (self, new)
        } else {
            let forward_node = self.forward_node.take();
            let backward_node = self.backward_node.take();
            let mi = Arc::new(self);
            (
                MetaInfo::from_node(mi.clone(), forward_node.clone(), backward_node.clone()),
                MetaInfo::from_node(mi, forward_node, backward_node),
            )
        }
    }

    /// Creates an `MetaInfo` with the parent and node given.
    fn from_node(
        parent: Arc<MetaInfo>,
        forward_node: Option<kv::Node>,
        backward_node: Option<kv::Node>,
    ) -> MetaInfo {
        MetaInfo {
            parent: Some(parent),
            tmap: None,
            smap: None,
            faststr_tmap: None,

            forward_node,
            backward_node,
        }
    }

    /// Insert a type into this `MetaInfo`.
    #[inline]
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) {
        self.tmap
            .get_or_insert_with(|| TypeMap::with_capacity(DEFAULT_MAP_SIZE))
            .insert(val);
    }

    /// Insert a faststr newtype into this `MetaInfo`.
    #[inline]
    pub fn insert_faststr<T: Send + Sync + 'static>(&mut self, val: FastStr) {
        self.faststr_tmap
            .get_or_insert_with(|| FastStrMap::with_capacity(DEFAULT_MAP_SIZE))
            .insert::<T>(val);
    }

    /// Insert a string k-v into this `MetaInfo`.
    #[inline]
    pub fn insert_string(&mut self, key: FastStr, val: FastStr) {
        self.smap
            .get_or_insert_with(|| AHashMap::with_capacity(DEFAULT_MAP_SIZE))
            .insert(key, val);
    }

    /// Check if `MetaInfo` contains entry
    #[inline]
    pub fn contains<T: 'static>(&self) -> bool {
        if self
            .tmap
            .as_ref()
            .map(|tmap| tmap.contains::<T>())
            .unwrap_or(false)
        {
            return true;
        }
        self.parent
            .as_ref()
            .map(|parent| parent.as_ref().contains::<T>())
            .unwrap_or(false)
    }

    /// Check if `MetaInfo` contains the given Faststr newtype
    #[inline]
    pub fn contains_faststr<T: 'static>(&self) -> bool {
        if self
            .faststr_tmap
            .as_ref()
            .map(|faststr_tmap| faststr_tmap.contains::<T>())
            .unwrap_or(false)
        {
            return true;
        }
        self.parent
            .as_ref()
            .map(|parent| parent.as_ref().contains_faststr::<T>())
            .unwrap_or(false)
    }

    /// Check if `MetaInfo` contains the given string k-v
    #[inline]
    pub fn contains_string<K: AsRef<str>>(&self, key: K) -> bool {
        if self
            .smap
            .as_ref()
            .map(|smap| smap.contains_key(key.as_ref()))
            .unwrap_or(false)
        {
            return true;
        }
        self.parent
            .as_ref()
            .map(|parent| parent.as_ref().contains_string(key))
            .unwrap_or(false)
    }

    /// Get a reference to a type previously inserted on this `MetaInfo`.
    #[inline]
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.tmap.as_ref().and_then(|tmap| tmap.get()).or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.as_ref().get::<T>())
        })
    }

    /// Remove a type from this `MetaInfo` and return it.
    /// Can only remove the type in the current scope.
    #[inline]
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.tmap.as_mut().and_then(|tmap| tmap.remove::<T>())
    }

    /// Get a reference to a faststr newtype previously inserted on this `MetaInfo`.
    #[inline]
    pub fn get_faststr<T: 'static>(&self) -> Option<&FastStr> {
        self.faststr_tmap
            .as_ref()
            .and_then(|faststr_tmap: &FastStrMap| faststr_tmap.get::<T>())
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|parent| parent.as_ref().get_faststr::<T>())
            })
    }

    /// Remove a faststr newtype from this `MetaInfo` and return it.
    /// Can only remove the type in the current scope.
    #[inline]
    pub fn remove_faststr<T: 'static>(&mut self) -> Option<FastStr> {
        self.faststr_tmap
            .as_mut()
            .and_then(|faststr_tmap| faststr_tmap.remove::<T>())
    }

    /// Get a reference to a string k-v previously inserted on this `MetaInfo`.
    #[inline]
    pub fn get_string<K: AsRef<str>>(&self, key: K) -> Option<&FastStr> {
        self.smap
            .as_ref()
            .and_then(|smap| smap.get(key.as_ref()))
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|parent| parent.as_ref().get_string(key))
            })
    }

    /// Remove a string k-v from this `MetaInfo` and return it.
    /// Can only remove the type in the current scope.
    #[inline]
    pub fn remove_string<K: AsRef<str>>(&mut self, key: K) -> Option<FastStr> {
        self.smap
            .as_mut()
            .and_then(|smap| smap.remove(key.as_ref()))
    }

    /// Clear the `MetaInfo` of all inserted MetaInfo.
    /// This will not clear the parent.
    #[inline]
    pub fn clear(&mut self) {
        self.parent = None;
        if let Some(tmap) = self.tmap.as_mut() {
            tmap.clear()
        }
        if let Some(smap) = self.smap.as_mut() {
            smap.clear()
        }
        if let Some(faststr_tmap) = self.faststr_tmap.as_mut() {
            faststr_tmap.clear()
        }
        if let Some(forward_node) = self.forward_node.as_mut() {
            forward_node.clear()
        }
        if let Some(backward_node) = self.backward_node.as_mut() {
            backward_node.clear()
        }
    }

    /// Extends self with the items from another `MetaInfo`.
    /// Only extend the items in the current scope.
    #[inline]
    pub fn extend(&mut self, other: MetaInfo) {
        if let Some(tmap) = other.tmap {
            self.tmap
                .get_or_insert_with(|| TypeMap::with_capacity(DEFAULT_MAP_SIZE))
                .extend(tmap);
        }

        if let Some(smap) = other.smap {
            self.smap
                .get_or_insert_with(|| AHashMap::with_capacity(DEFAULT_MAP_SIZE))
                .extend(smap);
        }

        if let Some(faststr_tmap) = other.faststr_tmap {
            self.faststr_tmap
                .get_or_insert_with(|| FastStrMap::with_capacity(DEFAULT_MAP_SIZE))
                .extend(faststr_tmap);
        }

        if let Some(node) = other.forward_node {
            if self.forward_node.is_none() {
                self.forward_node = Some(node);
            } else {
                self.forward_node.as_mut().unwrap().extend(node);
            }
        }

        if let Some(node) = other.backward_node {
            if self.backward_node.is_none() {
                self.backward_node = Some(node);
            } else {
                self.backward_node.as_mut().unwrap().extend(node);
            }
        }
    }

    fn ensure_forward_node(&mut self) {
        if self.forward_node.is_none() {
            self.forward_node = Some(Node::default())
        }
    }

    fn ensure_backward_node(&mut self) {
        if self.backward_node.is_none() {
            self.backward_node = Some(Node::default())
        }
    }
}

macro_rules! get_impl {
    ($name:ident,$node:ident,$func_name:ident) => {
        paste! {
            fn [<get_ $name>]<K: AsRef<str>>(&self, key: K) -> Option<FastStr> {
                match self.[<$node _node>].as_ref() {
                    Some(node) => node.[<get_ $func_name>](key),
                    None => None,
                }
            }
        }
    };
}

macro_rules! set_impl {
    ($name:ident,$node:ident,$func_name:ident) => {
        paste! {
            fn [<set_ $name>]<K: Into<FastStr>, V: Into<FastStr>>(
                &mut self,
                key: K,
                value: V,
            ) {
                self.[<ensure_ $node _node>]();
                self.[<$node _node>]
                    .as_mut()
                    .unwrap()
                    .[<set_ $func_name>](key, value)
            }
        }
    };
}

macro_rules! del_impl {
    ($name:ident,$node:ident,$func_name:ident) => {
        paste! {
            fn [<del_ $name>]<K: AsRef<str>>(&mut self, key: K) -> Option<FastStr> {
                if let Some(node) = self.[<$node _node>].as_mut() {
                    node.[<del_ $func_name>](key)
                } else {
                    None
                }
            }
        }
    };
}

impl forward::Forward for MetaInfo {
    get_impl!(persistent, forward, persistent);
    get_impl!(transient, forward, transient);
    get_impl!(upstream, forward, stale);

    set_impl!(persistent, forward, persistent);
    set_impl!(transient, forward, transient);
    set_impl!(upstream, forward, stale);

    del_impl!(persistent, forward, persistent);
    del_impl!(transient, forward, transient);
    del_impl!(upstream, forward, stale);

    fn get_all_persistents(&self) -> Option<&AHashMap<FastStr, FastStr>> {
        match self.forward_node.as_ref() {
            Some(node) => node.get_all_persistents(),
            None => None,
        }
    }

    fn get_all_persistents_and_transients_with_rpc_prefix(
        &self,
    ) -> Option<AHashMap<FastStr, FastStr>> {
        self.get_all_persistents_and_transients(RpcConverter)
    }

    fn get_all_persistents_and_transients_with_http_prefix(
        &self,
    ) -> Option<AHashMap<FastStr, FastStr>> {
        self.get_all_persistents_and_transients(HttpConverter)
    }

    fn get_all_transients(&self) -> Option<&AHashMap<FastStr, FastStr>> {
        match self.forward_node.as_ref() {
            Some(node) => node.get_all_transients(),
            None => None,
        }
    }

    fn get_all_upstreams(&self) -> Option<&AHashMap<FastStr, FastStr>> {
        match self.forward_node.as_ref() {
            Some(node) => node.get_all_stales(),
            None => None,
        }
    }

    fn strip_rpc_prefix_and_set_persistent<K: AsRef<str>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    ) {
        let key = key.as_ref();
        if let Some(key) = RpcConverter.remove_persistent_prefix(key) {
            self.set_persistent(key, value);
        }
    }

    fn strip_rpc_prefix_and_set_upstream<K: AsRef<str>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    ) {
        let key = key.as_ref();
        if let Some(key) = RpcConverter.remove_transient_prefix(key) {
            self.set_upstream(key, value);
        }
    }

    fn strip_http_prefix_and_set_persistent<K: AsRef<str>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    ) {
        let key = key.as_ref();
        if let Some(key) = HttpConverter.remove_persistent_prefix(key) {
            self.set_persistent(key, value);
        }
    }

    fn strip_http_prefix_and_set_upstream<K: AsRef<str>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    ) {
        let key = key.as_ref();
        if let Some(key) = HttpConverter.remove_transient_prefix(key) {
            self.set_upstream(key, value);
        }
    }
}

impl backward::Backward for MetaInfo {
    get_impl!(backward_transient, backward, transient);
    get_impl!(backward_downstream, backward, stale);

    set_impl!(backward_transient, backward, transient);
    set_impl!(backward_downstream, backward, stale);

    del_impl!(backward_transient, backward, transient);
    del_impl!(backward_downstream, backward, stale);

    fn get_all_backward_transients(&self) -> Option<&AHashMap<FastStr, FastStr>> {
        match self.backward_node.as_ref() {
            Some(node) => node.get_all_transients(),
            None => None,
        }
    }

    fn get_all_backward_downstreams(&self) -> Option<&AHashMap<FastStr, FastStr>> {
        match self.backward_node.as_ref() {
            Some(node) => node.get_all_stales(),
            None => None,
        }
    }

    fn get_all_backward_transients_with_rpc_prefix(&self) -> Option<AHashMap<FastStr, FastStr>> {
        self.get_all_backword_transients_with_prefix(RpcConverter)
    }

    fn get_all_backward_transients_with_http_prefix(&self) -> Option<AHashMap<FastStr, FastStr>> {
        self.get_all_backword_transients_with_prefix(HttpConverter)
    }

    fn strip_rpc_prefix_and_set_backward_downstream<K: AsRef<str>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    ) {
        let key = key.as_ref();
        if let Some(key) = RpcConverter.remove_backward_prefix(key) {
            self.set_backward_downstream(key, value);
        }
    }

    fn strip_http_prefix_and_set_backward_downstream<K: AsRef<str>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    ) {
        let key = key.as_ref();
        if let Some(key) = HttpConverter.remove_backward_prefix(key) {
            self.set_backward_downstream(key, value);
        }
    }
}

impl MetaInfo {
    #[inline]
    fn get_all_persistents_and_transients<C>(
        &self,
        converter: C,
    ) -> Option<AHashMap<FastStr, FastStr>>
    where
        C: Converter,
    {
        match self.forward_node.as_ref() {
            Some(node) => {
                let persistents = node.get_all_persistents();
                let transients = node.get_all_transients();
                let new_cap = persistents.map(|p| p.len()).unwrap_or(0)
                    + transients.map(|t| t.len()).unwrap_or(0);
                if new_cap == 0 {
                    return None;
                }
                let mut map = AHashMap::with_capacity(new_cap);
                if let Some(persistents) = persistents {
                    map.extend(
                        persistents
                            .iter()
                            .map(|(k, v)| (converter.add_persistent_prefix(k), v.clone())),
                    );
                }
                if let Some(transients) = transients {
                    map.extend(
                        transients
                            .iter()
                            .map(|(k, v)| (converter.add_transient_prefix(k), v.clone())),
                    );
                }
                Some(map)
            }
            None => None,
        }
    }

    #[inline]
    fn get_all_backword_transients_with_prefix<C>(
        &self,
        converter: C,
    ) -> Option<AHashMap<FastStr, FastStr>>
    where
        C: Converter,
    {
        match self.backward_node.as_ref() {
            Some(node) => {
                if let Some(t) = node.get_all_transients() {
                    let new_cap = t.len();
                    if new_cap == 0 {
                        return None;
                    }
                    let mut map = AHashMap::with_capacity(new_cap);
                    map.extend(
                        t.iter()
                            .map(|(k, v)| (converter.add_transient_prefix(k), v.clone())),
                    );
                    Some(map)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

impl fmt::Debug for MetaInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetaInfo").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove() {
        let mut map = MetaInfo::new();

        map.insert::<i8>(123);
        assert!(map.get::<i8>().is_some());

        map.remove::<i8>();
        assert!(map.get::<i8>().is_none());

        map.insert::<i8>(123);

        let mut m2 = MetaInfo::from(Arc::new(map));

        m2.remove::<i8>();
        assert!(m2.get::<i8>().is_some());
    }

    #[test]
    fn test_clear() {
        let mut map = MetaInfo::new();

        map.insert::<i8>(8);
        map.insert::<i16>(16);
        map.insert::<i32>(32);

        assert!(map.contains::<i8>());
        assert!(map.contains::<i16>());
        assert!(map.contains::<i32>());

        map.clear();

        assert!(!map.contains::<i8>());
        assert!(!map.contains::<i16>());
        assert!(!map.contains::<i32>());

        map.insert::<i8>(10);
        assert_eq!(*map.get::<i8>().unwrap(), 10);
    }

    #[test]
    fn test_integers() {
        let mut map = MetaInfo::new();

        map.insert::<i8>(8);
        map.insert::<i16>(16);
        map.insert::<i32>(32);
        map.insert::<i64>(64);
        map.insert::<i128>(128);
        map.insert::<u8>(8);
        map.insert::<u16>(16);
        map.insert::<u32>(32);
        map.insert::<u64>(64);
        map.insert::<u128>(128);
        assert!(map.get::<i8>().is_some());
        assert!(map.get::<i16>().is_some());
        assert!(map.get::<i32>().is_some());
        assert!(map.get::<i64>().is_some());
        assert!(map.get::<i128>().is_some());
        assert!(map.get::<u8>().is_some());
        assert!(map.get::<u16>().is_some());
        assert!(map.get::<u32>().is_some());
        assert!(map.get::<u64>().is_some());
        assert!(map.get::<u128>().is_some());

        let m2 = MetaInfo::from(Arc::new(map));
        assert!(m2.get::<i8>().is_some());
        assert!(m2.get::<i16>().is_some());
        assert!(m2.get::<i32>().is_some());
        assert!(m2.get::<i64>().is_some());
        assert!(m2.get::<i128>().is_some());
        assert!(m2.get::<u8>().is_some());
        assert!(m2.get::<u16>().is_some());
        assert!(m2.get::<u32>().is_some());
        assert!(m2.get::<u64>().is_some());
        assert!(m2.get::<u128>().is_some());
    }

    #[test]
    fn test_composition() {
        struct Magi<T>(pub T);

        struct Madoka {
            pub god: bool,
        }

        struct Homura {
            pub attempts: usize,
        }

        struct Mami {
            pub guns: usize,
        }

        let mut map = MetaInfo::new();

        map.insert(Magi(Madoka { god: false }));
        map.insert(Magi(Homura { attempts: 0 }));
        map.insert(Magi(Mami { guns: 999 }));

        assert!(!map.get::<Magi<Madoka>>().unwrap().0.god);
        assert_eq!(0, map.get::<Magi<Homura>>().unwrap().0.attempts);
        assert_eq!(999, map.get::<Magi<Mami>>().unwrap().0.guns);
    }

    #[test]
    fn test_metainfo() {
        #[derive(Debug, PartialEq)]
        struct MyType(i32);

        let mut metainfo = MetaInfo::new();

        metainfo.insert(5i32);
        metainfo.insert(MyType(10));

        assert_eq!(metainfo.get(), Some(&5i32));

        assert_eq!(metainfo.remove::<i32>(), Some(5i32));
        assert!(metainfo.get::<i32>().is_none());

        assert_eq!(metainfo.get::<bool>(), None);
        assert_eq!(metainfo.get(), Some(&MyType(10)));
    }

    #[test]
    fn test_extend() {
        #[derive(Debug, PartialEq)]
        struct MyType(i32);

        let mut metainfo = MetaInfo::new();

        metainfo.insert(5i32);
        metainfo.insert(MyType(10));

        let mut other = MetaInfo::new();

        other.insert(15i32);
        other.insert(20u8);

        metainfo.extend(other);

        assert_eq!(metainfo.get(), Some(&15i32));

        assert_eq!(metainfo.remove::<i32>(), Some(15i32));
        assert!(metainfo.get::<i32>().is_none());

        assert_eq!(metainfo.get::<bool>(), None);
        assert_eq!(metainfo.get(), Some(&MyType(10)));

        assert_eq!(metainfo.get(), Some(&20u8));
    }

    #[test]
    fn rpc_forward_test() {
        let mut metainfo = MetaInfo::new();
        metainfo.strip_rpc_prefix_and_set_persistent("RPC_PERSIST_TEST_KEY", "PERSIST");
        metainfo.strip_rpc_prefix_and_set_upstream("RPC_TRANSIT_TEST_KEY", "TRANSIT");
        assert_eq!(metainfo.get_persistent("TEST_KEY").unwrap(), "PERSIST");
        assert_eq!(metainfo.get_upstream("TEST_KEY").unwrap(), "TRANSIT");
        let map = metainfo
            .get_all_persistents_and_transients_with_rpc_prefix()
            .unwrap();
        assert_eq!(map.get("RPC_PERSIST_TEST_KEY").unwrap(), "PERSIST");
        // The `RPC_TRANSIT_TEST_KEY` is inserted into `upstream` and we cannot get it from
        // `transients`.
    }

    #[test]
    fn http_forward_test() {
        let mut metainfo = MetaInfo::new();
        metainfo.strip_http_prefix_and_set_persistent("rpc-persist-test-key", "persist");
        metainfo.strip_http_prefix_and_set_upstream("rpc-transit-test-key", "transit");
        assert_eq!(metainfo.get_persistent("TEST_KEY").unwrap(), "persist");
        assert_eq!(metainfo.get_upstream("TEST_KEY").unwrap(), "transit");
        let map = metainfo
            .get_all_persistents_and_transients_with_http_prefix()
            .unwrap();
        assert_eq!(map.get("rpc-persist-test-key").unwrap(), "persist");
        // The `RPC_TRANSIT_TEST_KEY` is inserted into `upstream` and we cannot get it from
        // `transients`.
    }
}
