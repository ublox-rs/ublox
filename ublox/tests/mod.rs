mod generator_test;
mod parser_binary_dump_test;
mod parser_tests;
mod rxm_sfrbx;

// All RTCM and RTCM + UBX tests require all these features
// to forge required packets easily.
#[cfg(all(feature = "std", feature = "rtcm"))]
mod ubx_rtcm;
