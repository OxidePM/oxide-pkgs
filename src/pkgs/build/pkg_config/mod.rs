use super::fetchurl::FetchUrl;
use crate::stdenv::Stdenv;
use oxide_core::{
    drv::{Drv, IntoDrv, LazyDrv},
    hash, local_file,
};

pub struct PkgConfig {
    pub stdenv: Stdenv,
    pub fetchurl: FetchUrl,
    pub libiconv: LazyDrv,
    pub vanilla: Option<bool>,
}

impl IntoDrv for PkgConfig {
    fn into_drv(self) -> Drv {
        let name = "pkg-config";
        let version = "0.29.2";
        let vanilla = self.vanilla.unwrap_or(false);
        self.stdenv
            .make_derivation()
            .name(name)
            .version(version)
            .src(self.fetchurl.fetch(
                format!("https://pkg-config.freedesktop.org/releases/{name}-{version}.tar.gz"),
                hash!("sha512:YzJSc1lXcHFabXhyWVhOa2FtWnNZWE5xWm14ellXcG1jMjNSc2FtWnNjMlJxYkdaaGFuTnNhMlpoYzJSclphcw"),
            ))
            .out("out")
            .out("man")
            .out("doc")
            .input_bool("STRICT_DEPS", true)
            .optional(!vanilla, |builder| builder.patch(local_file!("requires-private.patch")))
            .post_patch(format!(
                    "substituteInPlace ./config.guess ./glib/config.guess --replace-fail /usr/bin/uname uname\n{}", 
                    (!vanilla).then_some("rm -f check/check-requires-private check/check-gtk check/missing").unwrap_or_default(),
            ))
            .dep_build_host(self.libiconv)
            .configure_flags("--with-internal-glib")
            .input_bool("ENABLE_PARALLEL_BUILDING", true)
            .do_check()
            .post_install(r#"rm -f "$out"/bin/*-pkg-config"#)
            .build()
    }
}
