use crate::{build::fetchurl::FetchUrl, stdenv::Stdenv};
use oxide_core::prelude::*;

pub struct Hello {
    pub fetchurl: FetchUrl,
    pub stdenv: Stdenv,
}

impl IntoDrv for Hello {
    fn into_drv(self: Box<Self>) -> Drv {
        let version = "2.12.1";
        self.stdenv
            .builder()
            .name("hello")
            .version(version)
            .src(self.fetchurl.fetch(
                format!("mirror://gnu/hello/hello-{version}.tar.gz"),
                hash!("sha512:..."),
            ))
            .do_check()
            .do_install_check()
            .post_install_check(
                r#"
                stat "out/bin/hello"
                "#,
            )
            .build()
    }
}
