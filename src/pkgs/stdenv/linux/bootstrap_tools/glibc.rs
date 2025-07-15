use crate::stdenv::linux::bootstrap_files::BootstrapFiles;
use oxide_core::{
    drv::{Drv, DrvBuilder, IntoDrv},
    expr, local_file,
    system::System,
};

pub struct BootstrapTools {
    pub system: System,
    pub bootstrap_files: BootstrapFiles,
}

impl IntoDrv for BootstrapTools {
    fn into_drv(self) -> Drv {
        DrvBuilder::new()
            .name("bootstrap-tools")
            .builder(self.bootstrap_files.busybox)
            .arg("ash")
            .arg("-e")
            .arg(local_file!("glibc-unpack-bootstrap-tools.sh"))
            .system(self.system)
            .input("tarball", self.bootstrap_files.tools)
            .input("langC", "1")
            .input("langCC", "1")
            .input("isGNU", "1")
            .input(
                "hardeningunsupportedflags",
                expr![
                    "fortify3",
                    "shadowstack",
                    "pacret",
                    "stackclashprotection",
                    "trivialautovarinit",
                    "zerocallusedregs",
                ],
            )
            .build()
    }
}
