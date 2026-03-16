pub mod parser;

use num_enum::TryFromPrimitive;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum PacketType {
    // TODO: Fill these in from https://github.com/meshcore-dev/MeshCore/blob/main/examples/companion_radio/MyMesh.cpp#L67
    Ok = 0,
    Error = 1,
    SelfInfo = 5,
}
