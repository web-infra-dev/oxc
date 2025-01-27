/// Configuration for ESTree transformation on structs.
#[derive(Default, Debug)]
pub struct ESTreeStruct {
    /// `true` if properties of this struct should always
    #[expect(dead_code)]
    pub always_flatten: bool,
}
