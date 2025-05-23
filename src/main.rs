use std::sync::Arc;

use nu_plugin::{serve_plugin, MsgPackSerializer, Plugin, PluginCommand};
use nu_plugin::{EngineInterface, EvaluatedCall};
use nu_protocol::{Category, Config, Example, LabeledError, PipelineData, Signature, Value};
use ptree::item::StringItem;
use ptree::output::print_tree_with;
use ptree::print_config::PrintConfig;
use ptree::style::{Color, Style};
use ptree::TreeBuilder;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn version(&self) -> String {
        // This automatically uses the version of your package from Cargo.toml as the plugin version
        // sent to Nushell
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            // Commands should be added here
            // Box::new(TreeView::new(TreeBuilder::new("root".to_string()).build())),
            Box::new(TreeView),
        ]
    }
}

pub struct TreeView;
//  {
//     tree: StringItem,
// }

// impl TreeView {
//     fn new(tree: StringItem) -> TreeView {
//         Self { tree }
//     }
// }

impl PluginCommand for TreeView {
    type Plugin = TreePlugin;

    fn name(&self) -> &str {
        "tree"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name()).category(Category::Experimental)
    }

    fn description(&self) -> &str {
        "View the contents of the pipeline as a tree."
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "scope commands | where name == with-env | tree",
                description: "Transform the tabular output into a tree",
                result: None,
            },
            Example {
                example: "ls | tree",
                description: "Transform the tabular output into a tree",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &TreePlugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let _span = call.head;
        let config = engine.get_config()?;

        // Process different types of input
        let tree = match input {
            PipelineData::ListStream(list_stream, _) => {
                // For list streams, consume the stream directly
                let values = list_stream.into_value();
                let mut tree_builder = TreeBuilder::new("root".to_string());
                from_value_helper(&values, &mut tree_builder, config);
                tree_builder.build()
            }
            _ => from_value(&input, config),
        };

        // Set up the print configuration
        let tree_config = {
            let mut tree_config = PrintConfig::from_env();
            tree_config.branch = Style {
                foreground: Some(Color::Green),
                dimmed: true,
                ..Style::default()
            };
            tree_config.leaf = Style {
                bold: true,
                ..Style::default()
            };
            tree_config.indent = 4;
            tree_config
        };

        // Print out the tree using custom formatting
        print_tree_with(&tree, &tree_config)
            .map_err(|err| LabeledError::new(format!("Error calculating tree: {}", err)))?;

        Ok(PipelineData::Empty)
    }
}

fn from_value(input: &PipelineData, config: Arc<Config>) -> StringItem {
    let mut tree = TreeBuilder::new("".to_string());
    let builder = &mut tree;

    match input {
        PipelineData::Empty => {
            builder.add_empty_child("empty".to_string());
        }
        PipelineData::Value(value, _pipeline_metadata) => {
            builder.begin_child("value".to_string());
            from_value_helper(value, builder, config);
            builder.end_child();
        }
        PipelineData::ListStream(_, _) => {
            // For ListStreams, just add a placeholder node since we can't easily iterate over a reference
            builder.begin_child("list stream".to_string());
            builder.add_empty_child("<contains stream data>".to_string());
            builder.end_child();
        }
        PipelineData::ByteStream(_byte_stream, _pipeline_metadata) => {
            builder.add_empty_child("binary stream".to_string());
        }
    }

    builder.build()
}

fn from_value_helper(value: &Value, builder: &mut TreeBuilder, config: Arc<Config>) {
    match value {
        Value::Bool { val, .. } => {
            builder.add_empty_child(val.to_string());
        }
        Value::Int { val, .. } => {
            builder.add_empty_child(val.to_string());
        }
        Value::Float { val, .. } => {
            builder.add_empty_child(val.to_string());
        }
        Value::String { val, .. } => {
            builder.add_empty_child(val.clone());
        }
        Value::Glob { val, .. } => {
            builder.add_empty_child(val.to_string());
        }
        Value::Filesize { val, .. } => {
            builder.add_empty_child(val.to_string());
        }
        Value::Duration { val, .. } => {
            builder.add_empty_child(val.to_string());
        }
        Value::Date { val, .. } => {
            builder.add_empty_child(val.to_string());
        }
        Value::Range { val, .. } => {
            builder.add_empty_child(val.to_string());
        }
        Value::Record { val, .. } => {
            for (k, v) in val.iter() {
                builder.begin_child(k.clone());
                from_value_helper(v, builder, config.clone());
                builder.end_child();
            }
        }
        Value::List { vals, .. } => {
            for value in vals {
                from_value_helper(value, builder, config.clone());
            }
        }
        Value::Closure { val, .. } => {
            builder.add_empty_child(val.block_id.get().to_string());
        }
        Value::Error { error, .. } => {
            builder.add_empty_child(error.to_string());
        }
        Value::Binary { .. } => {
            builder.add_empty_child("binary".to_string());
        }
        Value::CellPath { val, .. } => {
            builder.add_empty_child(val.to_string());
        }
        Value::Custom { .. } => {
            builder.add_empty_child("custom".to_string());
        }
        Value::Nothing { .. } => {
            builder.add_empty_child("null".to_string());
        }
    }
}

#[test]
fn test_examples() -> Result<(), nu_protocol::ShellError> {
    use nu_plugin_test_support::PluginTest;

    // This will automatically run the examples specified in your command and compare their actual
    // output against what was specified in the example. You can remove this test if the examples
    // can't be tested this way, but we recommend including it if possible.

    PluginTest::new("tree", TreePlugin.into())?.test_command_examples(&TreeView)
}

fn main() {
    serve_plugin(&TreePlugin, MsgPackSerializer);
}
