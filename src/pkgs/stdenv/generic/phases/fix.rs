use crate::stdenv::StdenvBuilder;
use oxide_core::{drv::DrvBuilder, types::Cow};

pub struct FixPhase {
    pub fix: bool,
    pub strip: bool,
    pub patch_elf: bool,
    pub pre_fix: Option<Cow<str>>,
    pub fix_phase: Option<Cow<str>>,
    pub post_fix: Option<Cow<str>>,
}

impl FixPhase {
    pub fn new() -> Self {
        Self {
            fix: true,
            strip: true,
            patch_elf: true,
            pre_fix: None,
            fix_phase: None,
            post_fix: None,
        }
    }

    pub fn build(self, builder: DrvBuilder) -> DrvBuilder {
        if self.fix {
            builder
                .input("FIX", "1")
                .input_if("STRIP", self.strip.then(|| "1"))
                .input_if("PATCH_ELF", self.patch_elf.then(|| "1"))
                .input_if("PRE_FIX", self.pre_fix)
                .input_if("FIX_PHASE", self.fix_phase)
                .input_if("POST_FIX", self.post_fix)
        } else {
            builder
        }
    }
}

impl Default for FixPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl StdenvBuilder {
    pub fn dont_fix(mut self) -> Self {
        self.fix.fix = false;
        self
    }

    pub fn dont_strip<T>(mut self) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.fix.strip = false;
        self
    }

    pub fn dont_patch_elf<T>(mut self) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.fix.patch_elf = false;
        self
    }

    pub fn pre_fix<T>(mut self, pre_fix: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.fix.pre_fix = Some(pre_fix.into());
        self
    }

    pub fn fix_phase<T>(mut self, fix_phase: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.fix.fix_phase = Some(fix_phase.into());
        self
    }

    pub fn post_fix<T>(mut self, post_fix: T) -> Self
    where
        T: Into<Cow<str>>,
    {
        self.fix.post_fix = Some(post_fix.into());
        self
    }
}
