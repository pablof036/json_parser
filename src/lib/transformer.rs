use std::mem;
use crate::lib::model::transform_config::TransformConfig;
use crate::lib::model::tree::{JsonArrayType, JsonTree};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransformerError<'a> {
    #[error("Bad type definition in config: {{name}} needed.\n{0}")]
    BadTypeDefinition(&'a str),
    #[error("Bad field definition in config: {{field_name}} needed.\n{0}")]
    BadFieldDefinitionName(&'a str),
    #[error("Bad field definition in config: {{field_type}} needed. \n{0}")]
    BadFieldDefinitionType(&'a str),
    #[error("Bad array type definition in config: {{field_type}} needed. \n {0}")]
    BadArrayTypeDefinition(&'a str),
    #[error("Bad constructor definition: {{object_name}} needed.\n {0}")]
    BadConstructorDefinitionName(&'a str),
    #[error("Bad constructor definition: {{arguments}} needed.\n {0}")]
    BadConstructorDefinitionArgument(&'a str),
    #[error("Bad argument definition: {{name}} needed.\n {0}")]
    BadArgumentDefinitionName(&'a str),
    #[error("Bad constructor field definition: {{name}} needed.\n {0}")]
    BadConstructorFieldDefinition(&'a str),
}


pub struct Transformer<'a> {
    name: Option<&'a str>,
    config: TransformConfig<'a>,
    tree: Vec<JsonTree>,
    output: Vec<Vec<String>>,
}

impl<'a> Transformer<'a> {
    pub fn new(config: TransformConfig<'a>, tree: Vec<JsonTree>, name: Option<&'a str>) -> Result<Self, TransformerError<'a>> {
        let field_str = config.field_definition;
        let array_type_str = config.array_definition;
        let type_str = config.type_definition;

        if !type_str.contains("{object_name}") {
            return Err(TransformerError::BadTypeDefinition(type_str));
        }

        if !field_str.contains("{field_name}") {
            return Err(TransformerError::BadFieldDefinitionName(field_str));
        }

        if !field_str.contains("{field_type}") {
            return Err(TransformerError::BadFieldDefinitionType(field_str));
        }

        if !array_type_str.contains("{field_type}") {
            return Err(TransformerError::BadArrayTypeDefinition(array_type_str));
        }

        if let Some(ref constructor) = config.constructor {
            let constructor_str = constructor.definition;
            let argument_str = constructor.argument_definition;

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
                    return Err(TransformerError::BadConstructorFieldDefinition(field.field_definition));
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

    fn transform_object(&mut self, tree: &Vec<JsonTree>, name: String) {
        let mut object: Vec<String> = Vec::new();

        object.push(self.config.type_definition.replace("{object_name}", &name));

        let fields: Vec<(String, &String)> = tree.iter().map(|tree| match tree {
            JsonTree::Int(name) => (self.config.int_type.to_owned(), name),
            JsonTree::Float(name) => (self.config.float_type.to_owned(), name),
            JsonTree::String(name) => (self.config.string_type.to_owned(), name),
            JsonTree::Bool(name) => (self.config.bool_type.to_owned(), name),
            JsonTree::JsonObject(name, tree) => {
                self.transform_object(tree, name.to_owned());
                (name.clone(), name)
            },
            JsonTree::JsonArray(name, array_type) => {
                if let JsonArrayType::JsonObject(tree) = array_type {
                    self.transform_object(tree, name.to_owned());
                }
                let array_str = self.config.array_definition.replace("{field_type}", name);
                (array_str, name)
            }
        }).collect();

        for (type_str, name) in &fields {
            let with_name = self.config.field_definition.replace("{field_name}", name);
            object.push(with_name.replace("{field_type}", &type_str));
        }


        if let Some(ref constructor) = self.config.constructor {
            let mut arguments_str = String::new();
            for (i, (type_str, name)) in fields.iter().enumerate() {
                let with_type = constructor.argument_definition.replace("{type}", type_str);
                let with_name = with_type.replace("{name}", name);
                if i < fields.len() - 1 || (i == fields.len() - 1 && constructor.separator_at_end) {
                    arguments_str.push_str(&*(with_name + constructor.separator));
                } else {
                    arguments_str.push_str(&with_name);
                }
            }

            let with_name = constructor.definition.replace("{object_name}", &name);
            object.push(with_name.replace("{arguments}", &arguments_str));

            if let Some(ref field) = constructor.field_definition {
                for (_, name) in fields {
                    object.push(field.field_definition.replace("{name}", name));
                }
                object.push(field.end.to_owned());
            }
        }

        object.push(self.config.block_end.to_owned());

        self.output.push(object);
    }

    pub fn start_transform(mut self) -> Vec<Vec<String>> {
        let tree = mem::replace(&mut self.tree, Vec::new());
        self.transform_object(&tree, self.name.unwrap_or_else(|| "Root").to_owned());
        self.output
    }
}


#[cfg(test)]
mod tests {
    use crate::lib::model::transform_config::{RUST_DEFINITION, TransformConfig};
    use crate::lib::parser::lexer::Lexer;
    use crate::lib::parser::tokenizer::Tokenizer;
    use crate::lib::transformer::Transformer;

    #[test]
    fn simple_json() {
        let json = "{\"f1\": \"value\", \"f2\": true, \"f3\": 45.3, \"f4\": 12}";
        let expected_result = vec![
            vec![
                "struct Root {",
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
                "struct f4 {",
                "\tf5: bool,",
                "}",
            ],
            vec![
                "struct Root {",
                "\tf1: String,",
                "\tf2: bool,",
                "\tf3: f32,",
                "\tf4: f4,",
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
            type_definition: "{nn}",
            field_definition: "\t{field_ame}: {field_ype}",
            array_definition: "Vec<{field_type}>",
            block_end: "}",
            int_type: "i32",
            float_type: "f32",
            bool_type: "bool",
            string_type: "String",
            constructor: None
        };

        Transformer::new(bad_config, vec![], None).unwrap();
    }
}