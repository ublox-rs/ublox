pub struct CfgValGetBuilder<'a> {
    #[doc = " Message version"]
    pub version: u8,
    #[doc = " The layers from which the configuration items should be retrieved"]
    pub layers: CfgLayerGet,
    #[doc = ""]
    pub position: u16,
    #[doc = ""]
    pub cfg_data: CfgValIter<'a>,
}
impl<'a> CfgValGetBuilder<'a> {
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn into_packet_vec(self) -> Vec<u8> {
        let mut vec = Vec::new();
        self.extend_to(&mut vec);
        vec
    }
    #[inline]
    pub fn extend_to<T>(self, out: &mut T)
    where
        T: core::iter::Extend<u8> + core::ops::DerefMut<Target = [u8]>,
    {
        let mut len_bytes = 0;
        let header = [
            SYNC_CHAR_1,
            SYNC_CHAR_2,
            CfgValGet::CLASS,
            CfgValGet::ID,
            0,
            0,
        ];
        out.extend(header);
        let bytes = self.version.to_le_bytes();
        len_bytes += bytes.len();
        out.extend(bytes);
        let bytes = <CfgLayerGet>::into_raw(self.layers).to_le_bytes();
        len_bytes += bytes.len();
        out.extend(bytes);
        let bytes = self.position.to_le_bytes();
        len_bytes += bytes.len();
        out.extend(bytes);
        for f in self.cfg_data {
            len_bytes += f.extend_to(out);
        }
        let len_bytes = len_bytes.to_le_bytes();
        out[4] = len_bytes[0];
        out[5] = len_bytes[1];
        let (ck_a, ck_b) = ubx_checksum(&out[2..]);
        out.extend(core::iter::once(ck_a));
        out.extend(core::iter::once(ck_b));
    }
}
