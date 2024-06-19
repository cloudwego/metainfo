use faststr::FastStr;

use crate::{
    HTTP_PREFIX_BACKWARD, HTTP_PREFIX_PERSISTENT, HTTP_PREFIX_TRANSIENT, RPC_PREFIX_BACKWARD,
    RPC_PREFIX_PERSISTENT, RPC_PREFIX_TRANSIENT,
};

pub trait Converter {
    fn add_persistent_prefix(&self, key: &str) -> FastStr;
    fn add_transient_prefix(&self, key: &str) -> FastStr;
    #[allow(dead_code)]
    fn add_backward_prefix(&self, key: &str) -> FastStr;

    fn remove_persistent_prefix(&self, key: &str) -> Option<FastStr>;
    fn remove_transient_prefix(&self, key: &str) -> Option<FastStr>;
    fn remove_backward_prefix(&self, key: &str) -> Option<FastStr>;
}

pub struct RpcConverter;

impl RpcConverter {
    #[inline]
    fn add_prefix(&self, prefix: &'static str, key: &str) -> FastStr {
        let mut res = String::with_capacity(prefix.len() + key.len());
        res.push_str(prefix);
        res.push_str(key);
        FastStr::from_string(res)
    }

    #[inline]
    fn remove_prefix(&self, prefix: &'static str, key: &str) -> Option<FastStr> {
        let key = key.strip_prefix(prefix)?;
        Some(FastStr::from_string(key.to_owned()))
    }
}

impl Converter for RpcConverter {
    fn add_persistent_prefix(&self, key: &str) -> FastStr {
        self.add_prefix(RPC_PREFIX_PERSISTENT, key)
    }

    fn add_transient_prefix(&self, key: &str) -> FastStr {
        self.add_prefix(RPC_PREFIX_TRANSIENT, key)
    }

    fn add_backward_prefix(&self, key: &str) -> FastStr {
        self.add_prefix(RPC_PREFIX_BACKWARD, key)
    }

    fn remove_persistent_prefix(&self, key: &str) -> Option<FastStr> {
        self.remove_prefix(RPC_PREFIX_PERSISTENT, key)
    }

    fn remove_transient_prefix(&self, key: &str) -> Option<FastStr> {
        self.remove_prefix(RPC_PREFIX_TRANSIENT, key)
    }

    fn remove_backward_prefix(&self, key: &str) -> Option<FastStr> {
        self.remove_prefix(RPC_PREFIX_BACKWARD, key)
    }
}

pub struct HttpConverter;

impl HttpConverter {
    /// Convert `RPC_PERSIST_TEST_KEY` to `rpc-persist-test-key`
    #[inline]
    fn to_http_format(&self, key: &str, buf: &mut String) {
        for ch in key.chars() {
            let ch = match ch {
                'A'..='Z' => ch.to_ascii_lowercase(),
                '_' => '-',
                _ => ch,
            };
            buf.push(ch);
        }
    }

    /// Convert `rpc-persist-test-key` to `RPC_PERSIST_TEST_KEY`
    #[inline]
    fn to_rpc_format(&self, key: &str, buf: &mut String) {
        for ch in key.chars() {
            let ch = match ch {
                'a'..='z' => ch.to_ascii_uppercase(),
                '-' => '_',
                _ => ch,
            };
            buf.push(ch);
        }
    }

    #[inline]
    fn add_prefix_and_to_http_format(&self, prefix: &'static str, key: &str) -> FastStr {
        let mut buf = String::with_capacity(prefix.len() + key.len());
        buf.push_str(prefix);
        self.to_http_format(key, &mut buf);
        FastStr::from_string(buf)
    }

    #[inline]
    fn remove_prefix_and_to_rpc_format(&self, prefix: &'static str, key: &str) -> Option<FastStr> {
        let key = key.strip_prefix(prefix)?;
        let mut buf = String::with_capacity(key.len());
        self.to_rpc_format(key, &mut buf);
        Some(FastStr::from_string(buf))
    }
}

impl Converter for HttpConverter {
    fn add_persistent_prefix(&self, key: &str) -> FastStr {
        self.add_prefix_and_to_http_format(HTTP_PREFIX_PERSISTENT, key)
    }

    fn add_transient_prefix(&self, key: &str) -> FastStr {
        self.add_prefix_and_to_http_format(HTTP_PREFIX_TRANSIENT, key)
    }

    fn add_backward_prefix(&self, key: &str) -> FastStr {
        self.add_prefix_and_to_http_format(HTTP_PREFIX_BACKWARD, key)
    }

