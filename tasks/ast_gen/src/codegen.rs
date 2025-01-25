use bitflags::bitflags;
use rustc_hash::FxHashMap;

use crate::{
    derives::Derive,
    schema::{EnumDef, StructDef, TypeDef},
    Output, Result, Schema, DERIVES, GENERATORS,
};

pub type DeriveId = usize;
pub type GeneratorId = usize;

pub struct Codegen {
    /// Mapping from derive name to `DeriveId`
    derive_name_to_id: FxHashMap<&'static str, DeriveId>,
    /// Mapping from attr to ID of derive/generator which uses the attr,
    /// and positions attr can appear in
    pub attr_processors: FxHashMap<&'static str, (AttrProcessor, AttrPositions)>,
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
    /// Attribute positions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AttrPositions: u8 {
        const Struct = 1 << 0;
        const Enum = 1 << 1;
        const StructField = 1 << 2;
        const EnumVariant = 1 << 3;

        const Type = Self::Struct.bits() | Self::Enum.bits();
        const TypeOrStructField = Self::Type.bits() | Self::StructField.bits();
        const StructFieldOrEnumVariant = Self::StructField.bits() | Self::EnumVariant.bits();
        const Any = Self::Type.bits() | Self::StructFieldOrEnumVariant.bits();
    }
}

/// Attribute location.
pub enum AttrLocation<'s> {
    Struct(&'s mut StructDef),
    Enum(&'s mut EnumDef),
    /// Struct def and field index
    StructField(&'s mut StructDef, usize),
    /// Enum def and variant index
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

    /// Convert [`AttrLocation`] to [`StructDef`], if it's an `AttrLocation::Struct`.
    ///
    /// # Panics
    /// Panics if it's not an `AttrLocation::Struct`.
    #[expect(dead_code)]
    pub fn into_struct(self) -> &'s mut StructDef {
        match self {
            Self::Struct(struct_def) => struct_def,
            _ => panic!("AttrLocation is not a struct"),
        }
    }

    /// Convert [`AttrLocation`] to [`EnumDef`], if it's an `AttrLocation::Enum`.
    ///
    /// # Panics
    /// Panics if it's not an `AttrLocation::Enum`.
    #[expect(dead_code)]
    pub fn into_enum(self) -> &'s mut EnumDef {
        match self {
            Self::Enum(enum_def) => enum_def,
            _ => panic!("AttrLocation is not a enum"),
        }
    }

    /// Convert [`AttrLocation`] to [`StructDef`] and field index, if it's an `AttrLocation::StructField`.
    ///
    /// # Panics
    /// Panics if it's not an `AttrLocation::StructField`.
    pub fn into_struct_field(self) -> (&'s mut StructDef, usize) {
        match self {
            Self::StructField(struct_def, field_index) => (struct_def, field_index),
            _ => panic!("AttrLocation is not a struct field"),
        }
    }

    /// Convert [`AttrLocation`] to [`EnumDef`] and variant index, if it's an `AttrLocation::EnumVariant`.
    ///
    /// # Panics
    /// Panics if it's not an `AttrLocation::EnumVariant`.
    #[expect(dead_code)]
    pub fn into_enum_variant(self) -> (&'s mut EnumDef, usize) {
        match self {
            Self::EnumVariant(enum_def, variant_index) => (enum_def, variant_index),
            _ => panic!("AttrLocation is not an enum variant"),
        }
    }
}

pub trait Runner {
    #[expect(dead_code)]
    fn verb(&self) -> &'static str;

    fn name(&self) -> &'static str;

    fn file_path(&self) -> &'static str;

    fn run(&self, ctx: &Schema, codegen: &Codegen) -> Result<Vec<Output>>;
}
