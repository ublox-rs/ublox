pub mod common;

#[cfg(any(feature = "ubx_proto14", feature = "ubx_proto23"))]
pub mod proto14_23;

#[cfg(any(feature = "ubx_proto27", feature = "ubx_proto31"))]
pub mod proto27_31;
