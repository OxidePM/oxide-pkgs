use crate::{build::fetchurl::FetchUrl, stdenv::Stdenv};
use oxide_core::prelude::*;

pub struct Hello {
    pub fetchurl: FetchUrl,
    pub stdenv: Stdenv,
}

impl IntoDrv for Hello {
    fn into_drv(self) -> Drv {
        let version = "2.12.1";
        self.stdenv
            .make_derivation()
            .name("hello")
            .version(version)
            .src(self.fetchurl.fetch(
                format!("mirror://gnu/hello/hello-{version}.tar.gz"),
                hash!("sha512:YzJSc1lXcHFabXhyWVhOa2FtWnNZWE5xWm14ellXcG1jMjNSc2FtWnNjMlJxYkdaaGFuTnNhMlpoYzJSclphcw"),
            ))
            .do_check()
            .do_install_check()
            .post_install_check(
                r#"
                stat "out/bin/hello"
                "#,
            )
            // the following are just for testing
            .pre_unpack(r#"echo "hello pre unpack""#)
            .post_unpack(r#"echo "hello post unpack""#)
            .pre_patch(r#"echo "hello pre patch""#)
            .post_patch(r#"echo "hello post patch""#)
            .pre_configure(r#"echo "hello pre configure""#)
            .post_configure(r#"echo "hello post configure""#)
            .pre_build(r#"echo "hello pre build""#)
            .post_build(r#"echo "hello post build""#)
            .pre_check(r#"echo "hello pre check""#)
            .post_check(r#"echo "hello post check""#)
            .pre_install(r#"echo "hello pre install""#)
            .post_install(r#"echo "hello post install""#)
            .pre_fix(r#"echo "hello pre fix""#)
            .post_fix(r#"echo "hello post fix""#)
            .pre_install_check(r#"echo "hello pre install check""#)
            .post_install_check(r#"echo "hello post install check""#)
            .build()
    }
}
