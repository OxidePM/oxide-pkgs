use super::StdenvBuilder;
use oxide_core::{drv::DrvBuilder, expr::Expr};

#[derive(Default)]
pub struct Deps {
    pub build_build: Vec<Expr>,
    pub build_host: Vec<Expr>,
    pub build_target: Vec<Expr>,
    pub host_host: Vec<Expr>,
    pub host_target: Vec<Expr>,
    pub target_target: Vec<Expr>,
}

impl Deps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self, builder: DrvBuilder, propagated: bool) -> DrvBuilder {
        if propagated {
            builder
                .input("PROPAGATED_BUILD_BUILD", self.build_build)
                .input("PROPAGATED_BUILD_HOST", self.build_host)
                .input("PROPAGATED_BUILD_TARGET", self.build_target)
                .input("PROPAGATED_HOST_HOST", self.host_host)
                .input("PROPAGATED_HOST_TARGET", self.host_target)
                .input("PROPAGATED_TARGET_TARGET", self.target_target)
        } else {
            builder
                .input("DEPS_BUILD_BUILD", self.build_build)
                .input("DEPS_BUILD_HOST", self.build_host)
                .input("DEPS_BUILD_TARGET", self.build_target)
                .input("DEPS_HOST_HOST", self.host_host)
                .input("DEPS_HOST_TARGET", self.host_target)
                .input("DEPS_TARGET_TARGET", self.target_target)
        }
    }
}

impl StdenvBuilder {
    pub fn dep_build_build<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.build_build.push(dep.into());
        self
    }

    pub fn dep_build_host<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.build_host.push(dep.into());
        self
    }

    pub fn dep_build_target<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.build_target.push(dep.into());
        self
    }

    pub fn dep_host_host<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.host_host.push(dep.into());
        self
    }

    pub fn dep_host_target<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.host_target.push(dep.into());
        self
    }

    pub fn dep_target_target<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.target_target.push(dep.into());
        self
    }

    pub fn propagated_build_build<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.build_build.push(dep.into());
        self
    }

    pub fn propagated_build_host<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.build_host.push(dep.into());
        self
    }

    pub fn propagated_build_target<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.build_target.push(dep.into());
        self
    }

    pub fn propagated_host_host<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.host_host.push(dep.into());
        self
    }

    pub fn propagated_host_target<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.host_target.push(dep.into());
        self
    }

    pub fn propagated_target_target<T>(mut self, dep: T) -> Self
    where
        T: Into<Expr>,
    {
        self.deps.target_target.push(dep.into());
        self
    }
}
