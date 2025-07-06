use crate::stdenv::StdenvBuilder;
use oxide_core::{drv::DrvBuilder, types::Cow};

pub struct CheckPhase {
    pub check: bool,
    pub check_flags: Option<Cow<str>>,
    pub pre_check: Option<Cow<str>>,
    pub check_phase: Option<Cow<str>>,
    pub post_check: Option<Cow<str>>,
}

impl CheckPhase {
    pub fn new() -> Self {
        Self {
            check: false,
            check_flags: None,
            pre_check: None,
            check_phase: None,
            post_check: None,
        }
    }

    pub fn build(self, builder: DrvBuilder) -> DrvBuilder {
        if self.check {
            builder
                .input("CHECK", "1")
                .input_if("CHECK_FLAGS", self.check_flags)
                .input_if("PRE_CHECK", self.pre_check)
                .input_if("CHECK_PHASE", self.check_phase)
                .input_if("POST_CHECK", self.post_check)
        } else {
            builder
        }
    }
}

impl Default for CheckPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl StdenvBuilder {
    pub fn do_check(mut self) -> Self {
        self.check.check = true;
        self
    }

    pub fn check_flags<T>(mut self, check_flags: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.check.check_flags = Some(check_flags.into());
        self
    }

    pub fn pre_check<T>(mut self, pre_check: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.check.pre_check = Some(pre_check.into());
        self
    }

    pub fn check_phase<T>(mut self, check_phase: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.check.check_phase = Some(check_phase.into());
        self
    }

    pub fn post_check<T>(mut self, post_check: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.check.post_check = Some(post_check.into());
        self
    }
}
