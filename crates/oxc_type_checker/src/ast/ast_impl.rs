use std::ops::Deref;

use super::{FreshableType, Type, UnionType};

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

impl<T> Deref for FreshableType<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Fresh(inner, _) => inner,
            Self::Regular(inner, _) => inner,
        }
    }
}

impl<T> From<T> for FreshableType<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Fresh(value, None)
    }
}
impl<T: Default> Default for FreshableType<T> {
    #[inline]
    fn default() -> Self {
        Self::Fresh(T::default(), None)
    }
}

// impl<T> FreshableType<T> {
//     #[inline]
//     pub const fn is_fresh(&self) -> bool {
//         matches!(self, Self::Fresh(..))
//     }

//     #[inline]
//     pub const fn is_regular(&self) -> bool {
//         matches!(self, Self::Regular(..))
//     }
// }
