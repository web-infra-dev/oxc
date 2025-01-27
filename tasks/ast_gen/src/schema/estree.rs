/// Configuration for ESTree transformation on struct.
#[derive(Default, Debug)]
pub struct ESTreeStruct {
    /// `true` if properties of this struct should always
    #[expect(dead_code)]
    pub always_flatten: bool,
}

/// Configuration for ESTree transformation on struct field.
#[derive(Default, Debug)]
pub struct ESTreeStructField {
    /// `true` this property should be skipped in serialization
    pub skip: bool,
}
