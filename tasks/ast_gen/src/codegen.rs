use rustc_hash::FxHashMap;

use crate::{derives::Derive, Output, Result, Schema, DERIVES, GENERATORS};

pub type DeriveId = usize;
pub type GeneratorId = usize;

// TODO: Move `derive_name_to_id` into `Schema`?

pub struct Codegen {
    /// Mapping from derive name to `DeriveId`
    derive_name_to_id: FxHashMap<&'static str, DeriveId>,
    /// Mapping from type attr to ID of derive/generator which uses the attr
    #[expect(dead_code)]
    pub type_attrs: FxHashMap<&'static str, AttrProcessor>,
    /// Mapping from struct field attr to ID of derive/generator which uses the attr
    pub field_attrs: FxHashMap<&'static str, AttrProcessor>,
    /// Mapping from enum variant attr to ID of derive/generator which uses the attr
    #[expect(dead_code)]
    pub variant_attrs: FxHashMap<&'static str, AttrProcessor>,
}

impl Codegen {
    pub fn new() -> Self {
        let mut derive_name_to_id = FxHashMap::default();

        let mut type_attrs = FxHashMap::default();
        let mut field_attrs = FxHashMap::default();
        let mut variant_attrs = FxHashMap::default();

        for (id, &derive) in DERIVES.iter().enumerate() {
            derive_name_to_id.insert(derive.trait_name(), id);

            let processor = AttrProcessor::Derive(id);
            for &type_attr in derive.type_attrs() {
                let existing_processor = type_attrs.insert(type_attr, processor);
                if let Some(existing_processor) = existing_processor {
                    panic!(
                        "Two derives expect same type attr {type_attr:?}: {} and {}",
                        existing_processor.name(),
                        processor.name()
                    );
                }
            }

            for &field_attr in derive.field_attrs() {
                let existing_processor = field_attrs.insert(field_attr, processor);
                if let Some(existing_processor) = existing_processor {
                    panic!(
                        "Two derives expect same struct field attr {field_attr:?}: {} and {}",
                        existing_processor.name(),
                        processor.name()
                    );
                }
            }

            for &variant_attr in derive.variant_attrs() {
                let existing_processor = variant_attrs.insert(variant_attr, processor);
                if let Some(existing_processor) = existing_processor {
                    panic!(
                        "Two derives expect same enum variant attr {variant_attr:?}: {} and {}",
                        existing_processor.name(),
                        processor.name()
                    );
                }
            }
        }

        for (id, &generator) in GENERATORS.iter().enumerate() {
            let processor = AttrProcessor::Generator(id);

            for &type_attr in generator.type_attrs() {
                let existing_processor = type_attrs.insert(type_attr, processor);
                if let Some(existing_processor) = existing_processor {
                    panic!(
                        "Two derives/generators expect same type attr {type_attr:?}: {} and {}",
                        existing_processor.name(),
                        processor.name()
                    );
                }
            }

            for &field_attr in generator.field_attrs() {
                let existing_processor = field_attrs.insert(field_attr, processor);
                if let Some(existing_processor) = existing_processor {
                    panic!(
                        "Two derives/generators expect same struct field attr {field_attr:?}: {} and {}",
                        existing_processor.name(),
                        processor.name()
                    );
                }
            }

            for &variant_attr in generator.variant_attrs() {
                let existing_processor = variant_attrs.insert(variant_attr, processor);
                if let Some(existing_processor) = existing_processor {
                    panic!(
                        "Two derives/generators expect same enum variant attr {variant_attr:?}: {} and {}",
                        existing_processor.name(),
                        processor.name()
                    );
                }
            }
        }

        Self { derive_name_to_id, type_attrs, field_attrs, variant_attrs }
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

pub trait Runner {
    #[expect(dead_code)]
    fn verb(&self) -> &'static str;

    fn name(&self) -> &'static str;

    fn file_path(&self) -> &'static str;

    fn run(&self, ctx: &Schema, codegen: &Codegen) -> Result<Vec<Output>>;
}
