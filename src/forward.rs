use faststr::FastStr;

use crate::AHashMap;

pub trait Forward {
    fn get_persistent<K: AsRef<str>>(&self, key: K) -> Option<FastStr>;
    fn get_transient<K: AsRef<str>>(&self, key: K) -> Option<FastStr>;
    fn get_upstream<K: AsRef<str>>(&self, key: K) -> Option<FastStr>;

    fn get_all_persistents(&self) -> Option<&AHashMap<FastStr, FastStr>>;
    fn get_all_transients(&self) -> Option<&AHashMap<FastStr, FastStr>>;
    fn get_all_upstreams(&self) -> Option<&AHashMap<FastStr, FastStr>>;

    fn get_all_persistents_and_transients_with_rpc_prefix(
        &self,
    ) -> Option<AHashMap<FastStr, FastStr>>;
    fn get_all_persistents_and_transients_with_http_prefix(
        &self,
    ) -> Option<AHashMap<FastStr, FastStr>>;

    fn iter_persistents_and_transients_with_rpc_prefix(
        &self,
    ) -> impl Iterator<Item = (FastStr, &FastStr)>;
    fn iter_persistents_and_transients_with_http_prefix(
        &self,
    ) -> impl Iterator<Item = (FastStr, &FastStr)>;

    fn set_persistent<K: Into<FastStr>, V: Into<FastStr>>(&mut self, key: K, value: V);
    fn set_transient<K: Into<FastStr>, V: Into<FastStr>>(&mut self, key: K, value: V);
    fn set_upstream<K: Into<FastStr>, V: Into<FastStr>>(&mut self, key: K, value: V);

    fn strip_rpc_prefix_and_set_persistent<K: AsRef<str>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    );
    fn strip_rpc_prefix_and_set_upstream<K: AsRef<str>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    );

    fn strip_http_prefix_and_set_persistent<K: AsRef<str>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    );
    fn strip_http_prefix_and_set_upstream<K: AsRef<str>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    );

    fn del_persistent<K: AsRef<str>>(&mut self, key: K) -> Option<FastStr>;
    fn del_transient<K: AsRef<str>>(&mut self, key: K) -> Option<FastStr>;
    fn del_upstream<K: AsRef<str>>(&mut self, key: K) -> Option<FastStr>;
}
