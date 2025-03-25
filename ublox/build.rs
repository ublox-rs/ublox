fn main() {
    #[cfg(all(feature = "ubx_proto23", feature = "ubx_proto27",))]
    compile_error!(
        r#"The "ubx_proto23" and "ubx_proto27" features are mutually exclusive and cannot be activated at the same time. Please disable one or the other."#
    );

    #[cfg(all(feature = "ubx_proto27", feature = "ubx_proto31",))]
    compile_error!(
        r#"The "ubx_proto27" and "ubx_proto31" features are mutually exclusive and cannot be activated at the same time. Please disable one or the other."#
    );

    #[cfg(all(feature = "ubx_proto23", feature = "ubx_proto31",))]
    compile_error!(
        r#"The "ubx_proto23" and "ubx_proto31" features are mutually exclusive and cannot be activated at the same time. Please disable one or the other."#
    );

    #[cfg(all(
        not(feature = "ubx_proto23"),
        not(feature = "ubx_proto27"),
        not(feature = "ubx_proto31")
    ))]
    compile_error!(
        r#"At least one feature "ubx_protoXX" versions needs to be selected. Please select only one."#
    );
}
