// Functions which are no-ops or just delegate to other methods are marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{marker::PhantomData, num, ptr};

pub use serde::Serializer as SerdeSerializer;
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize as SerdeSerialize,
};
use serde_json::Serializer as SerdeJsonSerializer;

use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};

/// Serialize AST node to JSON.
///
/// If `include_ts_fields` is `true`, will serialize the whole AST.
/// If `false`, TypeScript fields will be skipped.
//
// Cannot actually panic because `serde_json` is infallible.
#[expect(clippy::missing_panics_doc)]
pub fn to_json<T: ESTree>(value: &T, include_ts_fields: bool) -> String {
    // Create `serde_json` serializer
    let buffer = vec![];
    let mut serde_serializer = SerdeJsonSerializer::new(buffer);

    // Serialize
    serialize(value, &mut serde_serializer, include_ts_fields).unwrap();

    // Convert buffer to string
    let buffer = serde_serializer.into_inner();
    // SAFETY: `serde_json` does not emit invalid UTF-8.
    // `serde_json::to_string` contains this same code.
    unsafe { String::from_utf8_unchecked(buffer) }
}

/// Serialize AST node with provided [`serde::Serializer`] serializer.
///
/// If `include_ts_fields` is `true`, will serialize the whole AST.
/// If `false`, TypeScript fields will be skipped.
///
/// # Errors
/// Returns `Err` if underlying `serde` serializer throws an error.
pub fn serialize<T: ESTree, S: SerdeSerializer>(
    value: &T,
    serde_serializer: S,
    include_ts_fields: bool,
) -> Result<S::Ok, S::Error> {
    // Wrap node in `NodeWrapper` with appropriate `ESTreeConfig`, and serialize it with `serde`
    if include_ts_fields {
        let wrapped = NodeWrapper::<SerConfigTS, T>::wrap(value);
        wrapped.serialize(serde_serializer)
    } else {
        let wrapped = NodeWrapper::<SerConfigJS, T>::wrap(value);
        wrapped.serialize(serde_serializer)
    }
}

/// Trait for AST nodes which can be serialized to ESTree.
pub trait ESTree {
    /// Serialize `self` in ESTree format.
    ///
    /// # Errors
    /// Returns `Err` if underlying `serde` serializer throws an error.
    fn serialize<C: SerConfig, S: SerdeSerializer>(
        &self,
        serializer: ESTreeSerializer<C, S>,
    ) -> Result<S::Ok, S::Error>;
}

/// Configs for AST serialization.
pub trait SerConfig {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool;
}

/// Config for serializing AST without TypeScript fields
struct SerConfigJS;

impl SerConfig for SerConfigJS {
    const INCLUDE_TS_FIELDS: bool = false;
}

/// Config for serializing AST with TypeScript fields
struct SerConfigTS;

impl SerConfig for SerConfigTS {
    const INCLUDE_TS_FIELDS: bool = true;
}

/// Serializer for ESTree.
///
/// Just a wrapper around a [`serde::Serializer`], with zero-sized [`SerConfig`]
/// to signal whether to serialize TypeScript fields or not.
#[repr(transparent)]
pub struct ESTreeSerializer<C: SerConfig, S: SerdeSerializer> {
    serde_serializer: S,
    config: PhantomData<C>,
}

impl<C: SerConfig, S: SerdeSerializer> ESTreeSerializer<C, S> {
    /// Create new [`ESTreeSerializer`] from a [`serde::Serializer`].
    // `#[inline(always)]` because it's a no-op
    #[inline(always)]
    fn new(serde_serializer: S) -> Self {
        Self { serde_serializer, config: PhantomData }
    }

    /// Serialize a struct.
    ///
    /// # Example
    /// ```
    /// struct Foo<'a> {
    ///     exprs: Vec<'a, Expression<'a>>,
    ///     type_annotations: Vec<'a, TypeAnnotation<'a>>,
    /// }
    ///
    /// impl ESTree for Foo {
    ///     fn serialize<C: SerConfig, S: SerdeSerializer>(
    ///         &self,
    ///         serializer: ESTreeSerializer<C, S>,
    ///     ) -> Result<S::Ok, S::Error> {
    ///         let mut ser = serializer.serialize_struct()?;
    ///         ser.serialize_field("exprs", &self.exprs)?;
    ///         ser.serialize_ts_field("type_annotations", &self.type_annotations)?;
    ///         ser.end()
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns `Err` if underlying `serde` serializer throws an error.
    //
    // `#[inline(always)]` because this function does nothing but delegate to
    // `SerdeSerializer::serialize_map`. `ESTreeStructSerializer::new` is a no-op.
    #[inline(always)]
    pub fn serialize_struct(self) -> Result<ESTreeStructSerializer<C, S>, S::Error> {
        let serde_map = S::serialize_map(self.serde_serializer, None)?;
        Ok(ESTreeStructSerializer::new(serde_map))
    }
}

