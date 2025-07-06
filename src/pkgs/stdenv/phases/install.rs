use crate::stdenv::StdenvBuilder;
use oxide_core::{drv::DrvBuilder, types::Cow};

pub struct InstallPhase {
    pub install: bool,
    pub install_targets: Option<Cow<str>>,
    pub install_flags: Option<Cow<str>>,
    pub pre_install: Option<Cow<str>>,
    pub install_phase: Option<Cow<str>>,
    pub post_install: Option<Cow<str>>,
}

impl InstallPhase {
    pub fn new() -> Self {
        Self {
            install: true,
            install_targets: None,
            install_flags: None,
            pre_install: None,
            install_phase: None,
            post_install: None,
        }
    }

    pub fn build(self, builder: DrvBuilder) -> DrvBuilder {
        if self.install {
            builder
                .input("INSTALL", "1")
                .input_if("INSTALL_TARGETS", self.install_targets)
                .input_if("INSTALL_FLAGS", self.install_flags)
                .input_if("PRE_INSTALL", self.pre_install)
                .input_if("INSTALL_PHASE", self.install_phase)
                .input_if("POST_INSTALL", self.post_install)
        } else {
            builder
        }
    }
}

impl Default for InstallPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl StdenvBuilder {
    pub fn dont_install(mut self) -> Self {
        self.install.install = false;
        self
    }

    pub fn install_targets<T>(mut self, install_targets: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.install.install_targets = Some(install_targets.into());
        self
    }

    pub fn install_flags<T>(mut self, install_flags: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.install.install_flags = Some(install_flags.into());
        self
    }

    pub fn pre_install<T>(mut self, pre_install: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.install.pre_install = Some(pre_install.into());
        self
    }

    pub fn install_phase<T>(mut self, install_phase: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.install.install_phase = Some(install_phase.into());
        self
    }

    pub fn post_install<T>(mut self, post_install: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.install.post_install = Some(post_install.into());
        self
    }
}
