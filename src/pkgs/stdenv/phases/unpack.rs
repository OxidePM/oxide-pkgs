use crate::stdenv::StdenvBuilder;
use oxide_core::{drv::DrvBuilder, types::Cow};

pub struct UnpackPhase {
    pub unpack: bool,
    pub pre_unpack: Option<Cow<str>>,
    pub unpack_phase: Option<Cow<str>>,
    pub post_unpack: Option<Cow<str>>,
}

impl UnpackPhase {
    pub fn new() -> Self {
        Self {
            unpack: true,
            pre_unpack: None,
            unpack_phase: None,
            post_unpack: None,
        }
    }

    pub fn build(self, builder: DrvBuilder) -> DrvBuilder {
        if self.unpack {
            builder
                .input("UNPACK", "1")
                .input_if("PRE_UNPACK", self.pre_unpack)
                .input_if("UNPACK_PHASE", self.unpack_phase)
                .input_if("POST_UNPACK", self.post_unpack)
        } else {
            builder
        }
    }
}

impl Default for UnpackPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl StdenvBuilder {
    pub fn dont_unpack(mut self) -> Self {
        self.unpack.unpack = false;
        self
    }

    pub fn pre_unpack<T>(mut self, pre_unpack: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.unpack.pre_unpack = Some(pre_unpack.into());
        self
    }

    pub fn unpack_phase<T>(mut self, unpack_phase: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.unpack.unpack_phase = Some(unpack_phase.into());
        self
    }

    pub fn post_unpack<T>(mut self, post_unpack: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.unpack.post_unpack = Some(post_unpack.into());
        self
    }
}
