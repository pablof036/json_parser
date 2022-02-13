use crate::lib::case::CaseType;

pub const RUST_DEFINITION: TransformConfig = TransformConfig {
    type_definition: "struct {object_name} {",
    field_definition: "\t{field_name}: {field_type},",
    array_definition: "Vec<{field_type}>",
    block_end: "}",
    int_type: "i32",
    float_type: "f32",
    bool_type: "bool",
    string_type: "String",
    constructor: None,
    case_type: CaseType::SnakeCase,
    object_case_type: CaseType::UpperCamelCase,
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
    case_type: CaseType::CamelCase,
    object_case_type: CaseType::UpperCamelCase,
    constructor: Some(
        ConstructorConfig {
            definition: "\tpublic {object_name}({arguments}) {",
            argument_definition: "{type} {name}",
            separator: ", ",
            separator_at_end: false,
            field_definition: Some(ConstructorField{
                field_definition: "\t\tthis.{name} = {name};",
                end: "\t}"
            })
        }
    ),
};

pub const DART_DEFINITION: TransformConfig = TransformConfig {
    type_definition: "class {object_name} {",
    field_definition: "\tfinal {field_type}? {field_name};",
    array_definition: "List<{field_type}>",
    block_end: "}",
    int_type: "int",
    float_type: "double",
    bool_type: "bool",
    string_type: "String",
    case_type: CaseType::CamelCase,
    object_case_type: CaseType::UpperCamelCase,
    constructor: Some(
        ConstructorConfig {
        definition: "\t{object_name}({{arguments}\n\t});",
        argument_definition: "\n\t\tthis.{name}",
        separator: ", ",
        separator_at_end: true,
        field_definition: None,
    })
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
    pub constructor: Option<ConstructorConfig<'a>>,
    pub case_type: CaseType,
    pub object_case_type: CaseType,
}

pub struct ConstructorConfig<'a> {
    pub definition: &'a str,
    pub argument_definition: &'a str,
    pub separator: &'a str,
    pub separator_at_end: bool,
    pub field_definition: Option<ConstructorField<'a>>,
}

pub struct ConstructorField<'a> {
    pub field_definition: &'a str,
    pub end: &'a str,
}