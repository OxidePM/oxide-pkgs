pub mod i686_unknown_linux;
pub mod x86_64_unknown_linux;

use oxide_core::{
    builtins,
    drv::{Drv, IntoDrv, LazyDrv},
    hash::Hash,
};

pub struct BootstrapFiles {
    pub tools: LazyDrv,
    pub busybox: LazyDrv,
}

pub struct FetchBootstrapFile {
    pub url: &'static str,
    pub hash: Hash,
    pub exec: bool,
}

impl IntoDrv for FetchBootstrapFile {
    fn into_drv(self) -> Drv {
        builtins::FetchUrl {
            name: None,
            url: self.url.into(),
            hash: self.hash,
            unpack: false,
            executable: self.exec,
        }
        .into_drv()
    }
}
