// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_gen/src/derives/estree.rs`

#![allow(unused_imports, unused_mut, clippy::match_same_arms)]

use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::source_type::*;

use crate::span::types::*;

impl Serialize for Span {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Span")?;
        map.serialize_entry("start", &self.start)?;
        map.serialize_entry("end", &self.end)?;
        map.serialize_entry("align", &self._align)?;
        map.end()
    }
}

impl Serialize for SourceType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SourceType")?;
        map.serialize_entry("language", &self.language)?;
        map.serialize_entry("moduleKind", &self.module_kind)?;
        map.serialize_entry("variant", &self.variant)?;
        map.end()
    }
}

impl Serialize for Language {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Language::JavaScript(x) => Serialize::serialize(x, serializer),
            Language::TypeScript(x) => Serialize::serialize(x, serializer),
            Language::TypeScriptDefinition(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for ModuleKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ModuleKind::Script(x) => Serialize::serialize(x, serializer),
            ModuleKind::Module(x) => Serialize::serialize(x, serializer),
            ModuleKind::Unambiguous(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for LanguageVariant {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LanguageVariant::Standard(x) => Serialize::serialize(x, serializer),
            LanguageVariant::Jsx(x) => Serialize::serialize(x, serializer),
        }
    }
}
