use crate::lib::model::transform_config::TransformConfig;
use crate::lib::model::tree::JsonTree;

pub struct Transformer<'a> {
    config: TransformConfig<'a>,
    tree: Vec<JsonTree<'a>>,
    output: Vec<Vec<&'a str>>,
}

impl<'a> Transformer<'a> {
    fn new(config: TransformConfig<'a>, tree: JsonTree<'a>) -> Self {
        if let JsonTree::Root(tree) = tree {
            Self {
                config,
                tree,
                output: vec![]
            }
        } else {
            panic!("tree root not provided")
        }
    }

    fn transform_object(&mut self) {
        while let Some(item) = self.tree.iter().next() {
            match item {
                JsonTree::Int(_) => {},
                JsonTree::Float(_) => {}
                JsonTree::String(_) => {}
                JsonTree::Bool(_) => {}
                JsonTree::JsonObject(_, _) => {}
                JsonTree::JsonArray(_, _) => {}
                JsonTree::Root(_) => {}
            }
        }
    }


    fn start_transform(mut self) -> Vec<&'a str> {
        self.output
    }
}