// Known byte offsets in the Ethos binary model format.
//
// Each constant is added as reverse engineering confirms it.
// Until confirmed, bytes are captured as opaque padding in schema.rs.
//
// Convention:
// - Offsets are from the start of file
// - Add source evidence as comments (e.g. "confirmed: model_A vs model_B diff at this offset")

// TODO: confirm magic bytes by inspecting a real .bin file
// pub const FILE_MAGIC: [u8; 4] = [...];

// Placeholder — all offsets are unconfirmed until real .bin files are analysed.
// Example of a confirmed entry:
// pub const OFFSET_MODEL_NAME: u64 = 0x04;   // confirmed
// pub const LEN_MODEL_NAME: usize  = 16;      // null-padded to 16 bytes
