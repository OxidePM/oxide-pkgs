use crate::{
    build::{
        curl::Curl,
        fetchurl::{FetchUrl, StdenvFetchUrl},
        pkg_config::PkgConfig,
    },
    development::{
        interpreters::perl::Perl,
        libraries::{libiconv::LibiConv, zlib::Zlib},
    },
    misc::hello::Hello,
    stdenv::{self, linux::Noop, Stdenv},
};
use oxide_core::prelude::*;
use std::collections::HashMap;

pub struct AllPkgs {
    pub stdenv: Stdenv,
    pub fetchurl: FetchUrl,
    pub perl: LazyDrv,
    pub hello: LazyDrv,
}

// TODO: make it more ergonomic
// TODO: libc
// TODO: zlib
// TODO: pkg-config
pub fn all_pkgs() -> (HashMap<String, LazyDrv>, Box<AllPkgs>) {
    let mut pkgs = HashMap::new();
    let stdenv = build_stdenv();
    let fetchurl = build_fetchurl(&stdenv);

    let perl = LazyDrv::new(Perl {
        stdenv: Stdenv::clone(&stdenv),
        fetchurl: FetchUrl::clone(&fetchurl),
        zlib: LazyDrv::new(Zlib {
            stdenv: Stdenv::clone(&stdenv),
            fetchurl: FetchUrl::clone(&fetchurl),
            shared: None,
            r#static: None,
            split_static_out: None,
        }),
        enable_threading: true,
    });
    pkgs.insert("perl".to_string(), LazyDrv::clone(&perl));

    let curl = LazyDrv::new(Curl {
        stdenv: Stdenv::clone(&stdenv),
        fetchurl: FetchUrl::clone(&fetchurl),
        pkg_config: LazyDrv::new(Noop),
        perl: LazyDrv::clone(&perl),
    });
    pkgs.insert("curl".to_string(), LazyDrv::clone(&curl));

    let hello = LazyDrv::new(Hello {
        stdenv: Stdenv::clone(&stdenv),
        fetchurl: FetchUrl::clone(&fetchurl),
    });
    pkgs.insert("hello".to_string(), LazyDrv::clone(&hello));
    (
        pkgs,
        Box::new(AllPkgs {
            stdenv,
            fetchurl,
            perl,
            hello,
        }),
    )
}

fn build_stdenv() -> Stdenv {
    let system = current_system();
    match system {
        System::x86_64_linux | System::i686_linux => {
            Stdenv::new(stdenv::linux::build_stdenv(system, true))
        }
        _ => unimplemented!(),
    }
}

pub fn build_fetchurl(stdenv: &Stdenv) -> FetchUrl {
    // to build fetchurl we must use builtins fetchurl to fetch its dependencies
    FetchUrl::new(StdenvFetchUrl {
        stdenv_no_cc: Stdenv::clone(&stdenv),
        curl: LazyDrv::new(Curl {
            stdenv: Stdenv::clone(&stdenv),
            fetchurl: FetchUrl::Builtins,
            pkg_config: LazyDrv::new(PkgConfig {
                stdenv: Stdenv::clone(&stdenv),
                fetchurl: FetchUrl::Builtins,
                libiconv: LazyDrv::new(LibiConv {
                    stdenv: Stdenv::clone(&stdenv),
                    fetchurl: FetchUrl::Builtins,
                    update_autotools_gnu_config_scripts: "".into(),
                    r#static: None,
                    shared: None,
                }),
                vanilla: None,
            }),
            perl: LazyDrv::new(Perl {
                stdenv: Stdenv::clone(&stdenv),
                fetchurl: FetchUrl::Builtins,
                zlib: LazyDrv::new(Zlib {
                    stdenv: Stdenv::clone(&stdenv),
                    fetchurl: FetchUrl::Builtins,
                    shared: None,
                    r#static: None,
                    split_static_out: None,
                }),
                enable_threading: true,
            }),
        }),
    })
}
