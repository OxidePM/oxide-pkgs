use crate::stdenv::StdenvBuilder;
use oxide_core::{drv::DrvBuilder, expr::Expr, types::Cow};

pub struct PatchPhase {
    pub patch: bool,
    pub patches: Vec<Expr>,
    pub patch_flags: Option<Cow<str>>,
    pub pre_patch: Option<Cow<str>>,
    pub patch_phase: Option<Cow<str>>,
    pub post_patch: Option<Cow<str>>,
}

impl PatchPhase {
    pub fn new() -> Self {
        Self {
            patch: true,
            patches: Vec::new(),
            patch_flags: None,
            pre_patch: None,
            patch_phase: None,
            post_patch: None,
        }
    }

    pub fn build(self, builder: DrvBuilder) -> DrvBuilder {
        if self.patch {
            builder
                .input("PATCH", "1")
                .input_if("PATCHES", (!self.patches.is_empty()).then(|| self.patches))
                .input_if("PATCH_FLAGS", self.patch_flags)
                .input_if("PRE_PATCH", self.pre_patch)
                .input_if("PATCH_PHASE", self.patch_phase)
                .input_if("POST_PATCH", self.post_patch)
        } else {
            builder
        }
    }
}

impl Default for PatchPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl StdenvBuilder {
    pub fn dont_patch(mut self) -> Self {
        self.patch.patch = false;
        self
    }

    pub fn patch<T>(mut self, patch: T) -> Self
    where
        T: Into<Expr>,
    {
        self.patch.patches.push(patch.into());
        self
    }

    pub fn patch_flags<T>(mut self, patch_flags: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.patch.patch_flags = Some(patch_flags.into());
        self
    }

    pub fn pre_patch<T>(mut self, pre_patch: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.patch.pre_patch = Some(pre_patch.into());
        self
    }

    pub fn patch_phase<T>(mut self, patch_phase: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.patch.patch_phase = Some(patch_phase.into());
        self
    }

    pub fn post_patch<T>(mut self, post_patch: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.patch.post_patch = Some(post_patch.into());
        self
    }
}
