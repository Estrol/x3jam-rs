#[derive(Debug, Clone, Copy, encoder::StructSerializer, encoder::StructDeserializer)]
pub struct SkillId(pub u32);
