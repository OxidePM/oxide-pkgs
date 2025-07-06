mod opt;
pub use opt::*;

mod phases;
pub use phases::*;

use oxide_core::drv::{Drv, IntoDrv, LazyDrv};
use std::ops::Deref;

pub struct StdenvDrv;

impl IntoDrv for StdenvDrv {
    fn into_drv(self: Box<Self>) -> Drv {
        unimplemented!()
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Stdenv(LazyDrv);

impl Deref for Stdenv {
    type Target = LazyDrv;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Stdenv {
    pub fn new(stdenv: StdenvDrv) -> Self {
        Self(LazyDrv::new(stdenv))
    }

    pub fn builder(&self) -> StdenvBuilder {
        StdenvBuilder::new(self)
    }
}