/// ESTree serializer for structs.
///
/// Created by [`ESTreeSerializer::serialize_struct`].
#[repr(transparent)]
pub struct ESTreeStructSerializer<C: SerConfig, S: SerdeSerializer> {
    serde_map: S::SerializeMap,
    config: PhantomData<C>,
}

impl<C: SerConfig, S: SerdeSerializer> ESTreeStructSerializer<C, S> {
    /// Create new [`ESTreeStructSerializer`].
    //
    // `#[inline(always)]` because it's a no-op
    #[inline(always)]
    fn new(serde_map: S::SerializeMap) -> Self {
        Self { serde_map, config: PhantomData }
    }

    /// Serialize a struct field which is plain JS.
    ///
    /// # Errors
    /// Returns `Err` if underlying `serde` serializer throws an error.
    //
    // `#[inline(always)]` because this function does nothing but delegate to
    // `SerializeMap::serialize_entry`. `NodeWrapper::wrap` is a no-op.
    #[inline(always)]
    pub fn serialize_field<T: ESTree>(&mut self, key: &str, value: &T) -> Result<(), S::Error> {
        let wrapped = NodeWrapper::<C, T>::wrap(value);
        self.serde_map.serialize_entry(key, &wrapped)
    }

    /// Serialize a struct field which is TypeScript.
    ///
    /// The field will be ignored if [`SerConfig`] specifies to ignore TS fields.
    ///
    /// # Errors
    /// Returns `Err` if underlying `serde` serializer throws an error.
    //
    // `#[inline(always)]` because this function does nothing but delegate to
    // `ESTreeStructSerializer::serialize_field`, or is a no-op if `!C::INCLUDE_TS_FIELDS`.
    #[inline(always)]
    pub fn serialize_ts_field<T: ESTree>(&mut self, key: &str, value: &T) -> Result<(), S::Error> {
        if C::INCLUDE_TS_FIELDS {
            self.serialize_field(key, value)
        } else {
            Ok(())
        }
    }

    /// Finish serializing a struct.
    ///
    /// # Errors
    /// Returns `Err` if underlying `serde` serializer throws an error.
    //
    // `#[inline(always)]` because this function does nothing but delegate to `SerializeMap::end`.
    #[inline(always)]
    pub fn end(self) -> Result<S::Ok, S::Error> {
        self.serde_map.end()
    }
}

/// Wrapper around a serializable AST node.
///
/// Purpose is to "smuggle" the config through serde.
#[repr(transparent)]
struct NodeWrapper<C: SerConfig, T: ESTree> {
    value: T,
    config: PhantomData<C>,
}

impl<'v, C: SerConfig, T: ESTree> NodeWrapper<C, T> {
    /// Convert a ref to an AST node to a ref to a [`NodeWrapper`] containing that node.
    //
    // `#[inline(always)]` because it's a no-op
    #[inline(always)]
    fn wrap(value: &'v T) -> &'v NodeWrapper<C, T> {
        // SAFETY: `NodeWrapper` is `#[repr(transparent)]` so guaranteed to have same layout as the
        // value it contains. So it's OK to convert a `&T` to a `&NodeWrapper<C, T>`.
        unsafe { &*ptr::from_ref(value).cast::<NodeWrapper<C, T>>() }
    }
}

// Implement `serde::Serialize` on all `NodeWrapper`s.
//
// Those implementations delegate to `ESTree::serialize`.
impl<C: SerConfig, T: ESTree> SerdeSerialize for NodeWrapper<C, T> {
    // `#[inline(always)]` because this function does nothing but delegate to `ESTree::serialize`.
    // `ESTreeSerializer::new` is a no-op.
    #[inline(always)]
    fn serialize<S: SerdeSerializer>(&self, serde_serializer: S) -> Result<S::Ok, S::Error> {
        let estree_serializer = ESTreeSerializer::<C, S>::new(serde_serializer);
        self.value.serialize(estree_serializer)
    }
}

// Implement `ESTree` on primitive types.

macro_rules! impl_estree_primitive {
    ($ty:ty, $method:ident) => {
        impl ESTree for $ty {
            // `#[inline(always)]` because this just delegates to `SerdeSerialize` method
            #[inline(always)]
            fn serialize<C: SerConfig, S: SerdeSerializer>(
                &self,
                serializer: ESTreeSerializer<C, S>,
            ) -> Result<S::Ok, S::Error> {
                serializer.serde_serializer.$method(*self)
            }
        }
    };
}

