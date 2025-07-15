use crate::{build::fetchurl::FetchUrl, stdenv::Stdenv};
use oxide_core::{
    drv::{Drv, IntoDrv},
    expr,
    expr::Expr,
    hash, local_file,
};

pub struct LibiConv {
    pub stdenv: Stdenv,
    pub fetchurl: FetchUrl,
    pub update_autotools_gnu_config_scripts: Expr,
    pub r#static: Option<bool>,
    pub shared: Option<bool>,
}

impl IntoDrv for LibiConv {
    fn into_drv(self) -> Drv {
        let name = "libiconv";
        let version = "1.17";
        let r#static = self.r#static.unwrap_or(false);
        let shared = self.r#static.unwrap_or(true);
        self.stdenv
            .make_derivation()
            .name(name)
            .version(version)
            .src(self.fetchurl.fetch(
                    format!("mirror://gnu/libiconv/{name}-${version}.tar.gzmirror://gnu/libiconv/{name}-{version}.tar.gz"), 
                    hash!("sha512:YzJSc1lXcHFabXhyWVhOa2FtWnNZWE5xWm14ellXcG1jMjNSc2FtWnNjMlJxYkdaaGFuTnNhMlpoYzJSclphcw"),
            ))
            .input_bool("ENABLE_PARALLEL_BUILDING", true)
            .dep_build_host(self.update_autotools_gnu_config_scripts)
            .input("SETUP_HOOKS", expr![
                local_file!("../../../build/setup-hooks/role.bash"),
                local_file!("setup-hook.sh"),
            ])
            .post_patch(format!("{}", (!shared).then_some("sed -i -e '/preload/d' Makefile.in").unwrap_or_default()))
            .configure_flags(format!("{}-static {}-shared", if r#static { "enable" } else { "disable" }, if shared { "enable" } else { "disable" }))
            .build()
    }
}
