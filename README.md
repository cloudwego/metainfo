# Metainfo

[![Crates.io](https://img.shields.io/crates/v/metainfo)](https://crates.io/crates/metainfo)
[![Documentation](https://docs.rs/metainfo/badge.svg)](https://docs.rs/metainfo)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/cloudwego/metainfo)
[![License](https://img.shields.io/crates/l/metainfo)](#license)
[![Build Status][actions-badge]][actions-url]

[actions-badge]: https://github.com/cloudwego/metainfo/actions/workflows/ci.yaml/badge.svg
[actions-url]: https://github.com/cloudwego/metainfo/actions

Transmissing metainfo across components.

## Quickstart

Metainfo is designed to be passed through task local, so we provided a unified key for it `metainfo::METAINFO`, and we recommend you to use it this way:

```rust
METAINFO.scope(...)
```

`MetaInfo` is used to passthrough information between components and even client-server.

It supports two types of info: typed map and string k-v.

It is designed to be tree-like, which means you can share a `MetaInfo` with multiple children.

Note: only the current scope is mutable.

Example:

```rust
use metainfo::MetaInfo;

fn test() {
    let mut m1 = MetaInfo::new();
    m1.insert::<i8>(2);
    assert_eq!(*m1.get::<i8>().unwrap(), 2);

    let (mut m1, mut m2) = m1.derive();
    assert_eq!(*m2.get::<i8>().unwrap(), 2);

    m2.insert::<i8>(4);
    assert_eq!(*m2.get::<i8>().unwrap(), 4);

    m2.remove::<i8>();
    assert_eq!(*m2.get::<i8>().unwrap(), 2);
}
```

## Related Projects

- [Volo][Volo]: A high-performance and strong-extensibility Rust RPC framework that helps developers build microservices.
- [Volo-rs][Volo-rs]: The volo ecosystem which contains a lot of useful components.
- [Motore][Motore]: Middleware abstraction layer powered by GAT.
- [Pilota][Pilota]: A thrift and protobuf implementation in pure rust with high performance and extensibility.

## Contributing

See [CONTRIBUTING.md](https://github.com/cloudwego/metainfo/blob/main/CONTRIBUTING.md) for more information.

## License

Metainfo is dual-licensed under the MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](https://github.com/cloudwego/metainfo/blob/main/LICENSE-MIT) and [LICENSE-APACHE](https://github.com/cloudwego/metainfo/blob/main/LICENSE-APACHE) for details.

## Community

- Email: [volo@cloudwego.io](mailto:volo@cloudwego.io)
- How to become a member: [COMMUNITY MEMBERSHIP](https://github.com/cloudwego/community/blob/main/COMMUNITY_MEMBERSHIP.md)
- Issues: [Issues](https://github.com/cloudwego/metainfo/issues)
- Feishu: Scan the QR code below with [Feishu](https://www.feishu.cn/) or [click this link](https://applink.feishu.cn/client/chat/chatter/add_by_link?link_token=b34v5470-8e4d-4c7d-bf50-8b2917af026b) to join our CloudWeGo Volo user group.

  <img src="https://github.com/cloudwego/metainfo/raw/main/.github/assets/volo-feishu-user-group.png" alt="Volo user group" width="50%" height="50%" />

[Volo]: https://github.com/cloudwego/volo
[Volo-rs]: https://github.com/volo-rs
[Motore]: https://github.com/cloudwego/motore
[Pilota]: https://github.com/cloudwego/pilota
[Metainfo]: https://github.com/lust-rs/metainfo
