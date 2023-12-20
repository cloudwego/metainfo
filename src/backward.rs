use ahash::AHashMap;
use faststr::FastStr;

pub trait Backward {
    // We don't think backward persistent makes sense.
    fn get_backward_transient<K: AsRef<str>>(&self, key: K) -> Option<FastStr>;
    fn get_backward_downstream<K: AsRef<str>>(&self, key: K) -> Option<FastStr>;

    fn get_all_backward_transients(&self) -> Option<&AHashMap<FastStr, FastStr>>;
    fn get_all_backward_downstreams(&self) -> Option<&AHashMap<FastStr, FastStr>>;

    fn get_all_backward_transients_with_rpc_prefix(&self) -> Option<AHashMap<FastStr, FastStr>>;
    fn get_all_backward_transients_with_http_prefix(&self) -> Option<AHashMap<FastStr, FastStr>>;

    fn set_backward_transient<K: Into<FastStr>, V: Into<FastStr>>(&mut self, key: K, value: V);
    fn set_backward_downstream<K: Into<FastStr>, V: Into<FastStr>>(&mut self, key: K, value: V);

    fn strip_rpc_prefix_and_set_backward_downstream<K: Into<FastStr>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    );

    fn strip_http_prefix_and_set_backward_downstream<K: Into<FastStr>, V: Into<FastStr>>(
        &mut self,
        key: K,
        value: V,
    );

    fn del_backward_transient<K: AsRef<str>>(&mut self, key: K) -> Option<FastStr>;
    fn del_backward_downstream<K: AsRef<str>>(&mut self, key: K) -> Option<FastStr>;
}
