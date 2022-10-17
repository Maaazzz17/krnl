use anyhow::Result;
use spirv::Capability;
use std::fmt::{self, Debug};

#[cfg(feature = "device")]
pub(crate) mod engine;
#[cfg(feature = "device")]
pub(crate) use engine::{
    ArcEngine as DeviceBase, Compute, DeviceBuffer, DeviceBufferInner, HostBuffer, KernelCache,
};

pub mod error {

    /// The "device" feature is not enabled.
    #[derive(Debug, thiserror::Error)]
    #[error("DeviceUnavailable")]
    pub struct DeviceUnavailable {}

    impl DeviceUnavailable {
        pub(crate) fn new() -> Self {
            Self {}
        }
    }
}
use error::*;

pub(crate) mod future {
    #[cfg(feature = "device")]
    pub(crate) use super::engine::HostBufferFuture;
}

mod options {
    use super::*;

    pub(super) struct DeviceOptions {
        pub(super) optimal_capabilities: Vec<Capability>,
    }

    impl Default for DeviceOptions {
        fn default() -> Self {
            use spirv::Capability::*;
            Self {
                optimal_capabilities: vec![VulkanMemoryModel],
            }
        }
    }
}
use options::DeviceOptions;

#[derive(Clone, derive_more::IsVariant, derive_more::Unwrap, Eq, PartialEq)]
pub(crate) enum DeviceInner {
    Host,
    #[cfg(feature = "device")]
    Device(DeviceBase),
}

impl DeviceInner {
    #[cfg(feature = "device")]
    pub(crate) fn device(&self) -> Option<&DeviceBase> {
        if let Self::Device(device) = self {
            Some(device)
        } else {
            None
        }
    }
    pub(crate) fn kind(&self) -> DeviceKind {
        match self {
            Self::Host => DeviceKind::Host,
            #[cfg(feature = "device")]
            Self::Device(_) => DeviceKind::Device,
        }
    }
}

pub(crate) enum DeviceKind {
    Host,
    #[cfg(feature = "device")]
    Device,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Device {
    pub(crate) inner: DeviceInner,
}

impl Device {
    pub fn host() -> Self {
        Self {
            inner: DeviceInner::Host,
        }
    }
    pub fn new(index: usize) -> Result<Self, anyhow::Error> {
        use spirv::Capability::*;
        let options = DeviceOptions {
            optimal_capabilities: vec![
                Int8,
                Int16,
                Int64,
                Float64,
                StorageBuffer8BitAccess,
                StorageBuffer16BitAccess,
            ],
        };
        #[cfg(test)] {
            use once_cell::sync::OnceCell;
            static DEVICE: OnceCell<Device> = OnceCell::new();
            if index == 0 {
                return DEVICE.get_or_try_init(|| {
                    Device::new_ext(index, &options)
                }).map(|x| x.clone());
            }
        }
        Self::new_ext(index, &options)
    }
    #[cfg_attr(not(feature = "device"), allow(unused_variables))]
    fn new_ext(index: usize, options: &DeviceOptions) -> Result<Self, anyhow::Error> {
        #[cfg(feature = "device")]
        {
            return Ok(Self {
                inner: DeviceInner::Device(DeviceBase::new(index, options)?),
            });
        }
        #[cfg(not(feature = "device"))]
        {
            Err(DeviceUnavailable::new().into())
        }
    }
    pub(crate) fn kind(&self) -> DeviceKind {
        self.inner.kind()
    }
    pub(crate) fn is_host(&self) -> bool {
        self.inner.is_host()
    }
    pub(crate) fn is_device(&self) -> bool {
        !self.is_host()
    }
    #[cfg(feature = "device")]
    pub(crate) fn as_device(&self) -> Option<&DeviceBase> {
        match &self.inner {
            DeviceInner::Host => None,
            DeviceInner::Device(device) => Some(device),
        }
    }
}

impl Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.inner {
            #[cfg(feature = "device")]
            DeviceInner::Device(device) => {
                write!(f, "Device({})", device.index())
            }
            DeviceInner::Host => write!(f, "Host"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "device")]
    #[test]
    fn device_new() -> Result<()> {
        let device = Device::new(0)?;
        Ok(())
    }
}
