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
                hash!("sha512:9yQf6t-5eOk_99NBJ694sGtza4w1_C0FFrBwwbsRSNe1EIyzZzer-y1GOTbnzpp4qrIwWKyiop1CZiR7z7--aA"),
            ))
            .do_check()
            .do_install_check()
            .post_install_check(r#"stat "out/bin/hello""#)
            .build()
    }
}
