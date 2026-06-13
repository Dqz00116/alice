use alice_core::components::{
    ConfigComponent, LoopComponent, MessagesComponent, ToolsComponent,
};
use alice_core::effect::Effect;
use alice_core::effect_executor::EffectExecutor;
use alice_core::event::{Event, LLMStreamEvent};
use alice_core::event_bus::EventBus;
use alice_core::providers::StreamingProvider;
use alice_core::systems::tool::tool_system;
use alice_core::tool_scheduler::ToolScheduler;
use alice_core::abort_manager::AbortManager;
use alice_core::types::{FunctionCall, Message, ToolCall, ToolDef};
use alice_core::world::{HasComponent, World};
use futures_core::Stream;
use std::pin::Pin;

#[derive(Default)]
struct TestComponents {
    messages: MessagesComponent,
    config: ConfigComponent,
    loop_state: LoopComponent,
    tools: ToolsComponent,
}

impl HasComponent<MessagesComponent> for TestComponents {
    fn get(&self) -> &MessagesComponent { &self.messages }
    fn get_mut(&mut self) -> &mut MessagesComponent { &mut self.messages }
}

impl HasComponent<ConfigComponent> for TestComponents {
    fn get(&self) -> &ConfigComponent { &self.config }
    fn get_mut(&mut self) -> &mut ConfigComponent { &mut self.config }
}

impl HasComponent<LoopComponent> for TestComponents {
    fn get(&self) -> &LoopComponent { &self.loop_state }
    fn get_mut(&mut self) -> &mut LoopComponent { &mut self.loop_state }
}

impl HasComponent<ToolsComponent> for TestComponents {
    fn get(&self) -> &ToolsComponent { &self.tools }
    fn get_mut(&mut self) -> &mut ToolsComponent { &mut self.tools }
}

struct NullProvider;

impl StreamingProvider for NullProvider {
    fn format_messages(&self, _messages: &[Message], _tools: &[ToolDef]) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn stream_chat(
        &self,
        _body: serde_json::Value,
    ) -> Pin<Box<dyn Stream<Item = LLMStreamEvent> + Send + '_>> {
        Box::pin(futures_util::stream::iter(vec![]))
    }
}

#[tokio::test]
async fn test_execute_tool_appends_tool_message() {
    let mut world = World::new(TestComponents::default());
    let mut event_bus = EventBus::new();
    let mut tool_scheduler = ToolScheduler::new();
    tool_scheduler.register(
        ToolDef {
            name: "echo".into(),
            description: "Echo back the input".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            }),
        },
        |args| {
            args.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        },
    );
    let mut abort_manager = AbortManager::new();

    let provider = NullProvider;
    let mut executor = EffectExecutor::new(
        &mut world,
        &mut event_bus,
        &tool_scheduler,
        &mut abort_manager,
        &provider,
    );
    executor
        .execute(vec![Effect::ExecuteTool {
            tool_call_id: "toolu_original_id".into(),
            tool_name: "echo".into(),
            args: serde_json::json!({ "message": "hello" }),
        }])
        .await;

    let msgs = &world.get::<MessagesComponent>().messages;
    assert_eq!(msgs.len(), 1);
    match &msgs[0] {
        Message::Tool {
            content,
            tool_call_id,
        } => {
            assert_eq!(content, "hello");
            assert_eq!(tool_call_id, "toolu_original_id");
        }
        _ => panic!("expected Message::Tool, got {:?}", msgs[0]),
    }
}

#[tokio::test]
async fn test_call_llm_increments_loop_step() {
    let mut world = World::new(TestComponents {
        loop_state: LoopComponent { step: 0, should_continue: true },
        ..TestComponents::default()
    });
    let mut event_bus = EventBus::new();
    let tool_scheduler = ToolScheduler::new();
    let mut abort_manager = AbortManager::new();

    struct CountingProvider;
    impl StreamingProvider for CountingProvider {
        fn format_messages(&self, _messages: &[Message], _tools: &[ToolDef]) -> serde_json::Value {
            serde_json::Value::Null
        }
        fn stream_chat(
            &self,
            _body: serde_json::Value,
        ) -> Pin<Box<dyn Stream<Item = LLMStreamEvent> + Send + '_>> {
            Box::pin(futures_util::stream::iter(vec![
                LLMStreamEvent::TextDelta { delta: "hi".into() },
                LLMStreamEvent::StreamEnd { stop_reason: "end_turn".into() },
            ]))
        }
    }

    let provider = CountingProvider;
    let mut executor = EffectExecutor::new(
        &mut world,
        &mut event_bus,
        &tool_scheduler,
        &mut abort_manager,
        &provider,
    );
    executor
        .execute(vec![Effect::CallLLM { messages: vec![] }])
        .await;

    assert_eq!(world.get::<LoopComponent>().step, 1);
}

#[tokio::test]
async fn test_execute_tool_unknown_tool_preserves_id() {
    let mut world = World::new(TestComponents::default());
    let mut event_bus = EventBus::new();
    let tool_scheduler = ToolScheduler::new();
    let mut abort_manager = AbortManager::new();

    let provider = NullProvider;
    let mut executor = EffectExecutor::new(
        &mut world,
        &mut event_bus,
        &tool_scheduler,
        &mut abort_manager,
        &provider,
    );
    executor
        .execute(vec![Effect::ExecuteTool {
            tool_call_id: "toolu_missing".into(),
            tool_name: "nonexistent".into(),
            args: serde_json::json!({}),
        }])
        .await;

    let msgs = &world.get::<MessagesComponent>().messages;
    assert_eq!(msgs.len(), 1);
    match &msgs[0] {
        Message::Tool {
            content,
            tool_call_id,
        } => {
            assert!(content.contains("unknown tool"));
            assert_eq!(tool_call_id, "toolu_missing");
        }
        _ => panic!("expected Message::Tool, got {:?}", msgs[0]),
    }
}

#[test]
fn test_tool_system_preserves_tool_call_id() {
    let world = World::new(TestComponents::default());
    let snapshot = world.snapshot();
    let tool_call = ToolCall {
        id: "toolu_abc123".into(),
        call_type: "tool_use".into(),
        function: FunctionCall {
            name: "bash".into(),
            arguments: r#"{"command":"ls"}"#.into(),
        },
    };
    let event = Event::LLMStream(LLMStreamEvent::ToolCall { tool_call });
    let effects = tool_system(&snapshot, &event);

    assert_eq!(effects.len(), 1);
    match &effects[0] {
        Effect::ExecuteTool {
            tool_call_id,
            tool_name,
            ..
        } => {
            assert_eq!(tool_call_id, "toolu_abc123");
            assert_eq!(tool_name, "bash");
        }
        _ => panic!("expected Effect::ExecuteTool, got {:?}", effects[0]),
    }
}
