use nu_plugin::{serve_plugin, MsgPackSerializer, Plugin, PluginCommand};
use nu_plugin::{EngineInterface, EvaluatedCall};
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signals, Signature, Type, Value};

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
            Box::new(ViewTree),
        ]
    }
}

pub struct ViewTree;

impl PluginCommand for ViewTree {
    type Plugin = TreePlugin;

    fn name(&self) -> &str {
        "tree"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .switch("shout", "(FIXME) Yell it instead", None)
            .input_output_type(
                Type::List(Type::String.into()),
                Type::List(Type::String.into()),
            )
            .category(Category::Experimental)
    }

    fn description(&self) -> &str {
        "(FIXME) help text for tree"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "[ Ellie ] | tree",
                description: "Say hello to Ellie",
                result: Some(Value::test_list(vec![Value::test_string(
                    "Hello, Ellie. How are you today?",
                )])),
            },
            Example {
                example: "[ Ellie ] | tree --shout",
                description: "Shout hello to Ellie",
                result: Some(Value::test_list(vec![Value::test_string(
                    "HELLO, ELLIE. HOW ARE YOU TODAY?",
                )])),
            },
        ]
    }

    fn run(
        &self,
        _plugin: &TreePlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let span = call.head;
        let shout = call.has_flag("shout")?;
        Ok(input.map(
            move |name| match name.as_str() {
                Ok(name) => {
                    let mut greeting = format!("Hello, {name}. How are you today?");
                    if shout {
                        greeting = greeting.to_uppercase();
                    }
                    Value::string(greeting, span)
                }
                Err(err) => Value::error(err, span),
            },
            &Signals::empty(),
        )?)
    }
}

#[test]
fn test_examples() -> Result<(), nu_protocol::ShellError> {
    use nu_plugin_test_support::PluginTest;

    // This will automatically run the examples specified in your command and compare their actual
    // output against what was specified in the example. You can remove this test if the examples
    // can't be tested this way, but we recommend including it if possible.

    PluginTest::new("tree", TreePlugin.into())?.test_command_examples(&ViewTree)
}

fn main() {
    serve_plugin(&TreePlugin, MsgPackSerializer);
}
