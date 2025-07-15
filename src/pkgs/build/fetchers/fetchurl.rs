use crate::stdenv::Stdenv;
use oxide_core::{builtins, prelude::*};

#[derive(Clone)]
pub enum FetchUrl {
    Stdenv(StdenvFetchUrl),
    Builtins,
}

impl FetchUrl {
    pub fn new(fetchurl: StdenvFetchUrl) -> Self {
        Self::Stdenv(fetchurl)
    }

    pub fn from_builtins() -> Self {
        Self::Builtins
    }

    // TODO: for now super simple, only url and hash
    // expand in the future to allow more arguments
    pub fn fetch<T>(&self, url: T, hash: Hash) -> LazyDrv
    where
        T: Into<Cow<str>>,
    {
        let url = url.into();
        match self {
            FetchUrl::Stdenv(fetchurl) => LazyDrv::new(FetchUrlParam {
                stdenv_no_cc: fetchurl.stdenv_no_cc.clone(),
                curl: LazyDrv::clone(&fetchurl.curl),
                url,
                hash,
            }),
            FetchUrl::Builtins => LazyDrv::new(builtins::FetchUrl {
                name: None,
                url,
                hash,
                unpack: false,
                executable: false,
            }),
        }
    }
}

#[derive(Clone)]
pub struct StdenvFetchUrl {
    pub stdenv_no_cc: Stdenv,
    pub curl: LazyDrv,
}

struct FetchUrlParam {
    stdenv_no_cc: Stdenv,
    curl: LazyDrv,
    url: Cow<str>,
    hash: Hash,
}

impl IntoDrv for FetchUrlParam {
    fn into_drv(self) -> Drv {
        let name = base_name(&self.url).to_string();
        self.stdenv_no_cc
            .make_derivation()
            .name(name)
            .builder(local_file!("builder.sh"))
            .fixed_hash(self.hash)
            .input("url", self.url)
            .input("curl", self.curl)
            .build()
    }
}
