mod generic;
pub mod linux;

pub use generic::*;

use std::{ops::Deref, rc::Rc};

#[derive(Clone)]
#[repr(transparent)]
pub struct Stdenv(Rc<StdenvDrv>);

impl Stdenv {
    pub fn new(stdenv: StdenvDrv) -> Self {
        Self(Rc::new(stdenv))
    }
}

impl Deref for Stdenv {
    type Target = StdenvDrv;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
