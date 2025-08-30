#[cfg(feature = "ubx_proto14")]
pub mod proto14;

#[cfg(feature = "ubx_proto14")]
pub use proto14::*;

#[cfg(any(
    feature = "ubx_proto23",
    feature = "ubx_proto27",
    feature = "ubx_proto31"
))]
pub mod proto23_27_31;

#[cfg(any(
    feature = "ubx_proto23",
    feature = "ubx_proto27",
    feature = "ubx_proto31"
))]
pub use proto23_27_31::*;
