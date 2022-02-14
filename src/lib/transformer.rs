use std::mem;
use crate::lib::model::transform_config::TransformConfig;
use crate::lib::model::tree::{JsonArrayType, JsonTree};
use thiserror::Error;
use crate::lib::case::{convert_case};

#[derive(Error, Debug)]
pub enum TransformerError {
    #[error("Bad type definition in config: {{name}} needed.\n{0}")]
    BadTypeDefinition(String),
    #[error("Bad field definition in config: {{field_name}} needed.\n{0}")]
    BadFieldDefinitionName(String),
    #[error("Bad field definition in config: {{field_type}} needed. \n{0}")]
    BadFieldDefinitionType(String),
    #[error("Bad field rename definition in config: {{name}} needed. \n{0}")]
    BadFieldRenameDefinition(String),
    #[error("Bad array type definition in config: {{field_type}} needed. \n {0}")]
    BadArrayTypeDefinition(String),
    #[error("Bad constructor definition: {{object_name}} needed.\n {0}")]
    BadConstructorDefinitionName(String),
    #[error("Bad constructor definition: {{arguments}} needed.\n {0}")]
    BadConstructorDefinitionArgument(String),
    #[error("Bad argument definition: {{name}} needed.\n {0}")]
    BadArgumentDefinitionName(String),
    #[error("Bad constructor field definition: {{name}} needed.\n {0}")]
    BadConstructorFieldDefinition(String),
}


/// Holds the data needed to turn a [JsonTree] into a representation provided by [TransformConfig].
pub struct Transformer {
    /// Name of the root object.
    name: Option<String>,
    /// Wanted representation of the [JsonTree]
    config: TransformConfig,
    /// Source tree
    tree: Vec<JsonTree>,
    /// Output of the transformer.
    /// Each vec represents an object, each String inside that vec represents a line.
    output: Vec<Vec<String>>,
}

/// Holds the type and name (maybe converted) of a field from [JsonTree] ready for writing into the output.
struct FieldInfo<'a> {
    ///In case the name is converted, `original_str` will be used in an annotation provided by [TransformConfig].
    original_str: &'a str,
    ///Type of the field.
    type_str: String,
    ///Name string, could be converted.
    name: String,
}

impl Transformer {

    /// Creates a new [Transformer].
    /// # Arguments
    /// * `config` config for output. Will be checked for correctness.
    /// * `tree` source json tree.
    /// * `name` name of the root object
    /// # Errors
    /// If [TransformConfig] contains invalid data, a [TransformerError] will be returned.
    pub fn new<'a>(config: TransformConfig, tree: Vec<JsonTree>, name: Option<String>) -> Result<Self, TransformerError> {
        let field_str = config.field_definition.to_string();
        let field_rename_str = config.name_change_annotation.to_string();
        let array_type_str = config.array_definition.to_string();
        let type_str = config.type_definition.to_string();

        if !type_str.contains("{object_name}") {
            return Err(TransformerError::BadTypeDefinition(type_str));
        }

        if !field_str.contains("{field_name}") {
            return Err(TransformerError::BadFieldDefinitionName(field_str));
        }

        if !field_rename_str.contains("{name}") {
            return Err(TransformerError::BadFieldRenameDefinition(type_str));
        }

        if !field_str.contains("{field_type}") {
            return Err(TransformerError::BadFieldDefinitionType(field_str));
        }

        if !array_type_str.contains("{field_type}") {
            return Err(TransformerError::BadArrayTypeDefinition(array_type_str));
        }

        if let Some(ref constructor) = config.constructor {
            let constructor_str = constructor.definition.to_string();
            let argument_str = constructor.argument_definition.to_string();

            if !constructor_str.contains("{object_name}") {
                return Err(TransformerError::BadConstructorDefinitionName(constructor_str));
            }

            if !constructor_str.contains("{arguments}") {
                return Err(TransformerError::BadConstructorDefinitionArgument(constructor_str));
            }

            if !argument_str.contains("{name}") {
                return Err(TransformerError::BadArgumentDefinitionName(argument_str));
            }

            if let Some(ref field) = constructor.field_definition {
                if !field.field_definition.contains("{name}") {
                    return Err(TransformerError::BadConstructorFieldDefinition(field.field_definition.to_string()));
                }
            }
        }

