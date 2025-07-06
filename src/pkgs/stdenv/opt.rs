use super::{
    Stdenv,
    phases::{
        BuildPhase, CheckPhase, ConfigurePhase, FixPhase, InstallCheckPhase, InstallPhase,
        PatchPhase, UnpackPhase,
    },
};
use oxide_core::{
    drv::{Drv, DrvBuilder, IntoDrv},
    expr::Expr,
    hash::Hash,
    local_file,
    system::System,
    types::Cow,
};

#[derive(Default)]
pub struct Deps {
    pub build_build: Vec<Expr>,
    pub build_host: Vec<Expr>,
    pub build_target: Vec<Expr>,
    pub host_host: Vec<Expr>,
    pub host_target: Vec<Expr>,
    pub target_target: Vec<Expr>,
}

impl Deps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self, builder: DrvBuilder, propagated: bool) -> DrvBuilder {
        if propagated {
            builder
                .input("PROPAGATED_BUILD_BUILD", self.build_build)
                .input("PROPAGATED_BUILD_HOST", self.build_host)
                .input("PROPAGATED_BUILD_TARGET", self.build_target)
                .input("PROPAGATED_HOST_HOST", self.host_host)
                .input("PROPAGATED_HOST_TARGET", self.host_target)
                .input("PROPAGATED_TARGET_TARGET", self.target_target)
        } else {
            builder
                .input("DEPS_BUILD_BUILD", self.build_build)
                .input("DEPS_BUILD_HOST", self.build_host)
                .input("DEPS_BUILD_TARGET", self.build_target)
                .input("DEPS_HOST_HOST", self.host_host)
                .input("DEPS_HOST_TARGET", self.host_target)
                .input("DEPS_TARGET_TARGET", self.target_target)
        }
    }
}

pub struct StdenvBuilder {
    // default drv args
    pub(super) drv_builder: DrvBuilder,
    pub(super) name: Option<Cow<str>>,
    pub(super) version: Option<Cow<str>>,
    pub(super) builder: Option<Expr>,
    // stdenv drv args
    pub(super) src: Option<Expr>,
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
    pub fn new(stdenv: &Stdenv) -> Self {
        let drv_builder = DrvBuilder::new().input("stdenv", &**stdenv);
        Self {
            drv_builder,
            name: None,
            version: None,
            builder: None,
            src: None,
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

    pub fn builder<T>(mut self, builder: T) -> Self
    where
        T: Into<Expr>,
    {
        self.builder = Some(builder.into());
        self
    }

    pub fn arg<T>(mut self, arg: T) -> Self
    where
        T: Into<Expr>,
    {
        self.drv_builder = self.drv_builder.arg(arg);
        self
    }

    pub fn src<T>(mut self, src: T) -> Self
    where
        T: Into<Expr>,
    {
        self.src = Some(src.into());
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
        let version = self.version.expect("version must be provided");
        let versioned_name = format!("{name}-{version}");
        let builder = self
            .drv_builder
            .name(versioned_name)
            .builder(self.builder.unwrap_or(local_file!("builder.sh")))
            .input_if("SRC", self.src)
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
}

impl IntoDrv for StdenvBuilder {
    fn into_drv(self: Box<Self>) -> Drv {
        self.build()
    }
}
