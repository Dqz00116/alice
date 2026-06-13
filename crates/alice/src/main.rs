use alice_core::components::{
    ConfigComponent, LoopComponent, MessagesComponent, ToolsComponent,
};
use alice_core::effect_executor::EffectExecutor;
use alice_core::event::{Event, InputEvent};
use alice_core::system_registry::SystemRegistry;
use alice_core::middleware::MiddlewarePipeline;
use alice_core::systems::{hook, input, output, provider, tool};
use alice_core::tool_scheduler::ToolScheduler;
use alice_core::abort_manager::AbortManager;
use alice_core::world::{HasComponent, World};
use alice_providers::anthropic::AnthropicProvider;
use alice_tools::{bash, echo, edit_file, glob, read_file, write_file};
use std::collections::VecDeque;
use std::io::{self, Write};

#[derive(Default)]
struct AllComponents {
    messages: MessagesComponent,
    config: ConfigComponent,
    loop_state: LoopComponent,
    tools: ToolsComponent,
}

impl HasComponent<MessagesComponent> for AllComponents {
    fn get(&self) -> &MessagesComponent { &self.messages }
    fn get_mut(&mut self) -> &mut MessagesComponent { &mut self.messages }
}

impl HasComponent<ConfigComponent> for AllComponents {
    fn get(&self) -> &ConfigComponent { &self.config }
    fn get_mut(&mut self) -> &mut ConfigComponent { &mut self.config }
}

impl HasComponent<LoopComponent> for AllComponents {
    fn get(&self) -> &LoopComponent { &self.loop_state }
    fn get_mut(&mut self) -> &mut LoopComponent { &mut self.loop_state }
}

impl HasComponent<ToolsComponent> for AllComponents {
    fn get(&self) -> &ToolsComponent { &self.tools }
    fn get_mut(&mut self) -> &mut ToolsComponent { &mut self.tools }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
    if api_key.is_none() {
        eprintln!("Warning: ANTHROPIC_API_KEY not set. LLM calls will fail.");
    }
    let base_url = match std::env::var("ANTHROPIC_BASE_URL") {
        Ok(url) if !url.is_empty() => url,
        _ => "https://api.anthropic.com".into(),
    };
    let model = match std::env::var("ANTHROPIC_MODEL") {
        Ok(m) if !m.is_empty() => m,
        _ => "claude-3-5-sonnet-20241022".into(),
    };

    let mut world = World::new(AllComponents {
        messages: MessagesComponent::default(),
        config: ConfigComponent {
            api_key: api_key.clone(),
            base_url: base_url.clone(),
            model: model.clone(),
            ..ConfigComponent::default()
        },
        loop_state: LoopComponent { step: 0, should_continue: true },
        tools: ToolsComponent {
            definitions: vec![echo::echo_def()],
        },
    });

    let mut queue: VecDeque<Event> = VecDeque::new();
    let mut tool_scheduler = ToolScheduler::new();
    tool_scheduler.register(echo::echo_def(), echo::echo_handler);
    tool_scheduler.register(bash::bash_def(), bash::bash_handler);
    tool_scheduler.register(read_file::read_file_def(), read_file::read_file_handler);
    tool_scheduler.register(write_file::write_file_def(), write_file::write_file_handler);
    tool_scheduler.register(edit_file::edit_file_def(), edit_file::edit_file_handler);
    tool_scheduler.register(glob::glob_def(), glob::glob_handler);
    let mut abort_manager = AbortManager::new();
    let pipeline = MiddlewarePipeline::new();

    let mut registry = SystemRegistry::<AllComponents>::new();
    registry.register(input::input_system::<AllComponents>, &["input.user"]);
    registry.register(provider::provider_system::<AllComponents>, &[
        "system.step_start",
        "tool.result",
        "tool.error",
    ]);
    registry.register(tool::tool_system::<AllComponents>, &["llm.tool_call"]);
    registry.register(
        output::output_system::<AllComponents>,
        &[
            "llm.thinking_delta",
            "llm.text_delta",
            "llm.tool_call",
            "llm.stream_end",
            "llm.stream_error",
        ],
    );
    registry.register(hook::hook_system::<AllComponents>, &["system.hook_trigger"]);

    let provider = AnthropicProvider::new(
        api_key.unwrap_or_default(),
        world.get::<ConfigComponent>().model.clone(),
        world.get::<ConfigComponent>().base_url.clone(),
    );

    loop {
        print!("You: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("/exit") || input.eq_ignore_ascii_case("/quit") {
            break;
        }
        if input.is_empty() {
            continue;
        }

        queue.push_back(Event::Input(InputEvent {
            source: "cli".into(),
            content: input.to_string(),
        }));

        print!("Alice: ");
        io::stdout().flush()?;

        while let Some(raw_event) = queue.pop_front() {
            if abort_manager.is_aborted() {
                break;
            }

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

        println!();

        if !world.get::<LoopComponent>().should_continue || abort_manager.is_aborted() {
            break;
        }
    }

    println!("[Alice session ended]");
    Ok(())
}
