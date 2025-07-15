use crate::build::fetchurl::FetchUrl;
use crate::stdenv::Stdenv;
use oxide_core::{
    drv::{Drv, IntoDrv, LazyDrv},
    hash,
};

pub struct Curl {
    pub stdenv: Stdenv,
    pub fetchurl: FetchUrl,
    // TODO: pkg-config
    pub pkg_config: LazyDrv,
    pub perl: LazyDrv,
}

impl IntoDrv for Curl {
    fn into_drv(self) -> Drv {
        let version = "8.14.1";
        self.stdenv
            .make_derivation()
            .name("curl")
            .version(version)
            .src(self.fetchurl.fetch(
                format!("https://curl.haxx.se/download/curl-{version}.tar.xz"),
                hash!("sha512:YzJSc1lXcHFabXhyWVhOa2FtWnNZWE5xWm14ellXcG1jMjNSc2FtWnNjMlJxYkdaaGFuTnNhMlpoYzJSclphcw"),
            ))
            // TODO: use indoc
            .post_patch(
r"substituteInPlace ./config.guess --replace-fail /usr/bin/uname uname
patchShebangs scripts")
            .out("bin")
            .out("dev")
            .out("out")
            .out("man")
            .out("devdoc")
            .input("ENABLE_PARALLEL_BUILDING", "1")
            .input("STRICT_DEPS", "1")
            .dep_build_host(self.pkg_config)
            .dep_build_host(self.perl)
            // TODO: add all optionals dep_build_host
            .pre_configure(
r"sed -e 's|/usr/bin|/no-such-path|g' -i.bak configure
rm src/tool_hugehelp.c")
            .configure_flags([
                "--enable-versioned-symbols",
                "--disable-manual"
            ].join(" "))
            .input("CXX", format!("{}c++", "TODO"))
            .input("CXXCPP", format!("{}c++ -E", "TODO"))
            .do_check()
            .pre_check("patchShebangs tests/")
            .post_install(
r#"moveToOutput bin/curl-config "$dev"
# Install completions
make -C scripts install"#)
            .build()
    }
}
