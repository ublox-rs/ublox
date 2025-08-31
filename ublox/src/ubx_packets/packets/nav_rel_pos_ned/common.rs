#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NavRelPosNedFlags(u32);

impl NavRelPosNedFlags {
    pub fn gnss_fix_ok(&self) -> bool {
        self.0 & 0x1 != 0
    }

    pub fn diff_soln(&self) -> bool {
        (self.0 >> 1) & 0x1 != 0
    }

    pub fn rel_pos_valid(&self) -> bool {
        (self.0 >> 2) & 0x1 != 0
    }

    pub fn carr_soln(&self) -> CarrierPhaseRangeSolutionStatus {
        match (self.0 >> 3) & 0x3 {
            0 => CarrierPhaseRangeSolutionStatus::NoSolution,
            1 => CarrierPhaseRangeSolutionStatus::SolutionWithFloatingAmbiguities,
            2 => CarrierPhaseRangeSolutionStatus::SolutionWithFixedAmbiguities,
            unknown => panic!("Unexpected 2-bit bitfield value {unknown}!"),
        }
    }

    pub fn is_moving(&self) -> bool {
        (self.0 >> 5) & 0x1 != 0
    }

    pub fn ref_pos_miss(&self) -> bool {
        (self.0 >> 6) & 0x1 != 0
    }

    pub fn ref_obs_miss(&self) -> bool {
        (self.0 >> 7) & 0x1 != 0
    }

    pub const fn from(x: u32) -> Self {
        Self(x)
    }
}

impl core::fmt::Debug for NavRelPosNedFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut dbg_struct = f.debug_struct("NavRelPosNedFlags");
        dbg_struct
            .field("gnss_fix_ok", &self.gnss_fix_ok())
            .field("diff_soln", &self.diff_soln())
            .field("rel_pos_valid", &self.rel_pos_valid())
            .field("carr_soln", &self.carr_soln())
            .field("is_moving", &self.is_moving())
            .field("ref_pos_miss", &self.ref_pos_miss())
            .field("ref_obs_miss", &self.ref_obs_miss());

        dbg_struct.finish()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CarrierPhaseRangeSolutionStatus {
    /// No carrier phase range solution
    NoSolution,
    /// Carrier phase range solution with floating ambiguities
    SolutionWithFloatingAmbiguities,
    /// Carrier phase range solution with fixed ambiguities
    SolutionWithFixedAmbiguities,
}
