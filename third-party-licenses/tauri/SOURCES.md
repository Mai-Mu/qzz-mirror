# Tauri and Rust runtime license sources

The JS API version matches `package-lock.json`. Rust crate versions come from `src-tauri/Cargo.lock` and the installed Cargo registry.
All listed license and notice files were copied byte for byte; SHA-256 values below identify the copied bytes.
`cargo-registry/<crate>-<version>/` denotes the extracted upstream crate package, not a user-specific absolute path.

## Scope

Windows x86-64 normal dependencies selected with:

```text
cargo tree --offline --locked --target x86_64-pc-windows-msvc --manifest-path src-tauri/Cargo.toml --edges normal,no-proc-macro --prefix none --format {p}
```

Build dependencies and procedural macro crates are excluded. The normal dependency graph is an inventory, not proof that every crate contributes code to the final linked executable.
Other applicable licenses (such as Unicode, BSD, MPL and Zlib) are preserved where present in these runtime packages.
This directory does not constitute a complete compliance certification or a complete inventory of the Rust standard library, native SDKs, WebView2 redistribution terms, or external scrcpy/adb components.

## Repository supplements and remaining limits

The installed crate archives for `alloc-stdlib 0.3.0`, `webview2-com 0.39.1`, and `webview2-com-sys 0.39.1` omit the repository-level license file. Their `LICENSE` texts were retrieved from the official repositories at the exact commit recorded in each archive's `.cargo_vcs_info.json`. The URLs and SHA-256 values are included below and in `FILES.json`.

The known missing top-level crate license texts in this scoped inventory have been filled. This does not establish completeness of file-level attribution, native SDK/loader terms, the Rust standard library, or other platform redistribution terms. The project also relies on the system-installed WebView2 runtime; that runtime is not included by this license collection.

## Copied texts

