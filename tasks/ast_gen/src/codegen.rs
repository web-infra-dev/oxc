use bitflags::bitflags;
use rustc_hash::FxHashMap;

use crate::{
    schema::{EnumDef, StructDef, TypeDef},
    Derive, Output, Result, Schema, DERIVES, GENERATORS,
};

pub type DeriveId = usize;
pub type GeneratorId = usize;

pub struct Codegen {
    /// Mapping from derive name to `DeriveId`
    derive_name_to_id: FxHashMap<&'static str, DeriveId>,
    /// Mapping from attribute name to ID of derive/generator which uses the attr,
    /// and legal positions for the attribute
    attr_processors: FxHashMap<&'static str, (AttrProcessor, AttrPositions)>,
}

impl Codegen {
    pub fn new() -> Self {
        let mut derive_name_to_id = FxHashMap::default();

        let mut attr_processors = FxHashMap::default();

        for (id, &derive) in DERIVES.iter().enumerate() {
            derive_name_to_id.insert(derive.trait_name(), id);

            let processor = AttrProcessor::Derive(id);
            for &(name, positions) in derive.attrs() {
                let existing = attr_processors.insert(name, (processor, positions));
                if let Some((existing_processor, _)) = existing {
                    panic!(
                        "Two derives expect same attr `#[{name:?}]`: {} and {}",
                        existing_processor.name(),
                        processor.name()
                    );
                }
            }
        }

        for (id, &generator) in GENERATORS.iter().enumerate() {
            let processor = AttrProcessor::Generator(id);

            for &(name, positions) in generator.attrs() {
                let existing_processor = attr_processors.insert(name, (processor, positions));
                if let Some((existing_processor, _)) = existing_processor {
                    panic!(
                        "Two derives/generators expect same attr {name:?}: {} and {}",
                        existing_processor.name(),
                        processor.name()
                    );
                }
            }
        }

        Self { derive_name_to_id, attr_processors }
    }

    #[expect(clippy::unused_self)]
    pub fn get_derive(&self, id: DeriveId) -> &'static dyn Derive {
        DERIVES[id]
    }

    pub fn get_derive_id_by_name(&self, name: &str) -> DeriveId {
        self.derive_name_to_id.get(name).copied().unwrap_or_else(|| {
            panic!("Unknown derive trait {name:?}");
        })
    }

    #[expect(dead_code)]
    pub fn get_derive_by_name(&self, name: &str) -> &dyn Derive {
        self.get_derive(self.get_derive_id_by_name(name))
    }

    /// Get processor (derive or generator) for an attribute, and legal positions for the attribute
    pub fn attr_processor(&self, attr_name: &str) -> Option<(AttrProcessor, AttrPositions)> {
        self.attr_processors.get(attr_name).copied()
    }
}

/// Processor of an attribute - either a derive or a generator.
#[derive(Clone, Copy, Debug)]
pub enum AttrProcessor {
    Derive(DeriveId),
    Generator(GeneratorId),
}

impl AttrProcessor {
    pub fn name(self) -> &'static str {
        match self {
            Self::Derive(id) => DERIVES[id].trait_name(),
            Self::Generator(_id) => "Unknown generator", // TODO
        }
    }
}

bitflags! {
    /// Positions in which an attribute is legal.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AttrPositions: u8 {
        const Struct = 1 << 0;
        const Enum = 1 << 1;
        const StructField = 1 << 2;
        const EnumVariant = 1 << 3;
    }
}

/// Macro to combine multiple `AttrPositions` as a const.
///
/// `attr_positions!(Struct | Enum)` is equivalent to `AttrPositions::Struct | AttrPositions::Enum`,
/// except it evaluates in const context.
///
/// Useful for `Derive::attrs` and `Generator::attrs` methods, where a const is required.
macro_rules! attr_positions {
    ($($positions:ident)|+) => {
        const {
            use $crate::codegen::AttrPositions;
            AttrPositions::empty() $(.union(AttrPositions::$positions))+
        }
    }
}
pub(crate) use attr_positions;

/// Attribute location.
pub enum AttrLocation<'s> {
    #[expect(dead_code)]
    Struct(&'s mut StructDef),
    #[expect(dead_code)]
    Enum(&'s mut EnumDef),
    /// Struct def and field index
    StructField(&'s mut StructDef, usize),
    /// Enum def and variant index
    #[expect(dead_code)]
    EnumVariant(&'s mut EnumDef, usize),
}

impl<'s> AttrLocation<'s> {
    pub fn from_type_def(type_def: &'s mut TypeDef) -> Self {
        match type_def {
            TypeDef::Struct(struct_def) => Self::Struct(struct_def),
            TypeDef::Enum(enum_def) => Self::Enum(enum_def),
            _ => panic!("TypeDef is not a struct or enum"),
        }
    }
}

/// Runner trait.
///
/// This is the super-trait of [`Derive`] and [`Generator`].
///
/// [`Generator`]: crate::Generator
pub trait Runner {
    #[expect(dead_code)]
    fn verb(&self) -> &'static str;

    fn name(&self) -> &'static str;

    fn file_path(&self) -> &'static str;

    fn run(&self, ctx: &Schema, codegen: &Codegen) -> Result<Vec<Output>>;
}
