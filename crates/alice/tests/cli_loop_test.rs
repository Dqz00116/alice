use alice_core::components::{
    ConfigComponent, LoopComponent, MessagesComponent, ToolsComponent,
};
use alice_core::effect_executor::EffectExecutor;
use alice_core::event::{Event, InputEvent, LLMStreamEvent, SystemEvent};
use alice_core::middleware::MiddlewarePipeline;
use alice_core::providers::StreamingProvider;
use alice_core::system_registry::SystemRegistry;
use alice_core::systems::{hook, input, output, provider, tool};
use alice_core::tool_scheduler::ToolScheduler;
use alice_core::abort_manager::AbortManager;
use alice_core::types::{Message, ToolDef};
use alice_core::world::{HasComponent, World};
use futures_core::Stream;
use std::collections::VecDeque;
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

struct EchoProvider;

impl StreamingProvider for EchoProvider {
    fn format_messages(&self, messages: &[Message], _tools: &[ToolDef]) -> serde_json::Value {
        serde_json::json!({ "messages": messages.len() })
    }

    fn stream_chat(
        &self,
        _body: serde_json::Value,
    ) -> Pin<Box<dyn Stream<Item = LLMStreamEvent> + Send + '_>> {
        Box::pin(futures_util::stream::iter(vec![
            LLMStreamEvent::TextDelta { delta: "Hello".into() },
            LLMStreamEvent::TextDelta { delta: " back".into() },
            LLMStreamEvent::StreamEnd { stop_reason: "end_turn".into() },
        ]))
    }
}

#[tokio::test]
async fn test_full_loop_with_mock_provider() {
    let mut world = World::new(TestComponents {
        loop_state: LoopComponent { step: 0, should_continue: true },
        ..TestComponents::default()
    });

    let mut queue: VecDeque<Event> = VecDeque::new();
    let tool_scheduler = ToolScheduler::new();
    let mut abort_manager = AbortManager::new();

    let mut registry = SystemRegistry::<TestComponents>::new();
    registry.register(input::input_system::<TestComponents>, &["input.user"]);
    registry.register(provider::provider_system::<TestComponents>, &[
        "system.step_start",
        "tool.result",
        "tool.error",
    ]);
    registry.register(tool::tool_system::<TestComponents>, &["llm.tool_call"]);
    registry.register(
        output::output_system::<TestComponents>,
        &[
            "llm.thinking_delta",
            "llm.text_delta",
            "llm.tool_call",
            "llm.stream_end",
            "llm.stream_error",
        ],
    );
    registry.register(hook::hook_system::<TestComponents>, &["system.hook_trigger"]);

    let provider = EchoProvider;

    queue.push_back(Event::Input(InputEvent {
        source: "test".into(),
        content: "Hi".into(),
    }));

    while let Some(event) = queue.pop_front() {
        let systems = registry.get_systems_for_event(&event);
        let effects = {
            let snapshot = world.snapshot();
            let mut effects = Vec::new();
            for system in systems {
                effects.extend(system.process(&snapshot, &event));
            }
            effects
        };

        let mut executor = EffectExecutor::new(
            &mut world,
            &mut queue,
            &tool_scheduler,
            &mut abort_manager,
            &provider,
        );
        executor.execute(effects).await;

        if !world.get::<LoopComponent>().should_continue {
            break;
        }
    }

    let messages = &world.get::<MessagesComponent>().messages;
    assert_eq!(messages.len(), 2, "expected user + assistant messages");
    assert!(matches!(messages[0], Message::User { .. }));
    assert!(matches!(messages[1], Message::Assistant { .. }));
    assert_eq!(world.get::<LoopComponent>().step, 1);
}

#[tokio::test]
async fn test_middleware_transforms_step_start() {
    let mut world = World::new(TestComponents {
        loop_state: LoopComponent { step: 0, should_continue: true },
        ..TestComponents::default()
    });

    let mut queue: VecDeque<Event> = VecDeque::new();
    let tool_scheduler = ToolScheduler::new();
    let mut abort_manager = AbortManager::new();

    let mut registry = SystemRegistry::<TestComponents>::new();
    registry.register(input::input_system::<TestComponents>, &["input.user"]);
    registry.register(provider::provider_system::<TestComponents>, &[
        "system.step_start",
        "tool.result",
        "tool.error",
    ]);
    registry.register(
        |_snapshot: &alice_core::world::Snapshot<&TestComponents>, event: &Event| {
            if let Event::System(SystemEvent::StepStart { step }) = event {
                assert_eq!(*step, 42, "middleware should transform step to 42");
            }
            vec![]
        },
        &["system.step_start"],
    );
    registry.register(tool::tool_system::<TestComponents>, &["llm.tool_call"]);
    registry.register(
        output::output_system::<TestComponents>,
        &[
            "llm.thinking_delta",
            "llm.text_delta",
            "llm.tool_call",
            "llm.stream_end",
            "llm.stream_error",
        ],
    );
    registry.register(hook::hook_system::<TestComponents>, &["system.hook_trigger"]);

    let provider = EchoProvider;

    let mut pipeline = MiddlewarePipeline::new();
    pipeline.add(|mut event, next| {
        if let Event::System(SystemEvent::StepStart { ref mut step }) = event {
            *step = 42;
        }
        next.run(event)
    });

    queue.push_back(Event::Input(InputEvent {
        source: "test".into(),
        content: "Hi".into(),
    }));

    while let Some(raw_event) = queue.pop_front() {
        let event = pipeline.run(raw_event);
        let systems = registry.get_systems_for_event(&event);
        let effects = {
            let snapshot = world.snapshot();
            let mut effects = Vec::new();
            for system in systems {
                effects.extend(system.process(&snapshot, &event));
            }
            effects
        };

        let mut executor = EffectExecutor::new(
            &mut world,
            &mut queue,
            &tool_scheduler,
            &mut abort_manager,
            &provider,
        );
        executor.execute(effects).await;

        if !world.get::<LoopComponent>().should_continue {
            break;
        }
    }

    // The probe system above would panic if the transformed step were not observable.
    let messages = &world.get::<MessagesComponent>().messages;
    assert_eq!(messages.len(), 2, "loop should still complete after transformation");
}
