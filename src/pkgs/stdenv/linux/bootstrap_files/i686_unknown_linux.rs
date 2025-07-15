use super::FetchBootstrapFile;
use oxide_core::{
    drv::{Drv, IntoDrv},
    hash,
};

pub struct BootstrapTools;

impl IntoDrv for BootstrapTools {
    fn into_drv(self) -> Drv {
        FetchBootstrapFile {
            url: "http://tarballs.nixos.org/stdenv/i686-unknown-linux-gnu/125cefd4cf8f857e5ff1aceaef9230ba578a033d/bootstrap-tools.tar.xz",
            hash: hash!("sha512:YzJSc1lXcHFabXhyWVhOa2FtWnNZWE5xWm14ellXcG1jMjNSc2FtWnNjMlJxYkdaaGFuTnNhMlpoYzJSclphcw"),
            exec: false,
        }.into_drv()
    }
}

pub struct Busybox;

impl IntoDrv for Busybox {
    fn into_drv(self) -> Drv {
        FetchBootstrapFile {
            url: "http://tarballs.nixos.org/stdenv/i686-unknown-linux-gnu/125cefd4cf8f857e5ff1aceaef9230ba578a033d/busybox",
            hash: hash!("sha512:YzJSc1lXcHFabXhyWVhOa2FtWnNZWE5xWm14ellXcG1jMjNSc2FtWnNjMlJxYkdaaGFuTnNhMlpoYzJSclphcw"),
            exec: true,
        }.into_drv()
    }
}
