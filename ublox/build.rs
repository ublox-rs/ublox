fn main() {
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
