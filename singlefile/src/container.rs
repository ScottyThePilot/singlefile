//! Container constructs providing managed access to a file.

pub mod owned;

#[cfg_attr(docsrs, doc(cfg(feature = "shared")))]
#[cfg(feature = "shared")]
pub mod shared;

#[cfg_attr(docsrs, doc(cfg(feature = "shared-async")))]
#[cfg(feature = "shared-async")]
pub mod shared_async;



pub use self::owned::Container;

#[cfg_attr(docsrs, doc(cfg(feature = "shared")))]
#[cfg(feature = "shared")]
pub use self::shared::ContainerShared;

#[cfg_attr(docsrs, doc(cfg(feature = "shared-async")))]
#[cfg(feature = "shared-async")]
pub use self::shared_async::ContainerSharedAsync;
