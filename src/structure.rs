use num_derive::FromPrimitive;

#[derive(PartialEq, Eq, FromPrimitive)]
pub enum StructureToken {
    BeginNode = 0x1,
    EndNode = 0x2,
    Property = 0x3,
    Nop = 0x4,
    End = 0x9,
}