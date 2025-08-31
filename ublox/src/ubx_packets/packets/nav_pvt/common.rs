use bitflags::bitflags;
use ublox_derive::ubx_extend_bitflags;

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Fix status flags for `NavPvt`
    #[derive(Debug)]
    pub struct NavPvtFlags: u8 {
        /// 1 = Position and velocity valid and within DOP and accuracy masks
        const GPS_FIX_OK = 1;
        /// 1 = Differential corrections were applied; DGPS used
        const DIFF_SOLN = 2;
        /// 1 = Heading of vehicle is valid, only set if the receiver is in sensor fusion mode
        const HEAD_VEH_VALID = 0x20;
        /// 1 = Carrier phase range solution with floating ambiguities (not supported for protocol versions less than 20.00)
        const CARR_SOLN_FLOAT = 0x40;
        /// 1 = Carrier phase range solution with fixed ambiguities (not supported for protocol versions less than 20.00)
        const CARR_SOLN_FIXED = 0x80;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Additional flags for `NavPvt`
    #[derive(Debug)]
    pub struct NavPvtFlags2: u8 {
        /// 1 = information about UTC Date and Time of Day validity confirmation
        /// is available. This flag is only supported in Protocol Versions
        /// 19.00, 19.10, 20.10, 20.20, 20.30, 22.00, 23.00, 23.01, 27 and 28.
        const CONFIRMED_AVAI = 0x20;
        /// 1 = UTC Date validity could be confirmed
        /// (confirmed by using an additional independent source)
        const CONFIRMED_DATE = 0x40;
        /// 1 = UTC Time of Day could be confirmed
        /// (confirmed by using an additional independent source)
        const CONFIRMED_TIME = 0x80;
    }
}

#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    /// Validity flags
    #[derive(Debug)]
    pub struct NavPvtValidFlags: u8 {
        /// 1 = valid UTC Date
        const VALID_DATE = 0x01;
        /// 1 = valid UTC time of day
        const VALID_TIME = 0x02;
        /// 1 = UTC time of day has been fully resolved (no seconds uncertainty).
        /// Cannot be used to check if time is completely solved.
        const FULLY_RESOLVED = 0x04;
        /// 1 = valid magnetic declination
        const VALID_MAG = 0x08;
    }
}