    fn remove_persistent_prefix(&self, key: &str) -> Option<FastStr> {
        self.remove_prefix_and_to_rpc_format(HTTP_PREFIX_PERSISTENT, key)
    }

    fn remove_transient_prefix(&self, key: &str) -> Option<FastStr> {
        self.remove_prefix_and_to_rpc_format(HTTP_PREFIX_TRANSIENT, key)
    }

    fn remove_backward_prefix(&self, key: &str) -> Option<FastStr> {
        self.remove_prefix_and_to_rpc_format(HTTP_PREFIX_BACKWARD, key)
    }
}

#[cfg(test)]
mod convert_tests {
    use crate::convert::{Converter, HttpConverter, RpcConverter};

    #[test]
    fn add_rpc_prefix() {
        assert_eq!(
            RpcConverter.add_persistent_prefix("TEST_KEY"),
            "RPC_PERSIST_TEST_KEY",
        );
        assert_eq!(
            RpcConverter.add_transient_prefix("TEST_KEY"),
            "RPC_TRANSIT_TEST_KEY",
        );
        assert_eq!(
            RpcConverter.add_backward_prefix("TEST_KEY"),
            "RPC_BACKWARD_TEST_KEY",
        );
    }

    #[test]
    fn remove_rpc_prefix() {
        assert_eq!(
            RpcConverter
                .remove_persistent_prefix("RPC_PERSIST_TEST_KEY")
                .as_deref(),
            Some("TEST_KEY"),
        );
        assert_eq!(
            RpcConverter
                .remove_transient_prefix("RPC_TRANSIT_TEST_KEY")
                .as_deref(),
            Some("TEST_KEY"),
        );
        assert_eq!(
            RpcConverter
                .remove_backward_prefix("RPC_BACKWARD_TEST_KEY")
                .as_deref(),
            Some("TEST_KEY"),
        );
        assert_eq!(
            RpcConverter
                .remove_persistent_prefix("RPC_PERSIST-TEST_KEY")
                .as_deref(),
            None,
        );
        assert_eq!(
            RpcConverter
                .remove_transient_prefix("RPC-TRANSIT_TEST_KEY")
                .as_deref(),
            None,
        );
        assert_eq!(
            RpcConverter
                .remove_backward_prefix("RPC_BBBBDDDD_TEST_KEY")
                .as_deref(),
            None,
        );
    }

    #[test]
    fn rpc_prefix_bidirect() {
        // remove after add
        assert_eq!(
            RpcConverter
                .remove_persistent_prefix(&RpcConverter.add_persistent_prefix("TEST_KEY"))
                .as_deref(),
            Some("TEST_KEY"),
        );
        assert_eq!(
            RpcConverter
                .remove_transient_prefix(&RpcConverter.add_transient_prefix("TEST_KEY"))
                .as_deref(),
            Some("TEST_KEY"),
        );
        assert_eq!(
            RpcConverter
                .remove_backward_prefix(&RpcConverter.add_backward_prefix("TEST_KEY"))
                .as_deref(),
            Some("TEST_KEY"),
        );

        // add after remove
        assert_eq!(
            RpcConverter.add_persistent_prefix(
                &RpcConverter
                    .remove_persistent_prefix("RPC_PERSIST_TEST_KEY")
                    .unwrap()
            ),
            "RPC_PERSIST_TEST_KEY",
        );
        assert_eq!(
            RpcConverter.add_transient_prefix(
                &RpcConverter
                    .remove_transient_prefix("RPC_TRANSIT_TEST_KEY")
                    .unwrap()
            ),
            "RPC_TRANSIT_TEST_KEY",
        );
        assert_eq!(
            RpcConverter.add_backward_prefix(
                &RpcConverter
                    .remove_backward_prefix("RPC_BACKWARD_TEST_KEY")
                    .unwrap()
            ),
            "RPC_BACKWARD_TEST_KEY",
        );
    }

    impl HttpConverter {
        fn to_http_format_string(&self, key: &str) -> String {
            let mut buf = String::with_capacity(key.len());
            self.to_http_format(key, &mut buf);
            buf
        }

        fn to_rpc_format_string(&self, key: &str) -> String {
            let mut buf = String::with_capacity(key.len());
            self.to_rpc_format(key, &mut buf);
            buf
        }
    }

