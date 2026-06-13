use alice_core::components::{
    ConfigComponent, LoopComponent, MessagesComponent, ProviderComponent, ToolsComponent,
};
use alice_core::world::{HasComponent, World};

#[derive(Default)]
struct MainComponents {
    messages: MessagesComponent,
    config: ConfigComponent,
    loop_state: LoopComponent,
    tools: ToolsComponent,
    provider: ProviderComponent,
}

impl HasComponent<MessagesComponent> for MainComponents {
    fn get(&self) -> &MessagesComponent { &self.messages }
    fn get_mut(&mut self) -> &mut MessagesComponent { &mut self.messages }
}

impl HasComponent<ConfigComponent> for MainComponents {
    fn get(&self) -> &ConfigComponent { &self.config }
    fn get_mut(&mut self) -> &mut ConfigComponent { &mut self.config }
}

impl HasComponent<LoopComponent> for MainComponents {
    fn get(&self) -> &LoopComponent { &self.loop_state }
    fn get_mut(&mut self) -> &mut LoopComponent { &mut self.loop_state }
}

impl HasComponent<ToolsComponent> for MainComponents {
    fn get(&self) -> &ToolsComponent { &self.tools }
    fn get_mut(&mut self) -> &mut ToolsComponent { &mut self.tools }
}

impl HasComponent<ProviderComponent> for MainComponents {
    fn get(&self) -> &ProviderComponent { &self.provider }
    fn get_mut(&mut self) -> &mut ProviderComponent { &mut self.provider }
}

fn main() {
    let _world = World::new(MainComponents::default());
    println!("Alice Agent ready.");
}
