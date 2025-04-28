fn main() {
    #[cfg(any(
        all(feature = "ubx_proto14", feature = "ubx_proto23"),
        all(feature = "ubx_proto14", feature = "ubx_proto27"),
        all(feature = "ubx_proto14", feature = "ubx_proto31"),
        all(feature = "ubx_proto23", feature = "ubx_proto27"),
        all(feature = "ubx_proto23", feature = "ubx_proto31"),
        all(feature = "ubx_proto27", feature = "ubx_proto31")
    ))]
    compile_error!(
        "The 'ubx_protoXX' features are mutually exclusive and cannot be activated at the same time. Please activate only one at a time."
    );

    #[cfg(all(
        not(feature = "ubx_proto14"),
        not(feature = "ubx_proto23"),
        not(feature = "ubx_proto27"),
        not(feature = "ubx_proto31")
    ))]
    compile_error!(
        "At least one feature 'ubx_protoXX' versions needs to be selected. Please select only one."
    );
}