impl_estree_primitive!(u8, serialize_u8);
impl_estree_primitive!(u16, serialize_u16);
impl_estree_primitive!(u32, serialize_u32);
impl_estree_primitive!(u64, serialize_u64);
impl_estree_primitive!(u128, serialize_u128);
impl_estree_primitive!(i8, serialize_i8);
impl_estree_primitive!(i16, serialize_i16);
impl_estree_primitive!(i32, serialize_i32);
impl_estree_primitive!(i64, serialize_i64);
impl_estree_primitive!(i128, serialize_i128);
impl_estree_primitive!(f32, serialize_f32);
impl_estree_primitive!(f64, serialize_f64);

impl_estree_primitive!(bool, serialize_bool);
impl_estree_primitive!(char, serialize_char);
impl_estree_primitive!(&str, serialize_str);
impl_estree_primitive!(&[u8], serialize_bytes);

impl ESTree for usize {
    // `#[inline(always)]` because this just delegates to `SerdeSerialize` method
    #[inline(always)]
    fn serialize<C: SerConfig, S: SerdeSerializer>(
        &self,
        serializer: ESTreeSerializer<C, S>,
    ) -> Result<S::Ok, S::Error> {
        #[expect(clippy::cast_possible_truncation)]
        if cfg!(target_pointer_width = "32") {
            serializer.serde_serializer.serialize_u32(*self as u32)
        } else {
            serializer.serde_serializer.serialize_u64(*self as u64)
        }
    }
}

impl ESTree for isize {
    // `#[inline(always)]` because this just delegates to `SerdeSerialize` method
    #[inline(always)]
    fn serialize<C: SerConfig, S: SerdeSerializer>(
        &self,
        serializer: ESTreeSerializer<C, S>,
    ) -> Result<S::Ok, S::Error> {
        #[expect(clippy::cast_possible_truncation)]
        if cfg!(target_pointer_width = "32") {
            serializer.serde_serializer.serialize_i32(*self as i32)
        } else {
            serializer.serde_serializer.serialize_i64(*self as i64)
        }
    }
}

macro_rules! impl_estree_non_zero {
    ($ty:ty, $method:ident) => {
        impl ESTree for $ty {
            // `#[inline(always)]` because this just delegates to `SerdeSerialize` method
            #[inline(always)]
            fn serialize<C: SerConfig, S: SerdeSerializer>(
                &self,
                serializer: ESTreeSerializer<C, S>,
            ) -> Result<S::Ok, S::Error> {
                serializer.serde_serializer.$method(self.get())
            }
        }
    };
}

impl_estree_non_zero!(num::NonZeroU8, serialize_u8);
impl_estree_non_zero!(num::NonZeroU16, serialize_u16);
impl_estree_non_zero!(num::NonZeroU32, serialize_u32);
impl_estree_non_zero!(num::NonZeroU64, serialize_u64);
impl_estree_non_zero!(num::NonZeroU128, serialize_u128);
impl_estree_non_zero!(num::NonZeroI8, serialize_i8);
impl_estree_non_zero!(num::NonZeroI16, serialize_i16);
impl_estree_non_zero!(num::NonZeroI32, serialize_i32);
impl_estree_non_zero!(num::NonZeroI64, serialize_i64);
impl_estree_non_zero!(num::NonZeroI128, serialize_i128);

impl ESTree for num::NonZeroUsize {
    // `#[inline(always)]` because this just delegates to `SerdeSerialize` method
    #[inline(always)]
    fn serialize<C: SerConfig, S: SerdeSerializer>(
        &self,
        serializer: ESTreeSerializer<C, S>,
    ) -> Result<S::Ok, S::Error> {
        #[expect(clippy::cast_possible_truncation)]
        if cfg!(target_pointer_width = "32") {
            serializer.serde_serializer.serialize_u32(self.get() as u32)
        } else {
            serializer.serde_serializer.serialize_u64(self.get() as u64)
        }
    }
}

impl ESTree for num::NonZeroIsize {
    // `#[inline(always)]` because this just delegates to `SerdeSerialize` method
    #[inline(always)]
    fn serialize<C: SerConfig, S: SerdeSerializer>(
        &self,
        serializer: ESTreeSerializer<C, S>,
    ) -> Result<S::Ok, S::Error> {
        #[expect(clippy::cast_possible_truncation)]
        if cfg!(target_pointer_width = "32") {
            serializer.serde_serializer.serialize_i32(self.get() as i32)
        } else {
            serializer.serde_serializer.serialize_i64(self.get() as i64)
        }
    }
}

