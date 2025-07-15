use crate::stdenv::StdenvBuilder;
use oxide_core::{drv::DrvBuilder, types::Cow};

pub struct InstallCheckPhase {
    pub install_check: bool,
    pub install_check_flags: Option<Cow<str>>,
    pub pre_install_check: Option<Cow<str>>,
    pub install_check_phase: Option<Cow<str>>,
    pub post_install_check: Option<Cow<str>>,
}

impl InstallCheckPhase {
    pub fn new() -> Self {
        Self {
            install_check: false,
            install_check_flags: None,
            pre_install_check: None,
            install_check_phase: None,
            post_install_check: None,
        }
    }

    pub fn build(self, builder: DrvBuilder) -> DrvBuilder {
        if self.install_check {
            builder
                .input("INSTALL_CHECK", "1")
                .input_if("INSTALL_CHECK_FLAGS", self.install_check_flags)
                .input_if("PRE_INSTALL_CHECK", self.pre_install_check)
                .input_if("INSTALL_CHECK_PHASE", self.install_check_phase)
                .input_if("POST_INSTALL_CHECK", self.post_install_check)
        } else {
            builder
        }
    }
}

impl Default for InstallCheckPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl StdenvBuilder {
    pub fn do_install_check(mut self) -> Self {
        self.install_check.install_check = true;
        self
    }

    pub fn install_check_flags<T>(mut self, install_check_flags: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.install_check.install_check_flags = Some(install_check_flags.into());
        self
    }

    pub fn pre_install_check<T>(mut self, pre_install_check: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.install_check.pre_install_check = Some(pre_install_check.into());
        self
    }

    pub fn install_check_phase<T>(mut self, install_check_phase: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.install_check.install_check_phase = Some(install_check_phase.into());
        self
    }

    pub fn post_install_check<T>(mut self, post_install_check: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.install_check.post_install_check = Some(post_install_check.into());
        self
    }
}
