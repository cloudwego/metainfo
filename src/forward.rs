use std::{borrow::Cow, collections::HashMap};

pub trait Forward {
    fn get_persistent<K: AsRef<str>>(&self, key: K) -> Option<&str>;
    fn get_transient<K: AsRef<str>>(&self, key: K) -> Option<&str>;
    fn get_upstream<K: AsRef<str>>(&self, key: K) -> Option<&str>;

    fn get_all_persistents(&self) -> Option<&HashMap<Cow<'static, str>, Cow<'static, str>>>;
    fn get_all_transients(&self) -> Option<&HashMap<Cow<'static, str>, Cow<'static, str>>>;
    fn get_all_upstreams(&self) -> Option<&HashMap<Cow<'static, str>, Cow<'static, str>>>;

    fn set_persistent<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        &mut self,
        key: K,
        value: V,
    );
    fn set_transient<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        &mut self,
        key: K,
        value: V,
    );
    fn set_upstream<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        &mut self,
        key: K,
        value: V,
    );

    fn strip_rpc_prefix_and_set_persistent<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        &mut self,
        key: K,
        value: V,
    );
    fn strip_rpc_prefix_and_set_upstream<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        &mut self,
        key: K,
        value: V,
    );

    fn strip_http_prefix_and_set_persistent<
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    >(
        &mut self,
        key: K,
        value: V,
    );
    fn strip_http_prefix_and_set_upstream<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        &mut self,
        key: K,
        value: V,
    );

    fn del_persistent<K: AsRef<str>>(&mut self, key: K);
    fn del_transient<K: AsRef<str>>(&mut self, key: K);
    fn del_upstream<K: AsRef<str>>(&mut self, key: K);
}
