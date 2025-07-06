use crate::stdenv::StdenvBuilder;
use oxide_core::{drv::DrvBuilder, types::Cow};

pub struct ConfigurePhase {
    pub configure: bool,
    pub configure_script: Option<Cow<str>>,
    pub configure_flags: Option<Cow<str>>,
    pub pre_configure: Option<Cow<str>>,
    pub configure_phase: Option<Cow<str>>,
    pub post_configure: Option<Cow<str>>,
}

impl ConfigurePhase {
    pub fn new() -> Self {
        Self {
            configure: true,
            configure_script: None,
            configure_flags: None,
            pre_configure: None,
            configure_phase: None,
            post_configure: None,
        }
    }

    pub fn build(self, builder: DrvBuilder) -> DrvBuilder {
        if self.configure {
            builder
                .input("CONFIGURE", "1")
                .input_if("CONFIGURE_SCRIPT", self.configure_script)
                .input_if("CONFIGURE_FLAGS", self.configure_flags)
                .input_if("PRE_CONFIGURE", self.pre_configure)
                .input_if("CONFIGURE_PHASE", self.configure_phase)
                .input_if("POST_CONFIGURE", self.post_configure)
        } else {
            builder
        }
    }
}

impl Default for ConfigurePhase {
    fn default() -> Self {
        Self::new()
    }
}

impl StdenvBuilder {
    pub fn dont_configure(mut self) -> Self {
        self.configure.configure = false;
        self
    }

    pub fn configure_script<T>(mut self, configure_script: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.configure.configure_script = Some(configure_script.into());
        self
    }

    pub fn configure_flags<T>(mut self, configure_flags: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.configure.configure_flags = Some(configure_flags.into());
        self
    }

    pub fn pre_configure<T>(mut self, pre_configure: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.configure.pre_configure = Some(pre_configure.into());
        self
    }

    pub fn configure_phase<T>(mut self, configure_phase: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.configure.configure_phase = Some(configure_phase.into());
        self
    }

    pub fn post_configure<T>(mut self, post_configure: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.configure.post_configure = Some(post_configure.into());
        self
    }
}
