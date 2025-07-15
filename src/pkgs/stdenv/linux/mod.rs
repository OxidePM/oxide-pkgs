use crate::{
    build::fetchurl::FetchUrl, development::interpreters::perl, stdenv::generic::StdenvDrv,
};
use bootstrap_files::{BootstrapFiles, i686_unknown_linux, x86_64_unknown_linux};
use bootstrap_tools::BootstrapTools;
use oxide_core::{
    drv::{Drv, DrvBuilder, IntoDrv, LazyDrv},
    expr,
    system::System,
};

use super::Stdenv;

pub mod bootstrap_files;
pub mod bootstrap_tools;

pub enum StdenvStage {
    Stage0 {
        local_system: System,
        glibc: bool,
        bootstrap_tools: LazyDrv,
    },
    Stage1 {
        local_system: System,
        glibc: bool,
        bootstrap_tools: LazyDrv,
        prev_stage: StdenvDrv,
    },
    Stage2 {
        local_system: System,
        glibc: bool,
        bootstrap_tools: LazyDrv,
        prev_stage: StdenvDrv,
    },
    Stage3 {
        local_system: System,
        glibc: bool,
        bootstrap_tools: LazyDrv,
        prev_stage: StdenvDrv,
    },
    Stdenv {
        stdenv: StdenvDrv,
    },
}

// TODO: group local_system, glibc and other system information in a single struct
pub fn build_stdenv(local_system: System, glibc: bool) -> StdenvDrv {
    let bootstrap_files = match local_system {
        System::x86_64_linux => BootstrapFiles {
            tools: LazyDrv::new(x86_64_unknown_linux::BootstrapTools),
            busybox: LazyDrv::new(x86_64_unknown_linux::Busybox),
        },
        System::i686_linux => BootstrapFiles {
            tools: LazyDrv::new(i686_unknown_linux::BootstrapTools),
            busybox: LazyDrv::new(i686_unknown_linux::Busybox),
        },
        _ => unimplemented!(),
    };

    let bootstrap_tools = LazyDrv::new(BootstrapTools {
        system: local_system,
        bootstrap_files,
        glibc,
    });

    let mut stage = stager(StdenvStage::Stage0 {
        local_system,
        glibc,
        bootstrap_tools,
    });
    loop {
        stage = stager(stage);
        if let StdenvStage::Stdenv { stdenv } = stage {
            return stdenv;
        }
    }
}

// TODO: remove Noop it is just for testing
pub struct Noop;

impl IntoDrv for Noop {
    fn into_drv(self) -> Drv {
        DrvBuilder::new()
            .name("no-op")
            .builder(r#"echo "no-op""#)
            .build()
    }
}

pub fn stager(stage: StdenvStage) -> StdenvStage {
    let common_pre_hook = r#"echo "common pre hook""#;
    match stage {
        StdenvStage::Stage0 {
            bootstrap_tools,
            local_system,
            glibc,
        } => {
            let mut stdenv = StdenvDrv {
                name: "bootstrap-stage0-stdenv-linux",
                initial_path: expr![LazyDrv::clone(&bootstrap_tools)],
                pre_hook: Some(format!(
                    r##"
                    # Don't patch #!/interpreter because it leads to retained
                    # dependencies on the bootstrapTools in the final stdenv.
                    DONT_PATCH_SHEBANGS=1
                    {common_pre_hook}
                    "##
                )),
                cc: None,
                shell: bootstrap_tools.suff("/bin/bash").into(),
                setup_script: None,
                build_platform: local_system,
                host_platform: local_system,
                target_platform: local_system,
                deps_build_host: Vec::new(),
                deps_host_target: Vec::new(),
                glibc: None,
                binutils: None,
                coreutils: None,
                gnugrep: None,
                perl: None,
            };
            if glibc {
                stdenv.glibc = Some(
                    stdenv
                        .make_derivation()
                        .name("bootstrap-stage0-glibc")
                        .version("bootstrap-files")
                        // TODO: this would not be necessary once we will have format strings
                        .out("out")
                        .out("dev")
                        .out("lib")
                        .input("bootstrap_tools", &bootstrap_tools)
                        .build_command(
                            r#"mkdir -p $out
ln -s ${bootstrap_tools}/lib $out/lib
ln -s ${bootstrap_tools}/include-glibc $out/include
# TODO: remove
mkdir "$dev"
mkdir "$lib""#,
                        )
                        .lazy(),
                );
            } else {
                panic!("only glibc support now")
            };
            stdenv.binutils = Some(LazyDrv::clone(&bootstrap_tools));
            stdenv.coreutils = Some(LazyDrv::clone(&bootstrap_tools));
            stdenv.gnugrep = Some(LazyDrv::clone(&bootstrap_tools));

            // StdenvStage::Stage1 {
            //     local_system,
            //     glibc,
            //     bootstrap_tools,
            //     prev_stage: stdenv,
            // }
            // TODO: for now use impure stdenv because
            // I still have to understand how to replicate the pure stdenv
            StdenvStage::Stdenv { stdenv }
        }
        StdenvStage::Stage1 {
            bootstrap_tools,
            local_system,
            glibc,
            prev_stage,
        } => {
            let stdenv = StdenvDrv {
                name: "bootstrap-stage1-stdenv-linux",
                perl: Some(LazyDrv::new(perl::Perl {
                    stdenv: Stdenv::new(prev_stage.clone()),
                    fetchurl: FetchUrl::Builtins,
                    zlib: LazyDrv::new(Noop),
                    enable_threading: false,
                })),
                ..prev_stage
            };
            StdenvStage::Stage2 {
                local_system,
                glibc,
                bootstrap_tools,
                prev_stage: stdenv,
            }
        }
        StdenvStage::Stage2 {
            local_system,
            glibc,
            bootstrap_tools,
            prev_stage,
        } => {
            let stdenv = StdenvDrv {
                name: "bootstrap-stage-xgcc-stdenv-linux",
                ..prev_stage
            };
            StdenvStage::Stage3 {
                local_system,
                glibc,
                bootstrap_tools,
                prev_stage: stdenv,
            }
        }
        StdenvStage::Stage3 { prev_stage, .. } => StdenvStage::Stdenv { stdenv: prev_stage },
        // no-op: should be unreachable
        StdenvStage::Stdenv { stdenv } => StdenvStage::Stdenv { stdenv },
    }
}
