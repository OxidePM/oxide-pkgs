use crate::{
    build::{
        curl::Curl,
        fetchurl::{FetchUrl, StdenvFetchUrl},
        pkg_config::PkgConfig,
    },
    development::{
        interpreters::perl::Perl,
        libraries::{libiconv::LibIConv, zlib::Zlib},
    },
    misc::hello::Hello,
    stdenv::{self, Stdenv, linux::Noop},
};
use oxide_core::prelude::*;
use std::collections::HashMap;

pub struct AllPkgs {
    pub stdenv: Stdenv,
    pub fetchurl: FetchUrl,
    pub zlib: LazyDrv,
    pub libiconv: LazyDrv,
    pub pkg_config: LazyDrv,
    pub perl: LazyDrv,
    pub curl: LazyDrv,
    pub hello: LazyDrv,
}

// TODO: make it more ergonomic
pub fn all_pkgs() -> (HashMap<String, LazyDrv>, Box<AllPkgs>) {
    let mut pkgs = HashMap::new();
    let stdenv = build_stdenv();
    let fetchurl = build_fetchurl(&stdenv);

    let zlib = LazyDrv::new(Zlib {
        stdenv: Stdenv::clone(&stdenv),
        fetchurl: FetchUrl::clone(&fetchurl),
        shared: None,
        r#static: None,
        split_static_out: None,
    });
    pkgs.insert("zlib".to_string(), LazyDrv::clone(&zlib));

    let libiconv = LazyDrv::new(LibIConv {
        stdenv: Stdenv::clone(&stdenv),
        fetchurl: FetchUrl::clone(&fetchurl),
        update_autotools_gnu_config_scripts: "".into(),
        shared: None,
        r#static: None,
    });
    pkgs.insert("libiconv".to_string(), LazyDrv::clone(&libiconv));

    let pkg_config = LazyDrv::new(PkgConfig {
        stdenv: Stdenv::clone(&stdenv),
        fetchurl: FetchUrl::clone(&fetchurl),
        libiconv: LazyDrv::clone(&libiconv),
        vanilla: None,
    });
    pkgs.insert("pkg-config".to_string(), LazyDrv::clone(&pkg_config));

    let perl = LazyDrv::new(Perl {
        stdenv: Stdenv::clone(&stdenv),
        fetchurl: FetchUrl::clone(&fetchurl),
        zlib: LazyDrv::clone(&zlib),
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
            zlib,
            libiconv,
            pkg_config,
            perl,
            curl,
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
                libiconv: LazyDrv::new(LibIConv {
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