| Package | Version | Declared license | Original source | Artifact file | SHA-256 |
|---|---|---|---|---|---|
| `adler2` | `2.0.1` | `0BSD OR MIT OR Apache-2.0` | `cargo-registry/adler2-2.0.1/LICENSE-0BSD` | `tauri/adler2-2.0.1/LICENSE-0BSD` | `861399f8c21c042b110517e76dc6b63a2b334276c8cf17412fc3c8908ca8dc17` |
| `adler2` | `2.0.1` | `0BSD OR MIT OR Apache-2.0` | `cargo-registry/adler2-2.0.1/LICENSE-APACHE` | `tauri/adler2-2.0.1/LICENSE-APACHE` | `8ada45cd9f843acf64e4722ae262c622a2b3b3007c7310ef36ac1061a30f6adb` |
| `adler2` | `2.0.1` | `0BSD OR MIT OR Apache-2.0` | `cargo-registry/adler2-2.0.1/LICENSE-MIT` | `tauri/adler2-2.0.1/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `aho-corasick` | `1.1.5` | `Unlicense OR MIT` | `cargo-registry/aho-corasick-1.1.5/COPYING` | `tauri/aho-corasick-1.1.5/COPYING` | `01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f` |
| `aho-corasick` | `1.1.5` | `Unlicense OR MIT` | `cargo-registry/aho-corasick-1.1.5/LICENSE-MIT` | `tauri/aho-corasick-1.1.5/LICENSE-MIT` | `0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f` |
| `alloc-no-stdlib` | `3.0.0` | `BSD-3-Clause` | `cargo-registry/alloc-no-stdlib-3.0.0/LICENSE` | `tauri/alloc-no-stdlib-3.0.0/LICENSE` | `c0c56f26d9c051cac4d200c34c84e7ae9aaa853e01a982a1df08b09931e518ae` |
| `alloc-stdlib` | `0.3.0` | `BSD-3-Clause` | `https://raw.githubusercontent.com/dropbox/rust-alloc-no-stdlib/0a81fd6928ea3b33c8cd484aa4575d50ffb98012/LICENSE` | `tauri/alloc-stdlib-0.3.0/LICENSE` | `c0c56f26d9c051cac4d200c34c84e7ae9aaa853e01a982a1df08b09931e518ae` |
| `anyhow` | `1.0.104` | `MIT OR Apache-2.0` | `cargo-registry/anyhow-1.0.104/LICENSE-APACHE` | `tauri/anyhow-1.0.104/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `anyhow` | `1.0.104` | `MIT OR Apache-2.0` | `cargo-registry/anyhow-1.0.104/LICENSE-MIT` | `tauri/anyhow-1.0.104/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `base64` | `0.23.1` | `MIT OR Apache-2.0` | `cargo-registry/base64-0.23.1/LICENSE-APACHE` | `tauri/base64-0.23.1/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `base64` | `0.23.1` | `MIT OR Apache-2.0` | `cargo-registry/base64-0.23.1/LICENSE-MIT` | `tauri/base64-0.23.1/LICENSE-MIT` | `ab499c75a0f0da8e0fe83bf9ba3a27e5c39705310f07107c6c66b69958d2401c` |
| `bitflags` | `2.13.2` | `MIT OR Apache-2.0` | `cargo-registry/bitflags-2.13.2/LICENSE-APACHE` | `tauri/bitflags-2.13.2/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `bitflags` | `2.13.2` | `MIT OR Apache-2.0` | `cargo-registry/bitflags-2.13.2/LICENSE-MIT` | `tauri/bitflags-2.13.2/LICENSE-MIT` | `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| `brotli` | `9.1.0` | `BSD-3-Clause AND MIT` | `cargo-registry/brotli-9.1.0/LICENSE.BSD-3-Clause` | `tauri/brotli-9.1.0/LICENSE.BSD-3-Clause` | `c0c56f26d9c051cac4d200c34c84e7ae9aaa853e01a982a1df08b09931e518ae` |
| `brotli` | `9.1.0` | `BSD-3-Clause AND MIT` | `cargo-registry/brotli-9.1.0/LICENSE.MIT` | `tauri/brotli-9.1.0/LICENSE.MIT` | `3d180008e36922a4e8daec11c34c7af264fed5962d07924aea928c38e8663c94` |
| `brotli-decompressor` | `6.2.0` | `BSD-3-Clause/MIT` | `cargo-registry/brotli-decompressor-6.2.0/LICENSE` | `tauri/brotli-decompressor-6.2.0/LICENSE` | `c0c56f26d9c051cac4d200c34c84e7ae9aaa853e01a982a1df08b09931e518ae` |
| `bytemuck` | `1.25.2` | `Zlib OR Apache-2.0 OR MIT` | `cargo-registry/bytemuck-1.25.2/LICENSE-APACHE` | `tauri/bytemuck-1.25.2/LICENSE-APACHE` | `e3ba223bb1423f0aad8c3dfce0fe3148db48926d41e6fbc3afbbf5ff9e1c89cb` |
| `bytemuck` | `1.25.2` | `Zlib OR Apache-2.0 OR MIT` | `cargo-registry/bytemuck-1.25.2/LICENSE-MIT` | `tauri/bytemuck-1.25.2/LICENSE-MIT` | `9df9ba60a11af705f2e451b53762686e615d86f76b169cf075c3237730dbd7e2` |
| `bytemuck` | `1.25.2` | `Zlib OR Apache-2.0 OR MIT` | `cargo-registry/bytemuck-1.25.2/LICENSE-ZLIB` | `tauri/bytemuck-1.25.2/LICENSE-ZLIB` | `84b34dd7608f7fb9b17bd588a6bf392bf7de504e2716f024a77d89f1b145a151` |
| `byteorder-lite` | `0.1.0` | `Unlicense OR MIT` | `cargo-registry/byteorder-lite-0.1.0/LICENSE-MIT` | `tauri/byteorder-lite-0.1.0/LICENSE-MIT` | `0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f` |
| `bytes` | `1.12.1` | `MIT` | `cargo-registry/bytes-1.12.1/LICENSE` | `tauri/bytes-1.12.1/LICENSE` | `45f522cacecb1023856e46df79ca625dfc550c94910078bd8aec6e02880b3d42` |
| `cfb` | `0.14.0` | `MIT` | `cargo-registry/cfb-0.14.0/LICENSE` | `tauri/cfb-0.14.0/LICENSE` | `ebaac853b53fec0ed475796f533ce676912a4df9ad9d8a621fa76d5060591e10` |
| `cfg-if` | `1.0.5` | `MIT OR Apache-2.0` | `cargo-registry/cfg-if-1.0.5/LICENSE-APACHE` | `tauri/cfg-if-1.0.5/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `cfg-if` | `1.0.5` | `MIT OR Apache-2.0` | `cargo-registry/cfg-if-1.0.5/LICENSE-MIT` | `tauri/cfg-if-1.0.5/LICENSE-MIT` | `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| `cookie` | `0.18.2` | `MIT OR Apache-2.0` | `cargo-registry/cookie-0.18.2/LICENSE-APACHE` | `tauri/cookie-0.18.2/LICENSE-APACHE` | `2773e20df8f4c52a026b5b578c7f2457341f5aa3fb6612fd87e1d2e1bd8f48ad` |
| `cookie` | `0.18.2` | `MIT OR Apache-2.0` | `cargo-registry/cookie-0.18.2/LICENSE-MIT` | `tauri/cookie-0.18.2/LICENSE-MIT` | `f28420c1906af38be726cd7842798f0194b27c41d6051c0d1e9ee348b8a4a0ea` |
| `crc32fast` | `1.5.3` | `MIT OR Apache-2.0` | `cargo-registry/crc32fast-1.5.3/LICENSE-APACHE` | `tauri/crc32fast-1.5.3/LICENSE-APACHE` | `c6596eb7be8581c18be736c846fb9173b69eccf6ef94c5135893ec56bd92ba08` |
| `crc32fast` | `1.5.3` | `MIT OR Apache-2.0` | `cargo-registry/crc32fast-1.5.3/LICENSE-MIT` | `tauri/crc32fast-1.5.3/LICENSE-MIT` | `61d383b05b87d78f94d2937e2580cce47226d17823c0430fbcad09596537efcf` |
| `crossbeam-channel` | `0.5.17` | `MIT OR Apache-2.0` | `cargo-registry/crossbeam-channel-0.5.17/LICENSE-APACHE` | `tauri/crossbeam-channel-0.5.17/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `crossbeam-channel` | `0.5.17` | `MIT OR Apache-2.0` | `cargo-registry/crossbeam-channel-0.5.17/LICENSE-MIT` | `tauri/crossbeam-channel-0.5.17/LICENSE-MIT` | `5734ed989dfca1f625b40281ee9f4530f91b2411ec01cb748223e7eb87e201ab` |
| `crossbeam-channel` | `0.5.17` | `MIT OR Apache-2.0` | `cargo-registry/crossbeam-channel-0.5.17/LICENSE-THIRD-PARTY` | `tauri/crossbeam-channel-0.5.17/LICENSE-THIRD-PARTY` | `b16db96b93b1d7cf7bea533f572091ec6bca3234fbe0a83038be772ff391a44c` |
| `crossbeam-utils` | `0.8.23` | `MIT OR Apache-2.0` | `cargo-registry/crossbeam-utils-0.8.23/LICENSE-APACHE` | `tauri/crossbeam-utils-0.8.23/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `crossbeam-utils` | `0.8.23` | `MIT OR Apache-2.0` | `cargo-registry/crossbeam-utils-0.8.23/LICENSE-MIT` | `tauri/crossbeam-utils-0.8.23/LICENSE-MIT` | `5734ed989dfca1f625b40281ee9f4530f91b2411ec01cb748223e7eb87e201ab` |
| `ctor` | `1.0.13` | `Apache-2.0 OR MIT` | `cargo-registry/ctor-1.0.13/LICENSE-APACHE` | `tauri/ctor-1.0.13/LICENSE-APACHE` | `a8ad31b1c3f40dca5a84119351b8fa8ddc868edd77fad8a8ebf6d8f2d16fa4ae` |
| `ctor` | `1.0.13` | `Apache-2.0 OR MIT` | `cargo-registry/ctor-1.0.13/LICENSE-MIT` | `tauri/ctor-1.0.13/LICENSE-MIT` | `bccaa8b6c09f94e81f06696e179dbe058464bbdfbc823b6d49cada1d71e84ac3` |
| `deranged` | `0.5.9` | `MIT OR Apache-2.0` | `cargo-registry/deranged-0.5.9/LICENSE-Apache` | `tauri/deranged-0.5.9/LICENSE-Apache` | `edd65bdd88957a205c47d53fa499eed8865a70320f0f03f6391668cb304ea376` |
| `deranged` | `0.5.9` | `MIT OR Apache-2.0` | `cargo-registry/deranged-0.5.9/LICENSE-MIT` | `tauri/deranged-0.5.9/LICENSE-MIT` | `231c837c45eb53f108fb48929e488965bc4fcc14e9ea21d35f50e6b99d98685b` |
| `dirs` | `7.0.0` | `MIT OR Apache-2.0` | `cargo-registry/dirs-7.0.0/LICENSE-APACHE` | `tauri/dirs-7.0.0/LICENSE-APACHE` | `d3174ad63e721d4c9dccb8ad4320848992d314369bc46319720b5802c9153fe9` |
| `dirs` | `7.0.0` | `MIT OR Apache-2.0` | `cargo-registry/dirs-7.0.0/LICENSE-MIT` | `tauri/dirs-7.0.0/LICENSE-MIT` | `6a2e0ade09a7d5f816f11566fee2b151b32235a7fad52b41d49cce96f833c1a9` |
| `dirs-sys` | `0.5.0` | `MIT OR Apache-2.0` | `cargo-registry/dirs-sys-0.5.0/LICENSE-APACHE` | `tauri/dirs-sys-0.5.0/LICENSE-APACHE` | `d3174ad63e721d4c9dccb8ad4320848992d314369bc46319720b5802c9153fe9` |
| `dirs-sys` | `0.5.0` | `MIT OR Apache-2.0` | `cargo-registry/dirs-sys-0.5.0/LICENSE-MIT` | `tauri/dirs-sys-0.5.0/LICENSE-MIT` | `6a2e0ade09a7d5f816f11566fee2b151b32235a7fad52b41d49cce96f833c1a9` |
| `dpi` | `0.1.2` | `Apache-2.0 AND MIT` | `cargo-registry/dpi-0.1.2/LICENSE` | `tauri/dpi-0.1.2/LICENSE` | `6dc0e068dcf3a5bc8e054205b85b7720e1d49265bbc64bf515d2cf79197df69a` |
| `dpi` | `0.1.2` | `Apache-2.0 AND MIT` | `cargo-registry/dpi-0.1.2/LICENSE-LIBM-MIT` | `tauri/dpi-0.1.2/LICENSE-LIBM-MIT` | `9eff396a99bafd208e7126602420d6864f6066c08b84485a5b032320a9c01997` |
| `dunce` | `1.0.5` | `CC0-1.0 OR MIT-0 OR Apache-2.0` | `cargo-registry/dunce-1.0.5/LICENSE` | `tauri/dunce-1.0.5/LICENSE` | `a2010f343487d3f7618affe54f789f5487602331c0a8d03f49e9a7c547cf0499` |
| `equivalent` | `1.0.3` | `Apache-2.0 OR MIT` | `cargo-registry/equivalent-1.0.3/LICENSE-APACHE` | `tauri/equivalent-1.0.3/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `equivalent` | `1.0.3` | `Apache-2.0 OR MIT` | `cargo-registry/equivalent-1.0.3/LICENSE-MIT` | `tauri/equivalent-1.0.3/LICENSE-MIT` | `7365cc8878a1d7ce155a58c4ca09c3d7a6be413efa5334a80ea842912b669349` |
| `erased-serde` | `0.4.11` | `MIT OR Apache-2.0` | `cargo-registry/erased-serde-0.4.11/LICENSE-APACHE` | `tauri/erased-serde-0.4.11/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `erased-serde` | `0.4.11` | `MIT OR Apache-2.0` | `cargo-registry/erased-serde-0.4.11/LICENSE-MIT` | `tauri/erased-serde-0.4.11/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `fdeflate` | `0.3.7` | `MIT OR Apache-2.0` | `cargo-registry/fdeflate-0.3.7/LICENSE-APACHE` | `tauri/fdeflate-0.3.7/LICENSE-APACHE` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `fdeflate` | `0.3.7` | `MIT OR Apache-2.0` | `cargo-registry/fdeflate-0.3.7/LICENSE-MIT` | `tauri/fdeflate-0.3.7/LICENSE-MIT` | `c77a4cf9da729987d0fe7ccd811e3bd27393914ddf3d23467c18cc22954513b3` |
| `flate2` | `1.1.10` | `MIT OR Apache-2.0` | `cargo-registry/flate2-1.1.10/LICENSE-APACHE` | `tauri/flate2-1.1.10/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `flate2` | `1.1.10` | `MIT OR Apache-2.0` | `cargo-registry/flate2-1.1.10/LICENSE-MIT` | `tauri/flate2-1.1.10/LICENSE-MIT` | `025436edff4cfcdde17a5811fdea78892d8482efd1abdec5a17872d07a4f2112` |
| `fnv` | `1.0.7` | `Apache-2.0 / MIT` | `cargo-registry/fnv-1.0.7/LICENSE-APACHE` | `tauri/fnv-1.0.7/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `fnv` | `1.0.7` | `Apache-2.0 / MIT` | `cargo-registry/fnv-1.0.7/LICENSE-MIT` | `tauri/fnv-1.0.7/LICENSE-MIT` | `65fdb6c76cd61612070c066eec9ecdb30ee74fb27859d0d9af58b9f499fd0c3e` |
| `form_urlencoded` | `1.2.2` | `MIT OR Apache-2.0` | `cargo-registry/form_urlencoded-1.2.2/LICENSE-APACHE` | `tauri/form_urlencoded-1.2.2/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `form_urlencoded` | `1.2.2` | `MIT OR Apache-2.0` | `cargo-registry/form_urlencoded-1.2.2/LICENSE-MIT` | `tauri/form_urlencoded-1.2.2/LICENSE-MIT` | `20c7855c364d57ea4c97889a5e8d98470a9952dade37bd9248b9a54431670e5e` |
| `getrandom` | `0.3.4` | `MIT OR Apache-2.0` | `cargo-registry/getrandom-0.3.4/LICENSE-APACHE` | `tauri/getrandom-0.3.4/LICENSE-APACHE` | `aaff376532ea30a0cd5330b9502ad4a4c8bf769c539c87ffe78819d188a18ebf` |
| `getrandom` | `0.3.4` | `MIT OR Apache-2.0` | `cargo-registry/getrandom-0.3.4/LICENSE-MIT` | `tauri/getrandom-0.3.4/LICENSE-MIT` | `29e9fe5074bd27e0e5d5d110394fbbcd841baee2651a3c4b4560a632702cede4` |
| `glob` | `0.3.4` | `MIT OR Apache-2.0` | `cargo-registry/glob-0.3.4/LICENSE-APACHE` | `tauri/glob-0.3.4/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `glob` | `0.3.4` | `MIT OR Apache-2.0` | `cargo-registry/glob-0.3.4/LICENSE-MIT` | `tauri/glob-0.3.4/LICENSE-MIT` | `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| `hashbrown` | `0.17.1` | `MIT OR Apache-2.0` | `cargo-registry/hashbrown-0.17.1/LICENSE-APACHE` | `tauri/hashbrown-0.17.1/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `hashbrown` | `0.17.1` | `MIT OR Apache-2.0` | `cargo-registry/hashbrown-0.17.1/LICENSE-MIT` | `tauri/hashbrown-0.17.1/LICENSE-MIT` | `ff8f68cb076caf8cefe7a6430d4ac086ce6af2ca8ce2c4e5a2004d4552ef52a2` |
| `heck` | `0.5.0` | `MIT OR Apache-2.0` | `cargo-registry/heck-0.5.0/LICENSE-APACHE` | `tauri/heck-0.5.0/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `heck` | `0.5.0` | `MIT OR Apache-2.0` | `cargo-registry/heck-0.5.0/LICENSE-MIT` | `tauri/heck-0.5.0/LICENSE-MIT` | `7b63ecd5f1902af1b63729947373683c32745c16a10e8e6292e2e2dcd7e90ae0` |
| `http` | `1.5.0` | `MIT OR Apache-2.0` | `cargo-registry/http-1.5.0/LICENSE-APACHE` | `tauri/http-1.5.0/LICENSE-APACHE` | `8bb1b50b0e5c9399ae33bd35fab2769010fa6c14e8860c729a52295d84896b7a` |
| `http` | `1.5.0` | `MIT OR Apache-2.0` | `cargo-registry/http-1.5.0/LICENSE-MIT` | `tauri/http-1.5.0/LICENSE-MIT` | `dc91f8200e4b2a1f9261035d4c18c33c246911a6c0f7b543d75347e61b249cff` |
| `icu_collections` | `2.4.0` | `Unicode-3.0` | `cargo-registry/icu_collections-2.4.0/LICENSE` | `tauri/icu_collections-2.4.0/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `icu_locale_core` | `2.4.0` | `Unicode-3.0` | `cargo-registry/icu_locale_core-2.4.0/LICENSE` | `tauri/icu_locale_core-2.4.0/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `icu_normalizer` | `2.4.0` | `Unicode-3.0` | `cargo-registry/icu_normalizer-2.4.0/LICENSE` | `tauri/icu_normalizer-2.4.0/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `icu_normalizer_data` | `2.4.0` | `Unicode-3.0` | `cargo-registry/icu_normalizer_data-2.4.0/LICENSE` | `tauri/icu_normalizer_data-2.4.0/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `icu_properties` | `2.4.0` | `Unicode-3.0` | `cargo-registry/icu_properties-2.4.0/LICENSE` | `tauri/icu_properties-2.4.0/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `icu_properties_data` | `2.4.0` | `Unicode-3.0` | `cargo-registry/icu_properties_data-2.4.0/LICENSE` | `tauri/icu_properties_data-2.4.0/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `icu_provider` | `2.4.0` | `Unicode-3.0` | `cargo-registry/icu_provider-2.4.0/LICENSE` | `tauri/icu_provider-2.4.0/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `idna` | `1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/idna-1.1.0/LICENSE-APACHE` | `tauri/idna-1.1.0/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `idna` | `1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/idna-1.1.0/LICENSE-MIT` | `tauri/idna-1.1.0/LICENSE-MIT` | `b38f11f6096706e6de553dabe2a7ed142d59b6fa8c97e290c67496154745cdd5` |
| `idna_adapter` | `1.2.2` | `Apache-2.0 OR MIT` | `cargo-registry/idna_adapter-1.2.2/LICENSE-APACHE` | `tauri/idna_adapter-1.2.2/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `idna_adapter` | `1.2.2` | `Apache-2.0 OR MIT` | `cargo-registry/idna_adapter-1.2.2/LICENSE-MIT` | `tauri/idna_adapter-1.2.2/LICENSE-MIT` | `8b43ce8accd61e9d370b5ca9e9c4f953279b5c239926c62315b40e24df51b726` |
| `image` | `0.25.10` | `MIT OR Apache-2.0` | `cargo-registry/image-0.25.10/LICENSE-APACHE` | `tauri/image-0.25.10/LICENSE-APACHE` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `image` | `0.25.10` | `MIT OR Apache-2.0` | `cargo-registry/image-0.25.10/LICENSE-MIT` | `tauri/image-0.25.10/LICENSE-MIT` | `c77a4cf9da729987d0fe7ccd811e3bd27393914ddf3d23467c18cc22954513b3` |
| `indexmap` | `2.14.2` | `Apache-2.0 OR MIT` | `cargo-registry/indexmap-2.14.2/LICENSE-APACHE` | `tauri/indexmap-2.14.2/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `indexmap` | `2.14.2` | `Apache-2.0 OR MIT` | `cargo-registry/indexmap-2.14.2/LICENSE-MIT` | `tauri/indexmap-2.14.2/LICENSE-MIT` | `ecc269ef87fd38a1d98e30bfac9ba964a9dbd9315c3770fed98d4d7cb5882055` |
| `infer` | `0.22.0` | `MIT` | `cargo-registry/infer-0.22.0/LICENSE` | `tauri/infer-0.22.0/LICENSE` | `270b0d81891aa44c7e8726cf9432dade9556a01c848bacd2f3ef839c9c1a1859` |
| `itoa` | `1.0.18` | `MIT OR Apache-2.0` | `cargo-registry/itoa-1.0.18/LICENSE-APACHE` | `tauri/itoa-1.0.18/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `itoa` | `1.0.18` | `MIT OR Apache-2.0` | `cargo-registry/itoa-1.0.18/LICENSE-MIT` | `tauri/itoa-1.0.18/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `json-patch` | `4.2.0` | `MIT/Apache-2.0` | `cargo-registry/json-patch-4.2.0/LICENSE-APACHE` | `tauri/json-patch-4.2.0/LICENSE-APACHE` | `b40930bbcf80744c86c46a12bc9da056641d722716c378f5659b9e555ef833e1` |
| `json-patch` | `4.2.0` | `MIT/Apache-2.0` | `cargo-registry/json-patch-4.2.0/LICENSE-MIT` | `tauri/json-patch-4.2.0/LICENSE-MIT` | `87f7527b1066337751f7f4d717089bf514ff8f7ee7d66c4c5b891b09e744ec11` |
| `jsonptr` | `0.7.1` | `MIT OR Apache-2.0` | `cargo-registry/jsonptr-0.7.1/LICENSE-APACHE` | `tauri/jsonptr-0.7.1/LICENSE-APACHE` | `2cf31f2ae7f3feba667d50fc1d4d21ecf217703c04c3068b83c9a73b6dde723a` |
| `jsonptr` | `0.7.1` | `MIT OR Apache-2.0` | `cargo-registry/jsonptr-0.7.1/LICENSE-MIT` | `tauri/jsonptr-0.7.1/LICENSE-MIT` | `2bc107bdc18a74263d20bdefdb8073e17bc1a01b6b374dc5835683a33ee5ba7d` |
| `keyboard-types` | `0.8.3` | `MIT OR Apache-2.0` | `cargo-registry/keyboard-types-0.8.3/LICENSE-APACHE` | `tauri/keyboard-types-0.8.3/LICENSE-APACHE` | `769f80b5bcb42ed0af4e4d2fd74e1ac9bf843cb80c5a29219d1ef3544428a6bb` |
| `keyboard-types` | `0.8.3` | `MIT OR Apache-2.0` | `cargo-registry/keyboard-types-0.8.3/LICENSE-MIT` | `tauri/keyboard-types-0.8.3/LICENSE-MIT` | `31dbbab009f1b2e59a1622d5955dfafc38ebe834000215b49d954c6f85fe927c` |
| `libc` | `0.2.191` | `MIT OR Apache-2.0` | `cargo-registry/libc-0.2.191/LICENSE-APACHE` | `tauri/libc-0.2.191/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `libc` | `0.2.191` | `MIT OR Apache-2.0` | `cargo-registry/libc-0.2.191/LICENSE-MIT` | `tauri/libc-0.2.191/LICENSE-MIT` | `123a331b5dbf04c30097fa43b8f858bc85df671fe776de498d01f3d6b7c1f69e` |
| `litemap` | `0.8.3` | `Unicode-3.0` | `cargo-registry/litemap-0.8.3/LICENSE` | `tauri/litemap-0.8.3/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `lock_api` | `0.4.14` | `MIT OR Apache-2.0` | `cargo-registry/lock_api-0.4.14/LICENSE-APACHE` | `tauri/lock_api-0.4.14/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `lock_api` | `0.4.14` | `MIT OR Apache-2.0` | `cargo-registry/lock_api-0.4.14/LICENSE-MIT` | `tauri/lock_api-0.4.14/LICENSE-MIT` | `c9a75f18b9ab2927829a208fc6aa2cf4e63b8420887ba29cdb265d6619ae82d5` |
| `log` | `0.4.34` | `MIT OR Apache-2.0` | `cargo-registry/log-0.4.34/LICENSE-APACHE` | `tauri/log-0.4.34/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `log` | `0.4.34` | `MIT OR Apache-2.0` | `cargo-registry/log-0.4.34/LICENSE-MIT` | `tauri/log-0.4.34/LICENSE-MIT` | `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| `memchr` | `2.8.3` | `Unlicense OR MIT` | `cargo-registry/memchr-2.8.3/COPYING` | `tauri/memchr-2.8.3/COPYING` | `01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f` |
| `memchr` | `2.8.3` | `Unlicense OR MIT` | `cargo-registry/memchr-2.8.3/LICENSE-MIT` | `tauri/memchr-2.8.3/LICENSE-MIT` | `0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f` |
| `mime` | `0.3.17` | `MIT OR Apache-2.0` | `cargo-registry/mime-0.3.17/LICENSE-APACHE` | `tauri/mime-0.3.17/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `mime` | `0.3.17` | `MIT OR Apache-2.0` | `cargo-registry/mime-0.3.17/LICENSE-MIT` | `tauri/mime-0.3.17/LICENSE-MIT` | `df9cfd06d8a44d9a671eadd39ffd97f166481da015a30f45dfd27886209c5922` |
| `miniz_oxide` | `0.8.9` | `MIT OR Zlib OR Apache-2.0` | `cargo-registry/miniz_oxide-0.8.9/LICENSE` | `tauri/miniz_oxide-0.8.9/LICENSE` | `4108245a1f2df9d4e94df8abed5b4ba0759bb2f9b40a6b939f1be141077ae50b` |
| `miniz_oxide` | `0.8.9` | `MIT OR Zlib OR Apache-2.0` | `cargo-registry/miniz_oxide-0.8.9/LICENSE-APACHE.md` | `tauri/miniz_oxide-0.8.9/LICENSE-APACHE.md` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `miniz_oxide` | `0.8.9` | `MIT OR Zlib OR Apache-2.0` | `cargo-registry/miniz_oxide-0.8.9/LICENSE-MIT.md` | `tauri/miniz_oxide-0.8.9/LICENSE-MIT.md` | `799e9ca9d179295ef372f25d3769cdda7d25bb2668add6a6a1e22d1e4c678b8d` |
| `miniz_oxide` | `0.8.9` | `MIT OR Zlib OR Apache-2.0` | `cargo-registry/miniz_oxide-0.8.9/LICENSE-ZLIB.md` | `tauri/miniz_oxide-0.8.9/LICENSE-ZLIB.md` | `0a54e647fe54104658b5e563c04c6f9edf251710e47bce692e0bd990a4ddaa39` |
| `miniz_oxide` | `0.9.1` | `MIT OR Zlib OR Apache-2.0` | `cargo-registry/miniz_oxide-0.9.1/LICENSE` | `tauri/miniz_oxide-0.9.1/LICENSE` | `4108245a1f2df9d4e94df8abed5b4ba0759bb2f9b40a6b939f1be141077ae50b` |
| `miniz_oxide` | `0.9.1` | `MIT OR Zlib OR Apache-2.0` | `cargo-registry/miniz_oxide-0.9.1/LICENSE-APACHE.md` | `tauri/miniz_oxide-0.9.1/LICENSE-APACHE.md` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `miniz_oxide` | `0.9.1` | `MIT OR Zlib OR Apache-2.0` | `cargo-registry/miniz_oxide-0.9.1/LICENSE-MIT.md` | `tauri/miniz_oxide-0.9.1/LICENSE-MIT.md` | `799e9ca9d179295ef372f25d3769cdda7d25bb2668add6a6a1e22d1e4c678b8d` |
| `miniz_oxide` | `0.9.1` | `MIT OR Zlib OR Apache-2.0` | `cargo-registry/miniz_oxide-0.9.1/LICENSE-ZLIB.md` | `tauri/miniz_oxide-0.9.1/LICENSE-ZLIB.md` | `0a54e647fe54104658b5e563c04c6f9edf251710e47bce692e0bd990a4ddaa39` |
| `mio` | `1.2.4` | `MIT` | `cargo-registry/mio-1.2.4/LICENSE` | `tauri/mio-1.2.4/LICENSE` | `07919255c7e04793d8ea760d6c2ce32d19f9ff02bdbdde3ce90b1e1880929a9b` |
| `moxcms` | `0.8.1` | `BSD-3-Clause OR Apache-2.0` | `cargo-registry/moxcms-0.8.1/LICENSE-APACHE.md` | `tauri/moxcms-0.8.1/LICENSE-APACHE.md` | `90bf2d659c43045111b65c733ab2a6d4cbcb422a098368c8c58a9ba3db4ed0c5` |
| `moxcms` | `0.8.1` | `BSD-3-Clause OR Apache-2.0` | `cargo-registry/moxcms-0.8.1/LICENSE.md` | `tauri/moxcms-0.8.1/LICENSE.md` | `2aa92cada6431e75615e3fe6cb1a9082c98f777d48ae1c087c0da0e37f7b8bff` |
| `muda` | `0.20.0` | `Apache-2.0 OR MIT` | `cargo-registry/muda-0.20.0/LICENSE-APACHE` | `tauri/muda-0.20.0/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `muda` | `0.20.0` | `Apache-2.0 OR MIT` | `cargo-registry/muda-0.20.0/LICENSE-MIT` | `tauri/muda-0.20.0/LICENSE-MIT` | `2ab5537b8c0cb1d475e2145b6a7994b04e7c71e6e5bd836f7f9c47c221a5ad9a` |
| `muda` | `0.20.0` | `Apache-2.0 OR MIT` | `cargo-registry/muda-0.20.0/LICENSE.spdx` | `tauri/muda-0.20.0/LICENSE.spdx` | `b99f9d43803f2ac979c50949df630bd8eeaf2fcd1b9fdb9093710d70f5dfb63d` |
| `num-conv` | `0.2.2` | `MIT OR Apache-2.0` | `cargo-registry/num-conv-0.2.2/LICENSE-Apache` | `tauri/num-conv-0.2.2/LICENSE-Apache` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `num-conv` | `0.2.2` | `MIT OR Apache-2.0` | `cargo-registry/num-conv-0.2.2/LICENSE-MIT` | `tauri/num-conv-0.2.2/LICENSE-MIT` | `e2e245f2b566d0bfafb0f6a04c16b82d710c4e6e7d799e39bb4ef6e541b85b98` |
| `num-traits` | `0.2.19` | `MIT OR Apache-2.0` | `cargo-registry/num-traits-0.2.19/LICENSE-APACHE` | `tauri/num-traits-0.2.19/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `num-traits` | `0.2.19` | `MIT OR Apache-2.0` | `cargo-registry/num-traits-0.2.19/LICENSE-MIT` | `tauri/num-traits-0.2.19/LICENSE-MIT` | `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| `once_cell` | `1.21.4` | `MIT OR Apache-2.0` | `cargo-registry/once_cell-1.21.4/LICENSE-APACHE` | `tauri/once_cell-1.21.4/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `once_cell` | `1.21.4` | `MIT OR Apache-2.0` | `cargo-registry/once_cell-1.21.4/LICENSE-MIT` | `tauri/once_cell-1.21.4/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `option-ext` | `0.2.0` | `MPL-2.0` | `cargo-registry/option-ext-0.2.0/LICENSE.txt` | `tauri/option-ext-0.2.0/LICENSE.txt` | `66a3107d5ad6a058aab753eaac2047ccb2ed0e39465dd0fe5844da3e300d5172` |
| `parking_lot` | `0.12.5` | `MIT OR Apache-2.0` | `cargo-registry/parking_lot-0.12.5/LICENSE-APACHE` | `tauri/parking_lot-0.12.5/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `parking_lot` | `0.12.5` | `MIT OR Apache-2.0` | `cargo-registry/parking_lot-0.12.5/LICENSE-MIT` | `tauri/parking_lot-0.12.5/LICENSE-MIT` | `c9a75f18b9ab2927829a208fc6aa2cf4e63b8420887ba29cdb265d6619ae82d5` |
| `parking_lot_core` | `0.9.12` | `MIT OR Apache-2.0` | `cargo-registry/parking_lot_core-0.9.12/LICENSE-APACHE` | `tauri/parking_lot_core-0.9.12/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `parking_lot_core` | `0.9.12` | `MIT OR Apache-2.0` | `cargo-registry/parking_lot_core-0.9.12/LICENSE-MIT` | `tauri/parking_lot_core-0.9.12/LICENSE-MIT` | `c9a75f18b9ab2927829a208fc6aa2cf4e63b8420887ba29cdb265d6619ae82d5` |
| `percent-encoding` | `2.3.2` | `MIT OR Apache-2.0` | `cargo-registry/percent-encoding-2.3.2/LICENSE-APACHE` | `tauri/percent-encoding-2.3.2/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `percent-encoding` | `2.3.2` | `MIT OR Apache-2.0` | `cargo-registry/percent-encoding-2.3.2/LICENSE-MIT` | `tauri/percent-encoding-2.3.2/LICENSE-MIT` | `b38f11f6096706e6de553dabe2a7ed142d59b6fa8c97e290c67496154745cdd5` |
| `phf` | `0.13.1` | `MIT` | `cargo-registry/phf-0.13.1/LICENSE` | `tauri/phf-0.13.1/LICENSE` | `0ab4d106b6faac07fb6a051815fd1b4d862d730895e2d7d7358c2f13565e7a38` |
| `phf_shared` | `0.13.1` | `MIT` | `cargo-registry/phf_shared-0.13.1/LICENSE` | `tauri/phf_shared-0.13.1/LICENSE` | `0ab4d106b6faac07fb6a051815fd1b4d862d730895e2d7d7358c2f13565e7a38` |
| `pin-project-lite` | `0.2.17` | `Apache-2.0 OR MIT` | `cargo-registry/pin-project-lite-0.2.17/LICENSE-APACHE` | `tauri/pin-project-lite-0.2.17/LICENSE-APACHE` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `pin-project-lite` | `0.2.17` | `Apache-2.0 OR MIT` | `cargo-registry/pin-project-lite-0.2.17/LICENSE-MIT` | `tauri/pin-project-lite-0.2.17/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `plist` | `1.10.3` | `MIT` | `cargo-registry/plist-1.10.3/LICENCE` | `tauri/plist-1.10.3/LICENCE` | `5b0ae40d1a35f7ae6591a28e44771240e6a88cb03a66c9189a45b9681639b466` |
| `png` | `0.18.1` | `MIT OR Apache-2.0` | `cargo-registry/png-0.18.1/LICENSE-APACHE` | `tauri/png-0.18.1/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `png` | `0.18.1` | `MIT OR Apache-2.0` | `cargo-registry/png-0.18.1/LICENSE-MIT` | `tauri/png-0.18.1/LICENSE-MIT` | `eaf40297c75da471f7cda1f3458e8d91b4b2ec866e609527a13acfa93b638652` |
| `potential_utf` | `0.1.6` | `Unicode-3.0` | `cargo-registry/potential_utf-0.1.6/LICENSE` | `tauri/potential_utf-0.1.6/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `powerfmt` | `0.2.1` | `MIT OR Apache-2.0` | `cargo-registry/powerfmt-0.2.1/LICENSE-Apache` | `tauri/powerfmt-0.2.1/LICENSE-Apache` | `155420c6403d4e0fca34105e3c03fdd6939b64c393c7ec6f95f5b72c5474eab0` |
| `powerfmt` | `0.2.1` | `MIT OR Apache-2.0` | `cargo-registry/powerfmt-0.2.1/LICENSE-MIT` | `tauri/powerfmt-0.2.1/LICENSE-MIT` | `070dbc7dda03a29296f2d58bdb9b7331af90f2abc9f31df22875d1eabaf29852` |
| `pxfm` | `0.1.30` | `BSD-3-Clause OR Apache-2.0` | `cargo-registry/pxfm-0.1.30/LICENSE-APACHE.md` | `tauri/pxfm-0.1.30/LICENSE-APACHE.md` | `90bf2d659c43045111b65c733ab2a6d4cbcb422a098368c8c58a9ba3db4ed0c5` |
| `pxfm` | `0.1.30` | `BSD-3-Clause OR Apache-2.0` | `cargo-registry/pxfm-0.1.30/LICENSE.md` | `tauri/pxfm-0.1.30/LICENSE.md` | `2aa92cada6431e75615e3fe6cb1a9082c98f777d48ae1c087c0da0e37f7b8bff` |
| `quick-xml` | `0.42.0` | `MIT` | `cargo-registry/quick-xml-0.42.0/LICENSE-MIT.md` | `tauri/quick-xml-0.42.0/LICENSE-MIT.md` | `f0cf9b1c62bbe3bd3a69f5f79c7158f513f612b4940a0a812d1db39d605318bc` |
| `raw-window-handle` | `0.6.2` | `MIT OR Apache-2.0 OR Zlib` | `cargo-registry/raw-window-handle-0.6.2/LICENSE-APACHE.md` | `tauri/raw-window-handle-0.6.2/LICENSE-APACHE.md` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `raw-window-handle` | `0.6.2` | `MIT OR Apache-2.0 OR Zlib` | `cargo-registry/raw-window-handle-0.6.2/LICENSE-MIT.md` | `tauri/raw-window-handle-0.6.2/LICENSE-MIT.md` | `9c5a80639a57c1c945570e7ebbca0706305849ce3c098021325cca9db2f7acc4` |
| `raw-window-handle` | `0.6.2` | `MIT OR Apache-2.0 OR Zlib` | `cargo-registry/raw-window-handle-0.6.2/LICENSE-ZLIB.md` | `tauri/raw-window-handle-0.6.2/LICENSE-ZLIB.md` | `ae895c8576e682f310fb70d93d9f99b1dd82872094c54a8f6cdfe2ca5c2d6ecb` |
| `regex` | `1.13.1` | `MIT OR Apache-2.0` | `cargo-registry/regex-1.13.1/LICENSE-APACHE` | `tauri/regex-1.13.1/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `regex` | `1.13.1` | `MIT OR Apache-2.0` | `cargo-registry/regex-1.13.1/LICENSE-MIT` | `tauri/regex-1.13.1/LICENSE-MIT` | `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| `regex-automata` | `0.4.18` | `MIT OR Apache-2.0` | `cargo-registry/regex-automata-0.4.18/LICENSE-APACHE` | `tauri/regex-automata-0.4.18/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `regex-automata` | `0.4.18` | `MIT OR Apache-2.0` | `cargo-registry/regex-automata-0.4.18/LICENSE-MIT` | `tauri/regex-automata-0.4.18/LICENSE-MIT` | `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| `regex-syntax` | `0.8.11` | `MIT OR Apache-2.0` | `cargo-registry/regex-syntax-0.8.11/LICENSE-APACHE` | `tauri/regex-syntax-0.8.11/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `regex-syntax` | `0.8.11` | `MIT OR Apache-2.0` | `cargo-registry/regex-syntax-0.8.11/LICENSE-MIT` | `tauri/regex-syntax-0.8.11/LICENSE-MIT` | `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| `same-file` | `1.0.6` | `Unlicense/MIT` | `cargo-registry/same-file-1.0.6/COPYING` | `tauri/same-file-1.0.6/COPYING` | `01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f` |
| `same-file` | `1.0.6` | `Unlicense/MIT` | `cargo-registry/same-file-1.0.6/LICENSE-MIT` | `tauri/same-file-1.0.6/LICENSE-MIT` | `cb3c929a05e6cbc9de9ab06a4c57eeb60ca8c724bef6c138c87d3a577e27aa14` |
| `scopeguard` | `1.2.0` | `MIT OR Apache-2.0` | `cargo-registry/scopeguard-1.2.0/LICENSE-APACHE` | `tauri/scopeguard-1.2.0/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `scopeguard` | `1.2.0` | `MIT OR Apache-2.0` | `cargo-registry/scopeguard-1.2.0/LICENSE-MIT` | `tauri/scopeguard-1.2.0/LICENSE-MIT` | `fb77f0a9c53e473abe5103c8632ef9f0f2874d4fb3f17cb2d8c661aab9cee9d7` |
| `semver` | `1.0.28` | `MIT OR Apache-2.0` | `cargo-registry/semver-1.0.28/LICENSE-APACHE` | `tauri/semver-1.0.28/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `semver` | `1.0.28` | `MIT OR Apache-2.0` | `cargo-registry/semver-1.0.28/LICENSE-MIT` | `tauri/semver-1.0.28/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `serde` | `1.0.229` | `MIT OR Apache-2.0` | `cargo-registry/serde-1.0.229/LICENSE-APACHE` | `tauri/serde-1.0.229/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `serde` | `1.0.229` | `MIT OR Apache-2.0` | `cargo-registry/serde-1.0.229/LICENSE-MIT` | `tauri/serde-1.0.229/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `serde-untagged` | `0.1.9` | `MIT OR Apache-2.0` | `cargo-registry/serde-untagged-0.1.9/LICENSE-APACHE` | `tauri/serde-untagged-0.1.9/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `serde-untagged` | `0.1.9` | `MIT OR Apache-2.0` | `cargo-registry/serde-untagged-0.1.9/LICENSE-MIT` | `tauri/serde-untagged-0.1.9/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `serde_core` | `1.0.229` | `MIT OR Apache-2.0` | `cargo-registry/serde_core-1.0.229/LICENSE-APACHE` | `tauri/serde_core-1.0.229/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `serde_core` | `1.0.229` | `MIT OR Apache-2.0` | `cargo-registry/serde_core-1.0.229/LICENSE-MIT` | `tauri/serde_core-1.0.229/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `serde_json` | `1.0.154` | `MIT OR Apache-2.0` | `cargo-registry/serde_json-1.0.154/LICENSE-APACHE` | `tauri/serde_json-1.0.154/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `serde_json` | `1.0.154` | `MIT OR Apache-2.0` | `cargo-registry/serde_json-1.0.154/LICENSE-MIT` | `tauri/serde_json-1.0.154/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `serde_spanned` | `1.1.2` | `MIT OR Apache-2.0` | `cargo-registry/serde_spanned-1.1.2/LICENSE-APACHE` | `tauri/serde_spanned-1.1.2/LICENSE-APACHE` | `249fdb6694467dd615b259f7ddde07be3f8ae5a72cecebb0e24f6629af4215fe` |
| `serde_spanned` | `1.1.2` | `MIT OR Apache-2.0` | `cargo-registry/serde_spanned-1.1.2/LICENSE-MIT` | `tauri/serde_spanned-1.1.2/LICENSE-MIT` | `6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6` |
| `serde_with` | `3.24.0` | `MIT OR Apache-2.0` | `cargo-registry/serde_with-3.24.0/LICENSE-APACHE` | `tauri/serde_with-3.24.0/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `serde_with` | `3.24.0` | `MIT OR Apache-2.0` | `cargo-registry/serde_with-3.24.0/LICENSE-MIT` | `tauri/serde_with-3.24.0/LICENSE-MIT` | `7576269ea71f767b99297934c0b2367532690f8c4badc695edf8e04ab6a1e545` |
| `serialize-to-javascript` | `0.1.2` | `MIT OR Apache-2.0` | `cargo-registry/serialize-to-javascript-0.1.2/LICENSE-APACHE` | `tauri/serialize-to-javascript-0.1.2/LICENSE-APACHE` | `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4` |
| `serialize-to-javascript` | `0.1.2` | `MIT OR Apache-2.0` | `cargo-registry/serialize-to-javascript-0.1.2/LICENSE-MIT` | `tauri/serialize-to-javascript-0.1.2/LICENSE-MIT` | `e53a7165e8bc3ced5e5628404b3d2d28ecc14ad720f1c09ba28385d5151ecf93` |
| `simd-adler32` | `0.3.10` | `MIT` | `cargo-registry/simd-adler32-0.3.10/LICENSE.md` | `tauri/simd-adler32-0.3.10/LICENSE.md` | `42a35170233e83e18856792e748de4c1ce4a63b2afce9a370c89ef3fe23f9f2d` |
| `siphasher` | `1.0.4` | `MIT OR Apache-2.0` | `cargo-registry/siphasher-1.0.4/COPYING` | `tauri/siphasher-1.0.4/COPYING` | `c962ee4d1d05ddc138b202b2540219ebc57893fcf97b364852094a9a94ce1365` |
| `siphasher` | `1.0.4` | `MIT OR Apache-2.0` | `cargo-registry/siphasher-1.0.4/LICENSE-APACHE` | `tauri/siphasher-1.0.4/LICENSE-APACHE` | `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30` |
| `siphasher` | `1.0.4` | `MIT OR Apache-2.0` | `cargo-registry/siphasher-1.0.4/LICENSE-MIT` | `tauri/siphasher-1.0.4/LICENSE-MIT` | `f9c4c77baa3828004ee54b8a4f2db2e88ed44a6237a493965bf551fac0fcb62d` |
| `smallvec` | `1.16.4` | `MIT OR Apache-2.0` | `cargo-registry/smallvec-1.16.4/LICENSE-APACHE` | `tauri/smallvec-1.16.4/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `smallvec` | `1.16.4` | `MIT OR Apache-2.0` | `cargo-registry/smallvec-1.16.4/LICENSE-MIT` | `tauri/smallvec-1.16.4/LICENSE-MIT` | `0b28172679e0009b655da42797c03fd163a3379d5cfa67ba1f1655e974a2a1a9` |
| `socket2` | `0.6.5` | `MIT OR Apache-2.0` | `cargo-registry/socket2-0.6.5/LICENSE-APACHE` | `tauri/socket2-0.6.5/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `socket2` | `0.6.5` | `MIT OR Apache-2.0` | `cargo-registry/socket2-0.6.5/LICENSE-MIT` | `tauri/socket2-0.6.5/LICENSE-MIT` | `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| `softbuffer` | `0.4.8` | `MIT OR Apache-2.0` | `cargo-registry/softbuffer-0.4.8/LICENSE-APACHE` | `tauri/softbuffer-0.4.8/LICENSE-APACHE` | `a4445427440043e8ce6fadf7a43fb906cb0d7f6c573a560a9f142cf9f9bf8ff4` |
| `softbuffer` | `0.4.8` | `MIT OR Apache-2.0` | `cargo-registry/softbuffer-0.4.8/LICENSE-MIT` | `tauri/softbuffer-0.4.8/LICENSE-MIT` | `0463bfa47d6d763d8c824333efb0e6d6df3946c13c3b077c78ed9f23ca50d0f2` |
| `stable_deref_trait` | `1.2.1` | `MIT OR Apache-2.0` | `cargo-registry/stable_deref_trait-1.2.1/LICENSE-APACHE` | `tauri/stable_deref_trait-1.2.1/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `stable_deref_trait` | `1.2.1` | `MIT OR Apache-2.0` | `cargo-registry/stable_deref_trait-1.2.1/LICENSE-MIT` | `tauri/stable_deref_trait-1.2.1/LICENSE-MIT` | `5e05b024f653a5ce199e77cbbbd42fb5553562ec714b819421ed0c3e552a75d7` |
| `tao` | `0.37.2` | `Apache-2.0` | `cargo-registry/tao-0.37.2/LICENSE` | `tauri/tao-0.37.2/LICENSE` | `6dc0e068dcf3a5bc8e054205b85b7720e1d49265bbc64bf515d2cf79197df69a` |
| `tao` | `0.37.2` | `Apache-2.0` | `cargo-registry/tao-0.37.2/LICENSE.spdx` | `tauri/tao-0.37.2/LICENSE.spdx` | `02b6448337b757772d90e5f31caaa0f9e6afaad2221c01771d0a3deaa4d4196c` |
| `tauri` | `2.12.5` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-2.12.5/LICENSE-APACHE-2.0` | `tauri/tauri-2.12.5/LICENSE-APACHE-2.0` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `tauri` | `2.12.5` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-2.12.5/LICENSE-MIT` | `tauri/tauri-2.12.5/LICENSE-MIT` | `9dd42ea92cff2ede5cd477cbfcce051b2d0115c0ac7f368ee88cb545055dff1d` |
| `@tauri-apps/api` | `2.12.4` | `Apache-2.0 OR MIT` | `node_modules/@tauri-apps/api/LICENSE-APACHE-2.0` | `tauri/tauri-js-api-2.12.4/LICENSE-APACHE-2.0` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `@tauri-apps/api` | `2.12.4` | `Apache-2.0 OR MIT` | `node_modules/@tauri-apps/api/LICENSE-MIT` | `tauri/tauri-js-api-2.12.4/LICENSE-MIT` | `9dd42ea92cff2ede5cd477cbfcce051b2d0115c0ac7f368ee88cb545055dff1d` |
| `@tauri-apps/api` | `2.12.4` | `Apache-2.0 OR MIT` | `node_modules/@tauri-apps/api/LICENSE.spdx` | `tauri/tauri-js-api-2.12.4/LICENSE.spdx` | `5e834fe9788c524210deda859c46a550ffa5aa0cba9ea0572f0d81120e97c008` |
| `tauri-plugin-single-instance` | `2.5.4` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-plugin-single-instance-2.5.4/LICENSE.spdx` | `tauri/tauri-plugin-single-instance-2.5.4/LICENSE.spdx` | `eb8a6c84630461b352badcab1dbe5d0168c56d377358b2b8c86b51003272d5ef` |
| `tauri-plugin-single-instance` | `2.5.4` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-plugin-single-instance-2.5.4/LICENSE_APACHE-2.0` | `tauri/tauri-plugin-single-instance-2.5.4/LICENSE_APACHE-2.0` | `0cec06e0e55fbc3dc5cee4fca9b607f66cb8f4e4dbcf3b3c013594dd156732e9` |
| `tauri-plugin-single-instance` | `2.5.4` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-plugin-single-instance-2.5.4/LICENSE_MIT` | `tauri/tauri-plugin-single-instance-2.5.4/LICENSE_MIT` | `b7f455413bfe5f9d7da501a4468d4c5f866a432c325947e98cc9535e8b26cd6d` |
| `tauri-runtime` | `2.12.2` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-runtime-2.12.2/LICENSE-APACHE-2.0` | `tauri/tauri-runtime-2.12.2/LICENSE-APACHE-2.0` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `tauri-runtime` | `2.12.2` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-runtime-2.12.2/LICENSE-MIT` | `tauri/tauri-runtime-2.12.2/LICENSE-MIT` | `9dd42ea92cff2ede5cd477cbfcce051b2d0115c0ac7f368ee88cb545055dff1d` |
| `tauri-runtime-wry` | `2.12.3` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-runtime-wry-2.12.3/LICENSE-APACHE-2.0` | `tauri/tauri-runtime-wry-2.12.3/LICENSE-APACHE-2.0` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `tauri-runtime-wry` | `2.12.3` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-runtime-wry-2.12.3/LICENSE-MIT` | `tauri/tauri-runtime-wry-2.12.3/LICENSE-MIT` | `9dd42ea92cff2ede5cd477cbfcce051b2d0115c0ac7f368ee88cb545055dff1d` |
| `tauri-utils` | `2.10.2` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-utils-2.10.2/LICENSE-APACHE-2.0` | `tauri/tauri-utils-2.10.2/LICENSE-APACHE-2.0` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `tauri-utils` | `2.10.2` | `Apache-2.0 OR MIT` | `cargo-registry/tauri-utils-2.10.2/LICENSE-MIT` | `tauri/tauri-utils-2.10.2/LICENSE-MIT` | `9dd42ea92cff2ede5cd477cbfcce051b2d0115c0ac7f368ee88cb545055dff1d` |
| `thiserror` | `2.0.21` | `MIT OR Apache-2.0` | `cargo-registry/thiserror-2.0.21/LICENSE-APACHE` | `tauri/thiserror-2.0.21/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `thiserror` | `2.0.21` | `MIT OR Apache-2.0` | `cargo-registry/thiserror-2.0.21/LICENSE-MIT` | `tauri/thiserror-2.0.21/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `time` | `0.3.55` | `MIT OR Apache-2.0` | `cargo-registry/time-0.3.55/LICENSE-Apache` | `tauri/time-0.3.55/LICENSE-Apache` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `time` | `0.3.55` | `MIT OR Apache-2.0` | `cargo-registry/time-0.3.55/LICENSE-MIT` | `tauri/time-0.3.55/LICENSE-MIT` | `2537228d9a1b44a5dc595241349cae7090b326c8de165aaf89bfddef4a00d0fc` |
| `time-core` | `0.1.9` | `MIT OR Apache-2.0` | `cargo-registry/time-core-0.1.9/LICENSE-Apache` | `tauri/time-core-0.1.9/LICENSE-Apache` | `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594` |
| `time-core` | `0.1.9` | `MIT OR Apache-2.0` | `cargo-registry/time-core-0.1.9/LICENSE-MIT` | `tauri/time-core-0.1.9/LICENSE-MIT` | `2537228d9a1b44a5dc595241349cae7090b326c8de165aaf89bfddef4a00d0fc` |
| `tinystr` | `0.8.5` | `Unicode-3.0` | `cargo-registry/tinystr-0.8.5/LICENSE` | `tauri/tinystr-0.8.5/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `tokio` | `1.54.1` | `MIT` | `cargo-registry/tokio-1.54.1/LICENSE` | `tauri/tokio-1.54.1/LICENSE` | `253cd04c6714889df2d32f3f64d669179a1c95c76ac43c40882c52eb06bc3552` |
| `toml` | `1.1.8+spec-1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/toml-1.1.8+spec-1.1.0/LICENSE-APACHE` | `tauri/toml-1.1.8+spec-1.1.0/LICENSE-APACHE` | `249fdb6694467dd615b259f7ddde07be3f8ae5a72cecebb0e24f6629af4215fe` |
| `toml` | `1.1.8+spec-1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/toml-1.1.8+spec-1.1.0/LICENSE-MIT` | `tauri/toml-1.1.8+spec-1.1.0/LICENSE-MIT` | `6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6` |
| `toml_datetime` | `1.1.2+spec-1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/toml_datetime-1.1.2+spec-1.1.0/LICENSE-APACHE` | `tauri/toml_datetime-1.1.2+spec-1.1.0/LICENSE-APACHE` | `249fdb6694467dd615b259f7ddde07be3f8ae5a72cecebb0e24f6629af4215fe` |
| `toml_datetime` | `1.1.2+spec-1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/toml_datetime-1.1.2+spec-1.1.0/LICENSE-MIT` | `tauri/toml_datetime-1.1.2+spec-1.1.0/LICENSE-MIT` | `6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6` |
| `toml_parser` | `1.1.5+spec-1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/toml_parser-1.1.5+spec-1.1.0/LICENSE-APACHE` | `tauri/toml_parser-1.1.5+spec-1.1.0/LICENSE-APACHE` | `249fdb6694467dd615b259f7ddde07be3f8ae5a72cecebb0e24f6629af4215fe` |
| `toml_parser` | `1.1.5+spec-1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/toml_parser-1.1.5+spec-1.1.0/LICENSE-MIT` | `tauri/toml_parser-1.1.5+spec-1.1.0/LICENSE-MIT` | `6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6` |
| `toml_writer` | `1.1.3+spec-1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/toml_writer-1.1.3+spec-1.1.0/LICENSE-APACHE` | `tauri/toml_writer-1.1.3+spec-1.1.0/LICENSE-APACHE` | `249fdb6694467dd615b259f7ddde07be3f8ae5a72cecebb0e24f6629af4215fe` |
| `toml_writer` | `1.1.3+spec-1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/toml_writer-1.1.3+spec-1.1.0/LICENSE-MIT` | `tauri/toml_writer-1.1.3+spec-1.1.0/LICENSE-MIT` | `6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6` |
| `tracing` | `0.1.44` | `MIT` | `cargo-registry/tracing-0.1.44/LICENSE` | `tauri/tracing-0.1.44/LICENSE` | `898b1ae9821e98daf8964c8d6c7f61641f5f5aa78ad500020771c0939ee0dea1` |
| `tracing-core` | `0.1.36` | `MIT` | `cargo-registry/tracing-core-0.1.36/LICENSE` | `tauri/tracing-core-0.1.36/LICENSE` | `898b1ae9821e98daf8964c8d6c7f61641f5f5aa78ad500020771c0939ee0dea1` |
| `tray-icon` | `0.25.1` | `MIT OR Apache-2.0` | `cargo-registry/tray-icon-0.25.1/LICENSE-APACHE` | `tauri/tray-icon-0.25.1/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `tray-icon` | `0.25.1` | `MIT OR Apache-2.0` | `cargo-registry/tray-icon-0.25.1/LICENSE-MIT` | `tauri/tray-icon-0.25.1/LICENSE-MIT` | `2ab5537b8c0cb1d475e2145b6a7994b04e7c71e6e5bd836f7f9c47c221a5ad9a` |
| `tray-icon` | `0.25.1` | `MIT OR Apache-2.0` | `cargo-registry/tray-icon-0.25.1/LICENSE.spdx` | `tauri/tray-icon-0.25.1/LICENSE.spdx` | `6c1e5d0ccf5e8951401cb964036ae664d52fc8d1148173b53bc54fc534f5d404` |
| `typeid` | `1.0.3` | `MIT OR Apache-2.0` | `cargo-registry/typeid-1.0.3/LICENSE-APACHE` | `tauri/typeid-1.0.3/LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| `typeid` | `1.0.3` | `MIT OR Apache-2.0` | `cargo-registry/typeid-1.0.3/LICENSE-MIT` | `tauri/typeid-1.0.3/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| `unicode-segmentation` | `1.13.3` | `MIT OR Apache-2.0` | `cargo-registry/unicode-segmentation-1.13.3/COPYRIGHT` | `tauri/unicode-segmentation-1.13.3/COPYRIGHT` | `23860c2a7b5d96b21569afedf033469bab9fe14a1b24a35068b8641c578ce24d` |
| `unicode-segmentation` | `1.13.3` | `MIT OR Apache-2.0` | `cargo-registry/unicode-segmentation-1.13.3/LICENSE-APACHE` | `tauri/unicode-segmentation-1.13.3/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `unicode-segmentation` | `1.13.3` | `MIT OR Apache-2.0` | `cargo-registry/unicode-segmentation-1.13.3/LICENSE-MIT` | `tauri/unicode-segmentation-1.13.3/LICENSE-MIT` | `7b63ecd5f1902af1b63729947373683c32745c16a10e8e6292e2e2dcd7e90ae0` |
| `url` | `2.5.8` | `MIT OR Apache-2.0` | `cargo-registry/url-2.5.8/LICENSE-APACHE` | `tauri/url-2.5.8/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `url` | `2.5.8` | `MIT OR Apache-2.0` | `cargo-registry/url-2.5.8/LICENSE-MIT` | `tauri/url-2.5.8/LICENSE-MIT` | `b38f11f6096706e6de553dabe2a7ed142d59b6fa8c97e290c67496154745cdd5` |
| `urlpattern` | `0.6.0` | `MIT` | `cargo-registry/urlpattern-0.6.0/LICENSE` | `tauri/urlpattern-0.6.0/LICENSE` | `5cf1c9dc617e4cad7fa03181ff893601ad5243aa3be527ef39034e1cd6377676` |
| `utf8_iter` | `1.0.4` | `Apache-2.0 OR MIT` | `cargo-registry/utf8_iter-1.0.4/COPYRIGHT` | `tauri/utf8_iter-1.0.4/COPYRIGHT` | `c30152c94a6d75e021adbc52b3a52470366a46edb917e17deae3259251af244c` |
| `utf8_iter` | `1.0.4` | `Apache-2.0 OR MIT` | `cargo-registry/utf8_iter-1.0.4/LICENSE-APACHE` | `tauri/utf8_iter-1.0.4/LICENSE-APACHE` | `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30` |
| `utf8_iter` | `1.0.4` | `Apache-2.0 OR MIT` | `cargo-registry/utf8_iter-1.0.4/LICENSE-MIT` | `tauri/utf8_iter-1.0.4/LICENSE-MIT` | `3fa4ca83dcc9237839b1bdeb2e6d16bdfb5ec0c5ce42b24694d8bbf0dcbef72c` |
| `uuid` | `1.29.0` | `Apache-2.0 OR MIT` | `cargo-registry/uuid-1.29.0/LICENSE-APACHE` | `tauri/uuid-1.29.0/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `uuid` | `1.29.0` | `Apache-2.0 OR MIT` | `cargo-registry/uuid-1.29.0/LICENSE-MIT` | `tauri/uuid-1.29.0/LICENSE-MIT` | `436bc5a105d8e57dcd8778730f3754f7bf39c14d2f530e4cde4bd2d17a83ec3d` |
| `walkdir` | `2.5.0` | `Unlicense/MIT` | `cargo-registry/walkdir-2.5.0/COPYING` | `tauri/walkdir-2.5.0/COPYING` | `01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f` |
| `walkdir` | `2.5.0` | `Unlicense/MIT` | `cargo-registry/walkdir-2.5.0/LICENSE-MIT` | `tauri/walkdir-2.5.0/LICENSE-MIT` | `0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f` |
| `web-time` | `1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/web-time-1.1.0/LICENSE-APACHE` | `tauri/web-time-1.1.0/LICENSE-APACHE` | `54a744942eb3fa63a74d8d359989ed6962c04f3f10ff20cfa08ff8aa7efbe4e6` |
| `web-time` | `1.1.0` | `MIT OR Apache-2.0` | `cargo-registry/web-time-1.1.0/LICENSE-MIT` | `tauri/web-time-1.1.0/LICENSE-MIT` | `f428305bbf2e70fbd807af0f3c7ac1b7acfb95311640f451e9d2e48db6d7e96b` |
| `webview2-com` | `0.39.1` | `MIT` | `https://raw.githubusercontent.com/wravery/webview2-rs/edc2caf886175ccaebe86078c9cfe1ae2a187328/LICENSE` | `tauri/webview2-com-0.39.1/LICENSE` | `0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73` |
| `webview2-com-sys` | `0.39.1` | `MIT` | `https://raw.githubusercontent.com/wravery/webview2-rs/edc2caf886175ccaebe86078c9cfe1ae2a187328/LICENSE` | `tauri/webview2-com-sys-0.39.1/LICENSE` | `0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73` |
| `winapi-util` | `0.1.11` | `Unlicense OR MIT` | `cargo-registry/winapi-util-0.1.11/COPYING` | `tauri/winapi-util-0.1.11/COPYING` | `01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f` |
| `winapi-util` | `0.1.11` | `Unlicense OR MIT` | `cargo-registry/winapi-util-0.1.11/LICENSE-MIT` | `tauri/winapi-util-0.1.11/LICENSE-MIT` | `cb3c929a05e6cbc9de9ab06a4c57eeb60ca8c724bef6c138c87d3a577e27aa14` |
| `window-vibrancy` | `0.8.1` | `Apache-2.0 OR MIT` | `cargo-registry/window-vibrancy-0.8.1/LICENSE-APACHE` | `tauri/window-vibrancy-0.8.1/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `window-vibrancy` | `0.8.1` | `Apache-2.0 OR MIT` | `cargo-registry/window-vibrancy-0.8.1/LICENSE-MIT` | `tauri/window-vibrancy-0.8.1/LICENSE-MIT` | `617b85f62a4f1b6a62d3d9f28f6d7003b7f68a70b7675a8174bbcc68401ac62d` |
| `window-vibrancy` | `0.8.1` | `Apache-2.0 OR MIT` | `cargo-registry/window-vibrancy-0.8.1/LICENSE.spdx` | `tauri/window-vibrancy-0.8.1/LICENSE.spdx` | `7ccef0d35e8507fa02d7e904a5277d62723f9ec576d52f6a865cab7b4dbf12ad` |
| `windows` | `0.62.2` | `MIT OR Apache-2.0` | `cargo-registry/windows-0.62.2/license-apache-2.0` | `tauri/windows-0.62.2/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows` | `0.62.2` | `MIT OR Apache-2.0` | `cargo-registry/windows-0.62.2/license-mit` | `tauri/windows-0.62.2/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `windows-collections` | `0.3.2` | `MIT OR Apache-2.0` | `cargo-registry/windows-collections-0.3.2/license-apache-2.0` | `tauri/windows-collections-0.3.2/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows-collections` | `0.3.2` | `MIT OR Apache-2.0` | `cargo-registry/windows-collections-0.3.2/license-mit` | `tauri/windows-collections-0.3.2/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `windows-core` | `0.62.2` | `MIT OR Apache-2.0` | `cargo-registry/windows-core-0.62.2/license-apache-2.0` | `tauri/windows-core-0.62.2/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows-core` | `0.62.2` | `MIT OR Apache-2.0` | `cargo-registry/windows-core-0.62.2/license-mit` | `tauri/windows-core-0.62.2/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `windows-future` | `0.3.2` | `MIT OR Apache-2.0` | `cargo-registry/windows-future-0.3.2/license-apache-2.0` | `tauri/windows-future-0.3.2/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows-future` | `0.3.2` | `MIT OR Apache-2.0` | `cargo-registry/windows-future-0.3.2/license-mit` | `tauri/windows-future-0.3.2/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `windows-link` | `0.2.1` | `MIT OR Apache-2.0` | `cargo-registry/windows-link-0.2.1/license-apache-2.0` | `tauri/windows-link-0.2.1/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows-link` | `0.2.1` | `MIT OR Apache-2.0` | `cargo-registry/windows-link-0.2.1/license-mit` | `tauri/windows-link-0.2.1/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `windows-numerics` | `0.3.1` | `MIT OR Apache-2.0` | `cargo-registry/windows-numerics-0.3.1/license-apache-2.0` | `tauri/windows-numerics-0.3.1/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows-numerics` | `0.3.1` | `MIT OR Apache-2.0` | `cargo-registry/windows-numerics-0.3.1/license-mit` | `tauri/windows-numerics-0.3.1/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `windows-result` | `0.4.1` | `MIT OR Apache-2.0` | `cargo-registry/windows-result-0.4.1/license-apache-2.0` | `tauri/windows-result-0.4.1/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows-result` | `0.4.1` | `MIT OR Apache-2.0` | `cargo-registry/windows-result-0.4.1/license-mit` | `tauri/windows-result-0.4.1/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `windows-strings` | `0.5.1` | `MIT OR Apache-2.0` | `cargo-registry/windows-strings-0.5.1/license-apache-2.0` | `tauri/windows-strings-0.5.1/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows-strings` | `0.5.1` | `MIT OR Apache-2.0` | `cargo-registry/windows-strings-0.5.1/license-mit` | `tauri/windows-strings-0.5.1/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `windows-sys` | `0.61.2` | `MIT OR Apache-2.0` | `cargo-registry/windows-sys-0.61.2/license-apache-2.0` | `tauri/windows-sys-0.61.2/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows-sys` | `0.61.2` | `MIT OR Apache-2.0` | `cargo-registry/windows-sys-0.61.2/license-mit` | `tauri/windows-sys-0.61.2/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `windows-threading` | `0.2.1` | `MIT OR Apache-2.0` | `cargo-registry/windows-threading-0.2.1/license-apache-2.0` | `tauri/windows-threading-0.2.1/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows-threading` | `0.2.1` | `MIT OR Apache-2.0` | `cargo-registry/windows-threading-0.2.1/license-mit` | `tauri/windows-threading-0.2.1/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `windows-version` | `0.1.7` | `MIT OR Apache-2.0` | `cargo-registry/windows-version-0.1.7/license-apache-2.0` | `tauri/windows-version-0.1.7/license-apache-2.0` | `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b` |
| `windows-version` | `0.1.7` | `MIT OR Apache-2.0` | `cargo-registry/windows-version-0.1.7/license-mit` | `tauri/windows-version-0.1.7/license-mit` | `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| `winnow` | `1.0.4` | `MIT` | `cargo-registry/winnow-1.0.4/LICENSE-MIT` | `tauri/winnow-1.0.4/LICENSE-MIT` | `cb5aedb296c5246d1f22e9099f925a65146f9f0d6b4eebba97fd27a6cdbbab2d` |
| `writeable` | `0.6.4` | `Unicode-3.0` | `cargo-registry/writeable-0.6.4/LICENSE` | `tauri/writeable-0.6.4/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `wry` | `0.57.1` | `Apache-2.0 OR MIT` | `cargo-registry/wry-0.57.1/LICENSE-APACHE` | `tauri/wry-0.57.1/LICENSE-APACHE` | `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2` |
| `wry` | `0.57.1` | `Apache-2.0 OR MIT` | `cargo-registry/wry-0.57.1/LICENSE-MIT` | `tauri/wry-0.57.1/LICENSE-MIT` | `ef88893533af22ffbb68ceef0724ba1389518df26165a34f3c99ad30b884df4f` |
| `wry` | `0.57.1` | `Apache-2.0 OR MIT` | `cargo-registry/wry-0.57.1/LICENSE.spdx` | `tauri/wry-0.57.1/LICENSE.spdx` | `26235c41e314fb0da35b33b2bb5bcbc4dd523839f8eeca52efcd43586f441fb5` |
| `yoke` | `0.8.4` | `Unicode-3.0` | `cargo-registry/yoke-0.8.4/LICENSE` | `tauri/yoke-0.8.4/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `zerofrom` | `0.1.8` | `Unicode-3.0` | `cargo-registry/zerofrom-0.1.8/LICENSE` | `tauri/zerofrom-0.1.8/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `zerotrie` | `0.2.5` | `Unicode-3.0` | `cargo-registry/zerotrie-0.2.5/LICENSE` | `tauri/zerotrie-0.2.5/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `zerovec` | `0.11.9` | `Unicode-3.0` | `cargo-registry/zerovec-0.11.9/LICENSE` | `tauri/zerovec-0.11.9/LICENSE` | `f367c1b8e1aa262435251e442901da4607b4650e0e63a026f5044473ecfb90f2` |
| `zmij` | `1.0.23` | `MIT` | `cargo-registry/zmij-1.0.23/LICENSE-MIT` | `tauri/zmij-1.0.23/LICENSE-MIT` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
