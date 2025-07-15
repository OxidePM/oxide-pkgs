use super::FetchBootstrapFile;
use oxide_core::{
    drv::{Drv, IntoDrv},
    hash,
};

pub struct BootstrapTools;

impl IntoDrv for BootstrapTools {
    fn into_drv(self) -> Drv {
        FetchBootstrapFile {
            url: "http://tarballs.nixos.org/stdenv/x86_64-unknown-linux-gnu/82b583ba2ba2e5706b35dbe23f31362e62be2a9d/bootstrap-tools.tar.xz",
            hash: hash!("sha512:rE3awurje4X3WxwmSf03-Acb8O8DBSBOmNk2f094EfVd5y9x8wt0nhenj14vp_jGtkMTTo64dpelx2V7TcluJA"),
            exec: false,
        }.into_drv()
    }
}

pub struct Busybox;

impl IntoDrv for Busybox {
    fn into_drv(self) -> Drv {
        FetchBootstrapFile {
            url: "http://tarballs.nixos.org/stdenv/x86_64-unknown-linux-gnu/82b583ba2ba2e5706b35dbe23f31362e62be2a9d/busybox",
            hash: hash!("sha512:j3Egq7E25ffdZsvHZNEHkISHyzaj_XGasw4VRd1O4D-5b0ME3ASGT6Yij_ESgu8ry5gjqQMLEb-6PKPwceOH6Q"),
            exec: true,
        }.into_drv()
    }
}
