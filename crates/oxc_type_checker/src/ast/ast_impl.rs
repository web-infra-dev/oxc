use super::{Type, UnionType};

// Conventions:
// - as_<type> returns Option<TypeVariant> if it contains that variant
// -into_<type> returns TypeVariant, panicking if this Type is not a TypeVariatn
impl<'a> Type<'a> {
    pub fn as_union(&self) -> Option<&UnionType<'a>> {
        match self {
            Self::Union(union) => Some(union),
            _ => None,
        }
    }
    /// # Panics
    /// If this [Type] is not a [UnionType]
    pub fn into_union(&self) -> &UnionType<'a> {
        self.as_union().expect("Type has TypeFlags::Union but is not a UnionType")
    }
}
