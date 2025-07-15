mod builder;
mod deps;
mod phases;

pub use builder::*;
pub use deps::*;
pub use phases::*;

use oxide_core::{
    drv::{Drv, DrvBuilder, IntoDrv, LazyDrv},
    expr::Expr,
    local_file,
    system::System,
};

#[allow(unused)]
#[derive(Clone)]
pub struct StdenvDrv {
    // generic
    pub name: &'static str,
    pub pre_hook: Option<String>,
    pub initial_path: Expr,
    pub cc: Option<LazyDrv>,
    pub shell: Expr,
    pub setup_script: Option<String>,
    pub build_platform: System,
    pub host_platform: System,
    pub target_platform: System,
    pub deps_build_host: Vec<Expr>,
    pub deps_host_target: Vec<Expr>,
    // specialized
    pub glibc: Option<LazyDrv>,
    pub binutils: Option<LazyDrv>,
    pub coreutils: Option<LazyDrv>,
    pub gnugrep: Option<LazyDrv>,
    pub perl: Option<LazyDrv>,
}

impl IntoDrv for StdenvDrv {
    fn into_drv(self) -> Drv {
        let setup = self
            .setup_script
            .map_or(local_file!("scripts/setup.sh"), Expr::from);
        let mut deps_build_host = self.deps_build_host;
        if let Some(ref cc) = self.cc {
            deps_build_host.push(cc.clone().into());
        }
        DrvBuilder::new()
            .name(self.name)
            .builder(self.shell)
            .arg("-c")
            .arg(local_file!("scripts/builder.sh"))
            .input_if("PRE_HOOK", self.pre_hook)
            .input("INITIAL_PATH", self.initial_path)
            .input("SETUP", setup)
            .input_if("cc", self.cc)
            .input("DEFAULT_BUILD_HOST", deps_build_host)
            .input("DEFAULT_HOST_TARGET", self.deps_host_target)
            .input("BUILD_PLATFORM", self.build_platform.to_string())
            .input("HOST_PLATFORM", self.host_platform.to_string())
            .input("TARGET_PLATFORM", self.target_platform.to_string())
            .input_if("glibc", self.glibc)
            .input_if("binutils", self.binutils)
            .input_if("coreutils", self.coreutils)
            .input_if("gnugrep", self.gnugrep)
            .input_if("perl", self.perl)
            .build()
    }
}

impl StdenvDrv {
    pub fn make_derivation(&self) -> StdenvBuilder {
        StdenvBuilder::new(self.clone())
    }
}