    #[test]
    fn http_format_convert_test() {
        fn check(rpc_style: &str, http_style: &str) {
            assert_eq!(HttpConverter.to_http_format_string(rpc_style), http_style);
        }
        check("RPC_PERSIST_TEST_KEY", "rpc-persist-test-key");
        check("RPC_TRANSIT_TEST_KEY", "rpc-transit-test-key");
        check("RPC_BACKWARD_TEST_KEY", "rpc-backward-test-key");
        check("TEST_KEY", "test-key");
    }

    #[test]
    fn rpc_format_convert_test() {
        fn check(http_style: &str, rpc_style: &str) {
            assert_eq!(HttpConverter.to_rpc_format_string(http_style), rpc_style);
        }
        check("rpc-persist-test-key", "RPC_PERSIST_TEST_KEY");
        check("rpc-transit-test-key", "RPC_TRANSIT_TEST_KEY");
        check("rpc-backward-test-key", "RPC_BACKWARD_TEST_KEY");
        check("test-key", "TEST_KEY");
    }

    #[test]
    fn format_bidirect_convert() {
        fn check_rpc(rpc_style: &str) {
            assert_eq!(
                HttpConverter.to_rpc_format_string(&HttpConverter.to_http_format_string(rpc_style)),
                rpc_style,
            );
        }
        fn check_http(http_style: &str) {
            assert_eq!(
                HttpConverter
                    .to_http_format_string(&HttpConverter.to_rpc_format_string(http_style)),
                http_style,
            );
        }
        check_rpc("RPC_PERSIST_TEST_KEY");
        check_rpc("RPC_TRANSIT_TEST_KEY");
        check_rpc("RPC_BACKWARD_TEST_KEY");
        check_rpc("TEST_KEY");
        check_http("rpc-persist-test-key");
        check_http("rpc-transit-test-key");
        check_http("rpc-backward-test-key");
        check_http("test-key");
    }

    #[test]
    fn add_http_prefix() {
        assert_eq!(
            HttpConverter.add_persistent_prefix("TEST_KEY"),
            "rpc-persist-test-key",
        );
        assert_eq!(
            HttpConverter.add_transient_prefix("TEST_KEY"),
            "rpc-transit-test-key",
        );
        assert_eq!(
            HttpConverter.add_backward_prefix("TEST_KEY"),
            "rpc-backward-test-key",
        );
    }

    #[test]
    fn remove_http_prefix() {
        assert_eq!(
            HttpConverter
                .remove_persistent_prefix("rpc-persist-test-key")
                .as_deref(),
            Some("TEST_KEY"),
        );
        assert_eq!(
            HttpConverter
                .remove_transient_prefix("rpc-transit-test-key")
                .as_deref(),
            Some("TEST_KEY"),
        );
        assert_eq!(
            HttpConverter
                .remove_backward_prefix("rpc-backward-test-key")
                .as_deref(),
            Some("TEST_KEY"),
        );
        assert_eq!(
            HttpConverter
                .remove_persistent_prefix("rpc-persist_test-key")
                .as_deref(),
            None,
        );
        assert_eq!(
            HttpConverter
                .remove_transient_prefix("rpc_transit-test-key")
                .as_deref(),
            None,
        );
        assert_eq!(
            HttpConverter
                .remove_backward_prefix("rpc-bbbbdddd-test-key")
                .as_deref(),
            None,
        );
    }

    #[test]
    fn http_prefix_bidirect() {
        // remove after add
        assert_eq!(
            HttpConverter
                .remove_persistent_prefix(&HttpConverter.add_persistent_prefix("TEST_KEY"))
                .as_deref(),
            Some("TEST_KEY"),
        );
        assert_eq!(
            HttpConverter
                .remove_transient_prefix(&HttpConverter.add_transient_prefix("TEST_KEY"))
                .as_deref(),
            Some("TEST_KEY"),
        );
        assert_eq!(
            HttpConverter
                .remove_backward_prefix(&HttpConverter.add_backward_prefix("TEST_KEY"))
                .as_deref(),
            Some("TEST_KEY"),
        );

        // add after remove
        assert_eq!(
            HttpConverter.add_persistent_prefix(
                &HttpConverter
                    .remove_persistent_prefix("rpc-persist-test-key")
                    .unwrap()
            ),
            "rpc-persist-test-key",
        );
        assert_eq!(
            HttpConverter.add_transient_prefix(
                &HttpConverter
                    .remove_transient_prefix("rpc-transit-test-key")
                    .unwrap()
            ),
            "rpc-transit-test-key",
        );
        assert_eq!(
            HttpConverter.add_backward_prefix(
                &HttpConverter
                    .remove_backward_prefix("rpc-backward-test-key")
                    .unwrap()
            ),
            "rpc-backward-test-key",
        );
    }
}