        Ok(Self {
            name,
            config,
            tree,
            output: vec![],
        })
    }

    /// Transforms an object of the tree.
    /// # Arguments
    /// * `tree` object source
    /// * `name` of the object
    fn transform_object(&mut self, tree: &Vec<JsonTree>, name: String) {
        let mut object: Vec<String> = Vec::new();

        object.push(self.config.type_definition.replace("{object_name}", &name));

        let fields: Vec<FieldInfo> = tree.iter().map(|tree| match tree {
            JsonTree::Int(name) => FieldInfo {
                type_str: self.config.int_type.to_string(),
                original_str: name,
                name: convert_case(name, &self.config.case_type)
            },
            JsonTree::Float(name) => FieldInfo {
                type_str: self.config.float_type.to_string(),
                original_str: name,
                name: convert_case(name, &self.config.case_type)
            },
            JsonTree::String(name) => FieldInfo {
                type_str: self.config.string_type.to_string(),
                original_str: name,
                name: convert_case(name, &self.config.case_type)
            },
            JsonTree::Bool(name) => FieldInfo {
                type_str: self.config.bool_type.to_string(),
                original_str: name,
                name: convert_case(name, &self.config.case_type)
            },
            JsonTree::JsonObject(name, tree) => {
                let case_str = convert_case(name, &self.config.case_type);
                let type_str = convert_case(name, &self.config.object_case_type);
                self.transform_object(tree, type_str.clone());
                FieldInfo {
                    type_str,
                    original_str: name,
                    name: case_str
                }
            },
            JsonTree::JsonArray(name, array_type) => {
                let case_str = convert_case(name, &self.config.case_type);
                let mut array_str = self.config.array_definition.replace("{field_type}", &case_str);

                if let JsonArrayType::JsonObject(tree) = array_type {
                    let type_str = convert_case(name, &self.config.object_case_type);
                    self.transform_object(tree, type_str.clone());
                    array_str = self.config.array_definition.replace("{field_type}", &type_str);
                }

                FieldInfo {
                    type_str: array_str,
                    original_str: name,
                    name: case_str
                }
            }
        }).collect();


        for field_info in fields.iter() {

            if field_info.name != field_info.original_str {
                let with_name = self.config.name_change_annotation.replace("{name}", field_info.original_str);
                object.push(with_name);
            }

            let with_name = self.config.field_definition.replace("{field_name}", &field_info.name);
            object.push(with_name.replace("{field_type}", &field_info.type_str));
        }

        if let Some(ref constructor) = self.config.constructor {
            let mut arguments_str = String::new();
            for (i, field_info) in fields.iter().enumerate() {
                let with_type = constructor.argument_definition.replace("{type}", &field_info.type_str);
                let with_name = with_type.replace("{name}", &field_info.name);
                if i < fields.len() - 1 || (i == fields.len() - 1 && constructor.separator_at_end) {
                    arguments_str.push_str(&*(with_name + &constructor.separator));
                } else {
                    arguments_str.push_str(&with_name);
                }
            }

            let with_name = constructor.definition.replace("{object_name}", &name);
            object.push(with_name.replace("{arguments}", &arguments_str));

            if let Some(ref field) = constructor.field_definition {
                for field_info in fields {
                    object.push(field.field_definition.replace("{name}", &field_info.name));
                }
                object.push(field.end.to_string());
            }
        }

        object.push(self.config.block_end.to_string());

        self.output.push(object);
    }

    /// consumes the struct and start the transformation process.
    /// # Returns
    /// Struct's field `output`. Each vector represents an object, each object is made of a vector of lines.
    pub fn start_transform(mut self) -> Vec<Vec<String>> {
        let tree = mem::replace(&mut self.tree, Vec::new());
        let name = self.name.clone().unwrap_or_else(|| String::from("Root"));
        self.transform_object(&tree, name);
        self.output
    }
}


#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use crate::lib::model::transform_config::CaseType;
    use crate::lib::model::transform_config::{RUST_DEFINITION, TransformConfig};
    use crate::lib::parser::lexer::Lexer;
    use crate::lib::parser::tokenizer::Tokenizer;
    use crate::lib::transformer::Transformer;

    #[test]
    fn simple_json() {
        let json = "{\"f1\": \"value\", \"f2\": true, \"f3\": 45.3, \"f4\": 12}";
        let expected_result = vec![
            vec![
                "#[derive(Serialize, Deserialize, Debug)]\nstruct Root {",
                "\tf1: String,",
                "\tf2: bool,",
                "\tf3: f32,",
                "\tf4: i32,",
                "}",
            ]
        ];

        let lexer = Lexer::new(json);
        let tokenizer = Tokenizer::new(lexer.start_lex());
        let transformer = Transformer::new(RUST_DEFINITION, tokenizer.start_tokenizer().unwrap(), None).unwrap();
        let result = transformer.start_transform();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn nested_json() {
        let json = "{\"f1\": \"value\", \"f2\": true, \"f3\": 45.3, \"f4\": {\"f5\": true}}";
        let expected_result = vec![
            vec![
                "#[derive(Serialize, Deserialize, Debug)]\nstruct F4 {",
                "\tf5: bool,",
                "}",
            ],
            vec![
                "#[derive(Serialize, Deserialize, Debug)]\nstruct Root {",
                "\tf1: String,",
                "\tf2: bool,",
                "\tf3: f32,",
                "\tf4: F4,",
                "}",
            ],
        ];

        let lexer = Lexer::new(json);
        let tokenizer = Tokenizer::new(lexer.start_lex());
        let transformer = Transformer::new(RUST_DEFINITION, tokenizer.start_tokenizer().unwrap(), None).unwrap();
        let result = transformer.start_transform();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic]
    fn fail_on_bad_config() {
        let bad_config = TransformConfig {
            type_definition: Cow::Borrowed("{nn}"),
            field_definition: Cow::Borrowed("\t{field_ame}: {field_ype}"),
            name_change_annotation: Cow::Borrowed("a"),
            array_definition: Cow::Borrowed("Vec<{field_type}>"),
            block_end: Cow::Borrowed("}"),
            int_type: Cow::Borrowed("i32"),
            float_type: Cow::Borrowed("f32"),
            bool_type: Cow::Borrowed("bool"),
            string_type: Cow::Borrowed("String"),
            constructor: None,
            case_type: CaseType::CamelCase,
            object_case_type: CaseType::UpperCamelCase
        };

        Transformer::new(bad_config, vec![], None).unwrap();
    }
}