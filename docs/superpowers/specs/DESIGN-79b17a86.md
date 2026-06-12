---
id: DESIGN-79b17a86
title: Alice Rust 重写 — 架构设计方案
status: approved
parent: FEAT-79b17a86
---

# Alice Rust 架构设计

TS 版核心闭环：`Event → EventBus → System(Snapshot, Event) → Effect[] → EffectExecutor → 更新 World / 发射新事件 → EventBus`。本设计围绕如何在 Rust 中表达这三个核心抽象（World/Snapshot、System、Effect/Event）展开。

---

## 方案 A：最小迁移 — 保持 TS 架构骨架

沿袭 TS 版五层模型 + 三个核心抽象映射到 Rust trait/enum。唯一的"重设计"在于 World 用泛型替代动态组件映射。

### World：trait + 关联类型，手动 HasComponent 实现

不使用宏。用户定义组件集合 struct，手动为每个组件实现 `HasComponent` trait：

```rust
// 用户定义组件集合
#[derive(Default)]
struct AgentComponents {
    messages: MessagesComponent,
    config: ConfigComponent,
    loop_state: LoopComponent,
    tools: ToolsComponent,
    provider: ProviderComponent,
}

// 每个组件手动实现 HasComponent trait
impl HasComponent<MessagesComponent> for AgentComponents {
    fn get(&self) -> &MessagesComponent { &self.messages }
    fn get_mut(&mut self) -> &mut MessagesComponent { &mut self.messages }
}

// World<C> 泛型参数 C 是组件集合
let world = World::new(AgentComponents::default());
let snapshot = world.snapshot();
let msgs: &MessagesComponent = snapshot.get::<MessagesComponent>();
```

优势：零宏依赖、完全显式、IDE 友好（自动补全、跳转）。
代价：每个组件需要手工实现 4 行 trait 样板代码，但总量固定（5 个组件 ≈ 20 行）。

### System：函数指针 + 注册器

```rust
pub type SystemFn = fn(&Snapshot<C>, &Event) -> Vec<Effect>;

// System 注册时指定关心的事件类型
registry.register(InputSystem, &[EventType::Input]);
registry.register(ProviderSystem, &[EventType::StepStart, EventType::ToolResult]);
```

保持纯函数签名，和 TS 完全一致。

### Effect：声明式枚举 + 异步执行器

```rust
pub enum Effect {
    CallLLM { messages: Vec<Message> },
    ExecuteTool { name: String, args: Value },
    AppendMessage { entity: String, message: Message },
    Emit { event: Event },
    Render { content: String, stream: StreamType },
    Abort { reason: String },
}
// EffectExecutor 顺序执行，callLLM 通过 tokio 异步
```

### 流式桥接：Stream trait + EventBus 推送

LLM Provider 返回 `Pin<Box<dyn Stream<Item = Result<LLMStreamEvent>>>>`。EffectExecutor 的 callLLM 处理函数遍历 Stream 的每个 item，逐个 emit 到 EventBus。

### 优点
- 概念映射直接，TS 版文档和经验可复用
- 改动最小，风险最低
- 泛型 World 消除了 TS 版的 `ComponentDataMap` 和类型擦除问题

### 缺点
- Stream trait 返回 `Pin<Box<...>>` 有轻微分配开销
- macro_rules! 定义组件集合增加学习成本
- System 仍是全局函数注册，大型 Agent 的模块化受限

### 估时：2-3 天

---

## 方案 B：异步 Actor 模型 — System 作为独立 Task

每个 System 是一个 tokio task，通过 mpsc channel 接收事件、返回 Effect。

```rust
pub struct SystemTask {
    sender: mpsc::Sender<(Event, oneshot::Sender<Vec<Effect>>)>,
    handle: JoinHandle<()>,
}

// World 被 Arc<RwLock<...>> 包装，System 通过 snapshot() 读取
// 多个 System 可以真正并发执行
```

### 优点
- 原生异步并发，tokio 调度器自动优化 CPU 利用率
- System 可以有内部状态（如 ProviderSystem 缓存 HTTP client）
- 与 Rust 异步生态（tower、hyper）风格一致

### 缺点
- **破坏无状态契约**：System 持有 task 内部状态，违背原设计哲学
- 事件顺序难以保证（多个 System 并发处理同一事件）
- oneshot 往返增加了延迟和复杂性
- 调试困难 —— 分布式在多个 task 中，日志交错

### 估时：4-5 天

---

## 方案 C：流式管道 — Stream 组合器链

不用 EventBus。System 变成 `(Snapshot, Event) -> Stream<Effect>`，通过 Stream 组合器（map、flat_map、merge）链式连接。

```rust
// 整个引擎是一个 Stream 管道
let pipeline = event_stream
    .flat_map(|event| input_system.process(&snapshot, event))
    .flat_map(|effect| provider_system.process(&snapshot, event))
    // ...
    .for_each(|effect| execute(effect));
```

LLM 流式输出作为嵌套 Stream 通过 flatten 展开，天然流式。

### 优点
- 纯函数式，类型签名优美
- 天然流式 —— 从源头到终端全是 Stream
- 不需要 EventBus 中间层
- 符合 Rust Iterator/Stream 生态习惯

### 缺点
- **System 间耦合**：管道中的 System 顺序是硬编码的，无法动态路由
- 一个 Event 触发多个 System 时需要手动 fan-out（`stream.clone()` 或 `broadcast`）
- 工具调用结果重新注入管道困难（需要循环引用或外部 channel）
- Stream trait 在 Rust 中尚未稳定（需要 `async_stream` 或 `futures` crate）

### 估时：3-4 天

---

## 推荐

**方案 A**。理由：

1. **保真度**：TS 版的 System/Effect/Event 抽象经历了深思熟虑的设计，方案 A 直接继承这些设计决策，不引入新的概念风险
2. **可测试性**：纯函数 System 可以在测试中直接构造 `Snapshot` 和 `Event` 调用，不依赖任何运行时基础设施
3. **渐进扩展**：先让核心闭环跑通，后续如有需要可以在方案 A 基础上引入 Actor 或 Stream 优化

方案 B 和 C 各有过人之处，但都从根本上改变了 TS 版的设计哲学，风险高于收益。

---

## 最终决策

- **架构方案**：A（最小迁移）
- **World 组件存储**：手写 `HasComponent` trait，零宏依赖
- **运行时**：tokio
- **错误处理**：thiserror (lib) + anyhow (CLI)
- **Crate 结构**：alice-core + alice-providers + alice-tools + alice (bin)
- **流式接口**：`Pin<Box<dyn Stream<Item = Result<LLMStreamEvent>>>>`
