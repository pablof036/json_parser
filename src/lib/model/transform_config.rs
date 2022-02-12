pub const RUST_DEFINITION: TransformConfig = TransformConfig {
    type_definition: "struct {object_name} {",
    field_definition: "\t{field_name}: {field_type},",
    array_definition: "Vec<{field_type}>",
    block_end: "}",
    int_type: "i32",
    float_type: "f32",
    bool_type: "bool",
    string_type: "String",
    single_file: true
};

pub const JAVA_DEFINITION: TransformConfig = TransformConfig {
    type_definition: "class {object_name} {",
    field_definition: "\tprivate final {field_type} {field_name};",
    array_definition: "{field_type}[]",
    block_end: "}",
    int_type: "int",
    float_type: "double",
    bool_type: "boolean",
    string_type: "String",
    single_file: true
};

pub struct TransformConfig<'a> {
    pub type_definition: &'a str,
    pub field_definition: &'a str,
    pub array_definition: &'a str,
    pub block_end: &'a str,
    pub int_type: &'a str,
    pub float_type: &'a str,
    pub bool_type: &'a str,
    pub string_type: &'a str,
    pub single_file: bool,
}