pub const RUST_DEFINITION: TransformConfig = TransformConfig {
    type_definition: "struct {0} {\n{1}\n}",
    field_definition: "\t{0}: {1}",
    array_definition: "Vec<{0}",
    int_type: "i32",
    float_type: "f32",
    bool_type: "bool",
    string_type: "String",
    single_file: true
};

pub struct TransformConfig<'a> {
    pub type_definition: &'a str,
    pub field_definition: &'a str,
    pub array_definition: &'a str,
    pub int_type: &'a str,
    pub float_type: &'a str,
    pub bool_type: &'a str,
    pub string_type: &'a str,
    pub single_file: bool,
}