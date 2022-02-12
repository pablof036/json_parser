use std::mem;
use crate::lib::model::transform_config::TransformConfig;
use crate::lib::model::tree::JsonTree;
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

        Ok(Self {
            name,
            config,
            tree,
            output: vec![],
        })
    }

    fn transform_object(&mut self, tree: Vec<JsonTree>, name: String) {
        let mut object: Vec<String> = Vec::new();

        object.push(self.config.type_definition.replace("{object_name}", &name));

        let mut tree = tree.into_iter();
        while let Some(item) = tree.next() {
            object.push(self.parse_tree(&item));
            if let JsonTree::JsonObject(name, object) = item {
                self.transform_object(object, name);
            }
        }

        object.push(self.config.block_end.to_owned());

        self.output.push(object);
    }

    fn parse_tree(&self, tree: &JsonTree) -> String {
        let field_str = self.config.field_definition;
        let array_type_str = self.config.array_definition;

        match tree {
            JsonTree::Int(name) => {
                let with_name = field_str.replace("{field_name}", name);
                with_name.replace("{field_type}", self.config.int_type)
            }
            JsonTree::Float(name) => {
                let with_name = field_str.replace("{field_name}", name);
                with_name.replace("{field_type}", self.config.float_type)
            }
            JsonTree::String(name) => {
                let with_name = field_str.replace("{field_name}", name);
                with_name.replace("{field_type}", self.config.string_type)
            }
            JsonTree::Bool(name) => {
                let with_name = field_str.replace("{field_name}", name);
                with_name.replace("{field_type}", self.config.bool_type)
            }
            JsonTree::JsonObject(name, _) => {
                let with_name = field_str.replace("{field_name}", name);
                with_name.replace("{field_type}", name)
            }
            JsonTree::JsonArray(name, _) => {
                let with_name = field_str.replace("{field_name}", name);
                let with_array_type = with_name.replace("{field_type}", array_type_str);
                with_array_type.replace("{field_type}", name)
            }
        }
    }

    pub fn start_transform(mut self) -> Vec<Vec<String>> {
        let tree = mem::replace(&mut self.tree, Vec::new());
        self.transform_object(tree, self.name.unwrap_or_else(|| "Root").to_owned());
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
            single_file: true,
        };

        Transformer::new(bad_config, vec![], None).unwrap();
    }
}