/// Ethos binary model schema.
///
/// The format is undocumented and proprietary. Fields are added incrementally
/// as reverse engineering confirms offsets and types.
///
/// Pattern: unknown byte regions are captured as Vec<u8> padding blocks.
/// When a field is confirmed, split the relevant padding block and add the
/// typed field above it. The total file size must remain identical.
use binrw::{BinRead, BinResult, BinWrite, Endian};

/// Entire Ethos model file.
///
/// Until reverse engineering confirms the structure, the entire file content
/// is stored as raw bytes. Sub-fields will be extracted as they are confirmed.
pub struct EthosModel {
    pub raw: Vec<u8>,
}

impl std::fmt::Debug for EthosModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EthosModel({} bytes)", self.raw.len())
    }
}

impl BinRead for EthosModel {
    type Args<'a> = ();

    fn read_options<R: binrw::io::Read + binrw::io::Seek>(
        reader: &mut R,
        _endian: Endian,
        _args: (),
    ) -> BinResult<Self> {
        let mut raw = Vec::new();
        std::io::Read::read_to_end(reader, &mut raw)?;
        Ok(EthosModel { raw })
    }
}

impl BinWrite for EthosModel {
    type Args<'a> = ();

    fn write_options<W: binrw::io::Write + binrw::io::Seek>(
        &self,
        writer: &mut W,
        _endian: Endian,
        _args: (),
    ) -> BinResult<()> {
        std::io::Write::write_all(writer, &self.raw)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinReaderExt;
    use std::io::Cursor;

    #[test]
    fn parse_empty_does_not_panic() {
        let data: &[u8] = &[];
        let mut cursor = Cursor::new(data);
        let model: EthosModel = cursor.read_le().unwrap();
        assert!(model.raw.is_empty());
    }

    #[test]
    fn parse_arbitrary_bytes() {
        let data: &[u8] = &[0x01, 0x02, 0x03, 0x04];
        let mut cursor = Cursor::new(data);
        let model: EthosModel = cursor.read_le().unwrap();
        assert_eq!(model.raw, data);
    }
}
