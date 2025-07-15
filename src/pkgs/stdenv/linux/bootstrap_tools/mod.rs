mod glibc;

use super::bootstrap_files::BootstrapFiles;
use oxide_core::{
    drv::{Drv, IntoDrv},
    system::System,
};

pub struct BootstrapTools {
    pub bootstrap_files: BootstrapFiles,
    // TODO: add support for other libc and substitute with enum
    pub system: System,
    pub glibc: bool,
}

impl IntoDrv for BootstrapTools {
    fn into_drv(self) -> Drv {
        if self.glibc {
            glibc::BootstrapTools {
                system: self.system,
                bootstrap_files: self.bootstrap_files,
            }
            .into_drv()
        } else {
            panic!("at the moment only glibc is supported");
        }
    }
}
