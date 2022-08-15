use std::{borrow::Cow, collections::HashMap};

pub trait Backward {
    // We don't think backward persistent makes sense.
    fn get_backward_transient<K: AsRef<str>>(&self, key: K) -> Option<&str>;
    fn get_backward_downstream<K: AsRef<str>>(&self, key: K) -> Option<&str>;

    fn get_all_backward_transients(&self)
        -> Option<&HashMap<Cow<'static, str>, Cow<'static, str>>>;
    fn get_all_backward_downstreams(
        &self,
    ) -> Option<&HashMap<Cow<'static, str>, Cow<'static, str>>>;

    fn set_backward_transient<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        &mut self,
        key: K,
        value: V,
    );
    fn set_backward_downstream<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        &mut self,
        key: K,
        value: V,
    );

    fn strip_rpc_prefix_and_set_backward_downstream<
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    >(
        &mut self,
        key: K,
        value: V,
    );

    fn strip_http_prefix_and_set_backward_downstream<
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    >(
        &mut self,
        key: K,
        value: V,
    );

    fn del_backward_transient<K: AsRef<str>>(&mut self, key: K);
    fn del_backward_downstream<K: AsRef<str>>(&mut self, key: K);
}
