use alice_core::components::{
    ConfigComponent, LoopComponent, MessagesComponent, ProviderComponent, ToolsComponent,
};
use alice_core::effect::Effect;
use alice_core::effect_executor::EffectExecutor;
use alice_core::event::LLMStreamEvent;
use alice_core::event_bus::EventBus;
use alice_core::providers::StreamingProvider;
use alice_core::tool_scheduler::ToolScheduler;
use alice_core::abort_manager::AbortManager;
use alice_core::types::{Message, ToolDef};
use alice_core::world::{HasComponent, World};
use futures_core::Stream;
use std::pin::Pin;

#[derive(Default)]
struct TestComponents {
    messages: MessagesComponent,
    config: ConfigComponent,
    loop_state: LoopComponent,
    tools: ToolsComponent,
    provider: ProviderComponent,
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

impl HasComponent<ProviderComponent> for TestComponents {
    fn get(&self) -> &ProviderComponent { &self.provider }
    fn get_mut(&mut self) -> &mut ProviderComponent { &mut self.provider }
}

struct NullProvider;

impl StreamingProvider for NullProvider {
    fn format_messages(&self, _messages: &[Message]) -> serde_json::Value {
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
            assert!(!tool_call_id.is_empty());
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
        fn format_messages(&self, _messages: &[Message]) -> serde_json::Value {
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
