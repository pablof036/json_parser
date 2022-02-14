use std::borrow::Cow;
use serde::{Serialize, Deserialize};

pub const RUST_DEFINITION: TransformConfig = TransformConfig {
    type_definition: Cow::Borrowed("#[derive(Serialize, Deserialize, Debug)]\nstruct {object_name} {"),
    field_definition: Cow::Borrowed("\t{field_name}: {field_type},"),
    name_change_annotation: Cow::Borrowed("\t#[serde(rename = \"{name}\")]"),
    array_definition: Cow::Borrowed("Vec<{field_type}>"),
    block_end: Cow::Borrowed("}"),
    int_type: Cow::Borrowed("i32"),
    float_type: Cow::Borrowed("f32"),
    bool_type: Cow::Borrowed("bool"),
    string_type: Cow::Borrowed("String"),
    constructor: None,
    case_type: CaseType::SnakeCase,
    object_case_type: CaseType::UpperCamelCase,
};

pub const JAVA_DEFINITION: TransformConfig = TransformConfig {
    type_definition: Cow::Borrowed("class {object_name} {"),
    field_definition: Cow::Borrowed("\tprivate final {field_type} {field_name};"),
    name_change_annotation: Cow::Borrowed("\t@SerializedName(value = \"{name}\")"),
    array_definition: Cow::Borrowed("{field_type}[]"),
    block_end: Cow::Borrowed("}"),
    int_type: Cow::Borrowed("int"),
    float_type: Cow::Borrowed("double"),
    bool_type: Cow::Borrowed("boolean"),
    string_type: Cow::Borrowed("String"),
    case_type: CaseType::CamelCase,
    object_case_type: CaseType::UpperCamelCase,
    constructor: Some(
        ConstructorConfig {
            definition: Cow::Borrowed("\tpublic {object_name}({arguments}) {"),
            argument_definition: Cow::Borrowed("{type} {name}"),
            separator: Cow::Borrowed(", "),
            separator_at_end: false,
            field_definition: Some(ConstructorField{
                field_definition: Cow::Borrowed("\t\tthis.{name} = {name};"),
                end: Cow::Borrowed("\t}"),
            })
        }
    ),
};

pub const DART_DEFINITION: TransformConfig = TransformConfig {
    type_definition: Cow::Borrowed("class {object_name} {"),
    field_definition: Cow::Borrowed("\tfinal {field_type}? {field_name};"),
    name_change_annotation: Cow::Borrowed("\t@JsonKey(name: '{name}')"),
    array_definition: Cow::Borrowed("List<{field_type}>"),
    block_end: Cow::Borrowed("}"),
    int_type: Cow::Borrowed("int"),
    float_type: Cow::Borrowed("double"),
    bool_type: Cow::Borrowed("bool"),
    string_type: Cow::Borrowed("String"),
    case_type: CaseType::CamelCase,
    object_case_type: CaseType::UpperCamelCase,
    constructor: Some(
        ConstructorConfig {
        definition: Cow::Borrowed("\t{object_name}({{arguments}\n\t});"),
        argument_definition: Cow::Borrowed("\n\t\tthis.{name}"),
        separator: Cow::Borrowed("), "),
        separator_at_end: true,
        field_definition: None,
    })
};

pub const KOTLIN_DEFINITION: TransformConfig = TransformConfig {
    type_definition: Cow::Borrowed("data class {object_name} ("),
    field_definition: Cow::Borrowed("\tval {field_name}: {field_type},"),
    name_change_annotation: Cow::Borrowed("\t@JsonKey(name: '{name}')"),
    array_definition: Cow::Borrowed("{field_type}[]"),
    block_end: Cow::Borrowed(");"),
    int_type: Cow::Borrowed("int"),
    float_type: Cow::Borrowed("double"),
    bool_type: Cow::Borrowed("bool"),
    string_type: Cow::Borrowed("String"),
    case_type: CaseType::CamelCase,
    object_case_type: CaseType::UpperCamelCase,
    constructor: None,
};

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum CaseType {
    SnakeCase,
    UpperCamelCase,
    CamelCase
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransformConfig {
    pub type_definition: Cow<'static, str>,
    pub field_definition: Cow<'static, str>,
    pub name_change_annotation: Cow<'static, str>,
    pub array_definition: Cow<'static, str>,
    pub block_end: Cow<'static, str>,
    pub int_type: Cow<'static, str>,
    pub float_type: Cow<'static, str>,
    pub bool_type: Cow<'static, str>,
    pub string_type: Cow<'static, str>,
    pub constructor: Option<ConstructorConfig>,
    pub case_type: CaseType,
    pub object_case_type: CaseType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConstructorConfig {
    pub definition: Cow<'static, str>,
    pub argument_definition: Cow<'static, str>,
    pub separator: Cow<'static, str>,
    pub separator_at_end: bool,
    pub field_definition: Option<ConstructorField>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConstructorField {
    pub field_definition: Cow<'static, str>,
    pub end: Cow<'static, str>,
}