use oxide_core::{
    drv::{Drv, IntoDrv},
    hash,
};

use crate::{build::fetchurl::FetchUrl, stdenv::Stdenv};

pub struct Zlib {
    pub stdenv: Stdenv,
    pub fetchurl: FetchUrl,
    pub shared: Option<bool>,
    pub r#static: Option<bool>,
    pub r#split_static_out: Option<bool>,
}

impl IntoDrv for Zlib {
    fn into_drv(self) -> Drv {
        let version = "1.3.1";
        let shared = self.shared.unwrap_or(true);
        let r#static = self.r#static.unwrap_or(true);
        let split_static_out = self.split_static_out.unwrap_or(shared && r#static);
        self.stdenv
            .make_derivation()
            .name("zlib")
            .version("1.3.1")
            .src(self.fetchurl.fetch(
                format!("https://www.zlib.net/fossils/zlib-{version}.tar.gz"),
                hash!("sha512:vUFhtqfzz1sc1ZrLRgf5N5tgEXZmVUiOz98ArlTJJkRE3wkwnPxOaZKbZlNaFwFpJZTF0ppe5awPrKPUKpHm_g"
                ),
            ))
            .input_bool("STRICT_DEPS", true)
            .out("out")
            .out("dev")
            .optional(split_static_out, |builder| builder.out("static"))
            .configure_flags(format!(
                "{}{}",
                r#static.then_some("--static").unwrap_or_default(),
                shared.then_some("--shared").unwrap_or_default()
            ))
            .input_bool("DONT_DISABLE_STATIC", true)
            .input_bool("DONT_ADD_STATIC_CONFIGURE_FLAGS", true)
            .input_bool("SET_OUTPUT_FLAG", false)
            .input("OUTPUT_DOC", "dev")
            .post_install(format!(
                "{}",
                split_static_out
                    .then_some(r#"moveToOutput lib/libz.a "$static""#)
                    .unwrap_or_default()
            ))
            .input_bool("ENABLE_PARALLEL_BUILDING", true)
            .do_check()
            .make_flags(format!(
                "{}",
                shared.then_some("SHARED_MODE=1").unwrap_or_default()
            ))
            .build()
    }
}
