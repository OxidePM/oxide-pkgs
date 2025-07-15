use super::{
    BuildPhase, CheckPhase, ConfigurePhase, Deps, FixPhase, InstallCheckPhase, InstallPhase,
    PatchPhase, UnpackPhase,
};
use crate::stdenv::StdenvDrv;
use oxide_core::{
    drv::{Drv, DrvBuilder, IntoDrv, LazyDrv},
    expr::Expr,
    hash::Hash,
    local_file,
    system::System,
    types::Cow,
};

pub struct StdenvBuilder {
    pub(super) stdenv: StdenvDrv,
    // default drv args
    pub(super) drv_builder: DrvBuilder,
    pub(super) name: Option<Cow<str>>,
    pub(super) version: Option<Cow<str>>,
    pub(super) builder: Option<Expr>,
    // stdenv drv args
    pub(super) src: Option<Expr>,
    pub(super) build_command: Option<Expr>,
    // deps
    pub(super) deps: Deps,
    pub(super) propagated: Deps,
    // phases
    pub(super) pre_phase: Option<Cow<str>>,
    pub(super) unpack: UnpackPhase,
    pub(super) patch: PatchPhase,
    pub(super) configure: ConfigurePhase,
    pub(super) build: BuildPhase,
    pub(super) check: CheckPhase,
    pub(super) install: InstallPhase,
    pub(super) fix: FixPhase,
    pub(super) install_check: InstallCheckPhase,
    pub(super) post_phase: Option<Cow<str>>,
}

impl StdenvBuilder {
    pub fn new(stdenv: StdenvDrv) -> Self {
        Self {
            stdenv,
            drv_builder: DrvBuilder::new(),
            name: None,
            version: None,
            builder: None,
            src: None,
            build_command: None,
            deps: Deps::new(),
            propagated: Deps::new(),
            pre_phase: None,
            unpack: UnpackPhase::new(),
            patch: PatchPhase::new(),
            configure: ConfigurePhase::new(),
            build: BuildPhase::new(),
            check: CheckPhase::new(),
            install: InstallPhase::new(),
            fix: FixPhase::new(),
            install_check: InstallCheckPhase::new(),
            post_phase: None,
        }
    }
}

impl StdenvBuilder {
    pub fn name<T>(mut self, name: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.name = Some(name.into());
        self
    }

    pub fn version<T>(mut self, version: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.version = Some(version.into());
        self
    }

    pub fn out<T>(mut self, out: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.drv_builder = self.drv_builder.out(out);
        self
    }

    pub fn fixed_hash(mut self, hash: Hash) -> Self {
        self.drv_builder = self.drv_builder.fixed_hash(hash);
        self
    }

    pub fn system(mut self, system: System) -> Self {
        self.drv_builder = self.drv_builder.system(system);
        self
    }

    pub fn input<K, V>(mut self, key: K, expr: V) -> Self
    where
        K: Into<String>,
        V: Into<Expr>,
    {
        self.drv_builder = self.drv_builder.input(key, expr);
        self
    }

    pub fn input_if<K, V>(mut self, key: K, expr: Option<V>) -> Self
    where
        K: Into<String>,
        V: Into<Expr>,
    {
        self.drv_builder = self.drv_builder.input_if(key, expr);
        self
    }

    pub fn input_bool<K>(mut self, key: K, v: bool) -> Self
    where
        K: Into<String>,
    {
        self.drv_builder = self.drv_builder.input_bool(key, v);
        self
    }

    pub fn optional<F>(self, v: bool, f: F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        if v {
            f(self)
        } else {
            self
        }
    }

    pub fn builder<T>(mut self, builder: T) -> Self
    where
        T: Into<Expr>,
    {
        self.builder = Some(builder.into());
        self
    }

    pub fn src<T>(mut self, src: T) -> Self
    where
        T: Into<Expr>,
    {
        self.src = Some(src.into());
        self
    }

    pub fn build_command<T>(mut self, build_command: T) -> Self
    where
        T: Into<Expr>,
    {
        self.build_command = Some(build_command.into());
        self
    }

    pub fn pre_phase<T>(mut self, pre_phase: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.pre_phase = Some(pre_phase.into());
        self
    }

    pub fn post_phase<T>(mut self, post_phase: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.post_phase = Some(post_phase.into());
        self
    }

    pub fn build(self) -> Drv {
        let name = self.name.expect("name must be provided");
        // TODO: should we allow derivations with no version???
        let versioned_name = if let Some(version) = self.version {
            format!("{name}-{version}")
        } else {
            name.to_string()
        };
        let builder = self
            .drv_builder
            .name(versioned_name)
            .builder(self.stdenv.shell.clone())
            .arg("-e")
            .arg(local_file!("scripts/source-stdenv.sh"))
            .arg(
                self.builder
                    .unwrap_or(local_file!("scripts/default-builder.sh")),
            )
            .input("stdenv", LazyDrv::new(self.stdenv))
            .input_if("SRC", self.src)
            .input_if("BUILD_COMMAND", self.build_command)
            .input_if("PRE_PHASE", self.pre_phase)
            .input_if("POST_PHASE", self.post_phase);
        let builder = self.deps.build(builder, false);
        let builder = self.propagated.build(builder, true);
        let builder = self.unpack.build(builder);
        let builder = self.patch.build(builder);
        let builder = self.configure.build(builder);
        let builder = self.build.build(builder);
        let builder = self.check.build(builder);
        let builder = self.install.build(builder);
        let builder = self.fix.build(builder);
        let builder = self.install_check.build(builder);
        builder.build()
    }

    pub fn lazy(self) -> LazyDrv {
        pub struct Wrapper(StdenvBuilder);
        impl IntoDrv for Wrapper {
            fn into_drv(self) -> Drv {
                self.0.build()
            }
        }
        LazyDrv::new(Wrapper(self))
    }
}
