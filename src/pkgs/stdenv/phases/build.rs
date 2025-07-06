use crate::stdenv::StdenvBuilder;
use oxide_core::{drv::DrvBuilder, types::Cow};

pub struct BuildPhase {
    pub build: bool,
    pub makefile: Option<Cow<str>>,
    pub make_flags: Option<Cow<str>>,
    pub build_flags: Option<Cow<str>>,
    pub pre_build: Option<Cow<str>>,
    pub build_phase: Option<Cow<str>>,
    pub post_build: Option<Cow<str>>,
}

impl BuildPhase {
    pub fn new() -> Self {
        Self {
            build: true,
            makefile: None,
            make_flags: None,
            build_flags: None,
            pre_build: None,
            build_phase: None,
            post_build: None,
        }
    }

    pub fn build(self, builder: DrvBuilder) -> DrvBuilder {
        if self.build {
            builder
                .input("BUILD", "1")
                .input_if("MAKEFILE", self.makefile)
                .input_if("MAKE_FLAGS", self.make_flags)
                .input_if("PRE_BUILD", self.pre_build)
                .input_if("BUILD_PHASE", self.build_phase)
                .input_if("POST_BUILD", self.post_build)
        } else {
            builder
        }
    }
}

impl Default for BuildPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl StdenvBuilder {
    pub fn dont_build(mut self) -> Self {
        self.build.build = false;
        self
    }

    pub fn makefile<T>(mut self, makefile: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.build.makefile = Some(makefile.into());
        self
    }

    pub fn make_flags<T>(mut self, make_flags: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.build.make_flags = Some(make_flags.into());
        self
    }

    pub fn pre_build<T>(mut self, pre_build: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.build.pre_build = Some(pre_build.into());
        self
    }

    pub fn build_phase<T>(mut self, build_phase: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.build.build_phase = Some(build_phase.into());
        self
    }

    pub fn post_build<T>(mut self, post_build: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.build.post_build = Some(post_build.into());
        self
    }
}