impl ESTree for () {
    // `#[inline(always)]` because this just delegates to `SerdeSerialize::serialize_unit`
    #[inline(always)]
    fn serialize<C: SerConfig, S: SerdeSerializer>(
        &self,
        serializer: ESTreeSerializer<C, S>,
    ) -> Result<S::Ok, S::Error> {
        serializer.serde_serializer.serialize_unit()
    }
}

// Implement `ESTree` on `Option`, `Box`, and `Vec`.

impl<T: ESTree> ESTree for Option<T> {
    // `#[inline]` because this function is very small.
    #[inline]
    fn serialize<C: SerConfig, S: SerdeSerializer>(
        &self,
        serializer: ESTreeSerializer<C, S>,
    ) -> Result<S::Ok, S::Error> {
        match self {
            Some(value) => {
                let wrapped = NodeWrapper::<C, T>::wrap(value);
                serializer.serde_serializer.serialize_some(wrapped)
            }
            None => serializer.serde_serializer.serialize_none(),
        }
    }
}

impl<T: ESTree> ESTree for ArenaBox<'_, T> {
    // `#[inline(always)]` because this just delegates to `ArenaBox::serialize`.
    // `NodeWrapper::wrap_box` is a no-op.
    #[inline(always)]
    fn serialize<C: SerConfig, S: SerdeSerializer>(
        &self,
        serializer: ESTreeSerializer<C, S>,
    ) -> Result<S::Ok, S::Error> {
        let wrapped = NodeWrapper::<C, T>::wrap(self.as_ref());
        wrapped.serialize(serializer.serde_serializer)
    }
}

impl<T: ESTree> ESTree for ArenaVec<'_, T> {
    fn serialize<C: SerConfig, S: SerdeSerializer>(
        &self,
        serializer: ESTreeSerializer<C, S>,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serde_serializer.serialize_seq(Some(self.len()))?;
        for element in self {
            let wrapped = NodeWrapper::<C, T>::wrap(element);
            seq.serialize_element(wrapped)?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;

    use super::*;

    #[test]
    fn test_serialize() {
        pub struct Foo<'a> {
            is_it_really_typescript: bool,
            bar: ArenaBox<'a, Bar>,
            yes_bar: Option<ArenaBox<'a, Bar>>,
            no_bar: Option<ArenaBox<'a, Bar>>,
            bars: ArenaVec<'a, Bar>,
        }

        pub struct Bar {
            big: u64,
            type_annotations: bool,
        }

        impl ESTree for Foo<'_> {
            fn serialize<C: SerConfig, S: SerdeSerializer>(
                &self,
                serializer: ESTreeSerializer<C, S>,
            ) -> Result<S::Ok, S::Error> {
                let mut ser = serializer.serialize_struct()?;
                ser.serialize_ts_field("is_it_really_typescript", &self.is_it_really_typescript)?;
                ser.serialize_field("bar", &self.bar)?;
                ser.serialize_field("yes_bar", &self.yes_bar)?;
                ser.serialize_field("no_bar", &self.no_bar)?;
                ser.serialize_field("bars", &self.bars)?;
                ser.end()
            }
        }

        impl ESTree for Bar {
            fn serialize<C: SerConfig, S: SerdeSerializer>(
                &self,
                serializer: ESTreeSerializer<C, S>,
            ) -> Result<S::Ok, S::Error> {
                let mut ser = serializer.serialize_struct()?;
                ser.serialize_field("big", &self.big)?;
                ser.serialize_ts_field("type_annotations", &self.type_annotations)?;
                ser.end()
            }
        }

        let allocator = Allocator::new();

        let mut bars = ArenaVec::new_in(&allocator);
        bars.push(Bar { big: 1000, type_annotations: true });
        bars.push(Bar { big: 2000, type_annotations: false });

        let foo = Foo {
            is_it_really_typescript: true,
            bar: ArenaBox::new_in(Bar { big: 123, type_annotations: true }, &allocator),
            yes_bar: Some(ArenaBox::new_in(Bar { big: 456, type_annotations: false }, &allocator)),
            no_bar: None,
            bars,
        };

        let js_json = to_json(&foo, false);
        assert_eq!(
            js_json,
            r#"{"bar":{"big":123},"yes_bar":{"big":456},"no_bar":null,"bars":[{"big":1000},{"big":2000}]}"#
        );
        let ts_json = to_json(&foo, true);
        assert_eq!(
            ts_json,
            r#"{"is_it_really_typescript":true,"bar":{"big":123,"type_annotations":true},"yes_bar":{"big":456,"type_annotations":false},"no_bar":null,"bars":[{"big":1000,"type_annotations":true},{"big":2000,"type_annotations":false}]}"#
        );
    }
}
