# AGENTS.md

> 本文件面向 AI 编程助手。如果你刚拿到这个项目，请从这里开始了解 Alice 的整体结构、构建方式、代码约定与当前状态。

## 1. 项目概述

Alice 是一个**数据驱动、无状态、流式并发、高度可定制**的 CLI AI Agent 引擎。它的核心职责是编排 LLM 交互与工具调用：接收用户输入，驱动模型流式输出，捕获工具调用并执行，再把结果喂回模型，直到满足终止条件。

项目使用 **Rust** 实现，位于 `crates/` 目录下的 workspace 中。

根据当前文档与 `.devflow/state.toml`，当前进行中的工作流是 `REQ-38f782cc` —— "Complete Rust Rewrite and Remove TypeScript Implementation"。TypeScript 原型已移除，Rust 版本正在补齐完整的主循环、Provider SSE 解析、Middleware 等能力。

## 2. 技术栈

| 层级 | 技术 |
|---|---|
| 语言 | Rust（2021 edition） |
| Rust 运行时 | `tokio`（异步运行时） |
| Rust HTTP | `reqwest`（Provider 层） |
| Rust 序列化 | `serde` / `serde_json` |
| Rust 错误处理 | `thiserror`（库层）+ `anyhow`（CLI 层） |
| 工作流工具 | DevFlow（`.devflow/` 目录） |

Rust 侧依赖精简，无外部运行时依赖。

## 3. 仓库结构

```
alice/
├── crates/                       # Rust 实现
│   ├── alice-core/               # 核心引擎库
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs          # Message、ToolCall、ToolDef
│   │   │   ├── event.rs          # Event 枚举
│   │   │   ├── effect.rs         # Effect 枚举
│   │   │   ├── world.rs          # World<Components> + HasComponent trait + Snapshot
│   │   │   ├── event_bus.rs
│   │   │   ├── system.rs         # System trait + 函数指针 blanket impl
│   │   │   ├── system_registry.rs
│   │   │   ├── effect_executor.rs
│   │   │   ├── tool_scheduler.rs
│   │   │   ├── abort_manager.rs
│   │   │   ├── middleware.rs     # Middleware 管道（实现中）
│   │   │   ├── components.rs     # 数据组件定义
│   │   │   └── systems/          # input/provider/tool/output/hook
│   │   └── tests/                # 集成与单元测试
│   ├── alice-providers/          # Provider 实现
│   │   ├── src/
│   │   │   ├── traits.rs         # StreamingProvider trait
│   │   │   └── anthropic.rs      # AnthropicProvider（SSE 解析实现中）
│   ├── alice-tools/              # 内置工具
│   │   ├── src/
│   │   │   ├── traits.rs
│   │   │   └── echo.rs           # echo 工具示例
│   └── alice/                    # CLI 二进制入口
│       └── src/main.rs           # CLI 主循环（实现中）
│
├── docs/                         # 需求、设计、计划、证据、完成总结
│   ├── requirements/REQ-*.md
│   ├── features/FEAT-*.md
│   ├── superpowers/specs/DESIGN-*.md
│   ├── superpowers/plans/PLAN-*.md
│   ├── evidence/EVIDENCE-*.md
│   ├── completion/COMPLETION-*.md
│   └── alice-design.md           # 历史设计文档（TS 版，仅供参考）
│
├── .devflow/                     # DevFlow 工作流配置
│   ├── config.toml
│   ├── state.toml                # 当前工作流状态（gitignored）
│   ├── workflows/MODE-A.toml     # 特性开发工作流
│   ├── workflows/MODE-B.toml     # 调试工作流
│   └── prompts/                  # 各步骤 prompt 模板
│
├── Cargo.toml                    # Rust workspace 定义
└── Cargo.lock
```

## 4. 构建与测试命令

```bash
cargo build                    # 调试构建整个 workspace
cargo build --release          # 发布构建
cargo test                     # 运行所有 Rust 测试
cargo test --all-targets       # 包含集成测试
cargo clippy --all-targets     # 静态检查
```

### 当前验证状态（参考）

- `cargo test`：Rust 测试全部通过。
- `cargo clippy --all-targets`：零警告。
- `cargo build --release`：四个 crate 均成功编译。

## 5. 架构概述

### 五层模型（自上而下）

| 层 | 职责 | Rust 代表文件 |
|---|---|---|
| Host（装配层） | 初始化 World、注册 System、配置 Provider、安装 Middleware | `crates/alice/src/main.rs` |
| Core（核心层） | EventBus、EffectExecutor、SystemRegistry、ToolScheduler、AbortManager、World/Snapshot | `crates/alice-core/src/*.rs` |
| Business（业务层） | InputSystem、ProviderSystem、ToolSystem、OutputSystem、HookSystem | `crates/alice-core/src/systems/*.rs` |
| Data（数据层） | Messages、Provider、Tools、Config、Loop 等纯数据组件 | `crates/alice-core/src/components.rs` |
| Infrastructure（基础设施层） | StreamingProvider、ToolAccesses、Middleware | `crates/alice-providers/src/traits.rs`、`crates/alice-core/src/middleware.rs` |

### 三个核心抽象

1. **World / Snapshot**
   - `World<Components>` 通过 `HasComponent` trait 实现编译期类型安全，Snapshot 是不可变借用包装。

2. **System**
   - 纯函数：`Fn(&Snapshot, &Event) -> Vec<Effect>`。
   - 不持有状态、不直接修改 World、不产生实际副作用。

3. **Effect**
   - 副作用的"描述"而非副作用本身。七种变体：`callLLM`、`executeTool`、`appendMessage`、`updateComponent`、`emit`、`render`、`abort`。
   - 由 `EffectExecutor` 统一解释执行。

### 主循环数据流

```
用户输入 → InputSystem（appendMessage + emit step_start）
  → ProviderSystem（callLLM）
    → LLM 流式输出
      → OutputSystem（render 文本/思考块）
      → ToolSystem（executeTool）
        → ToolScheduler
          → tool.result 事件
            → ProviderSystem（下一轮 LLM 调用）
              → ... 循环直到 shouldContinue=false
```

### 四级扩展点（灵活度递增）

1. **Middleware**：事件管道拦截（日志、鉴权、限流）。
2. **Hook**：生命周期回调（`beforeStep`、`afterStep`、`beforeToolCall`、`afterToolCall`、`shouldContinue`）。
3. **Custom System**：自定义事件处理或覆盖内置行为。
4. **Custom Provider**：实现 `StreamingProvider` 接口接入任意 LLM。

## 6. 代码风格约定

- **注释与文档**：核心设计文档（`docs/alice-design.md`、`docs/superpowers/specs/DESIGN-*.md`）使用中文；代码注释中英文混合，关键设计点用中文说明。
- **无状态优先**：System 必须是纯函数，不要在其中保存闭包状态或修改外部变量。
- **Effect 描述化**：需要修改 World、调用 LLM、执行工具时，返回 Effect 让 `EffectExecutor` 执行，不要直接在 System 里产生副作用。
- **Rust 零警告**：`.devflow/config.toml` 约束 `zero_warnings = true`。提交前必须保证 `cargo clippy --all-targets` 无警告。
- **Rust 泛型组件**：不要引入运行时 `Box<dyn Any>` 或 downcast；组件通过 `HasComponent` trait 在编译期解析。

## 7. 测试策略

- 测试框架：**Cargo 内置测试**。
- 单元/集成测试位置：`crates/alice-core/tests/*.rs`、`crates/alice/tests/*.rs`。
- TDD 友好：System 是纯函数，可直接 `system.process(&snapshot, &event)` 测试。

### 验证流程

修改 Rust 代码后，按以下顺序验证：

```bash
cargo test --all-targets
cargo clippy --all-targets
cargo build --release
```

## 8. DevFlow 工作流

项目使用 **DevFlow** 进行结构化开发。相关配置在 `.devflow/` 目录。

### 常用命令

```bash
devflow current                 # 查看当前步骤
devflow done                    # 检查 gate 并前进
devflow back                    # 回退一步
devflow list-workflows          # 列出工作流
devflow select-workflow MODE-A  # 切换工作流
```

### 工作流模式

- **MODE-A**（`.devflow/workflows/MODE-A.toml`）：特性开发工作流，8 个阶段：需求 → 审批 → 头脑风暴 → 写计划 → 实现（SDD）→ 代码评审 → 测试 → 验证 → 完成。
- **MODE-B**（`.devflow/workflows/MODE-B.toml`）：调试工作流，4 个阶段 + 4.5 架构质疑阶段，失败 3 次后必须人工介入。

### 核心规则

1. 永远先 `devflow current` 再动手写代码。
2. 只能通过 `devflow done` 前进，不要跳步。
3. 不要为未来的步骤创建文件。
4. Gate 类型包括：`file_exists`、`file_contains`、`command_success`、`user_approved`、`state_set`。

## 9. 安全注意事项

- **Provider API Key**：Anthropic Provider 需要 API key。当前实现中需要在 Host 层传入，不要硬编码到源码或提交到仓库。
- **工具执行**：`ToolScheduler` 当前会同步执行 `ToolHandler`，未来计划区分只读工具与写工具以实现并发/互斥。新增工具时务必验证输入参数，避免命令注入或越权操作。
- **Rust 借用规则**：`EffectExecutor` 持有 `World` 的可变引用，处理 Effect 时需注意 Snapshot 与 World 的可变/不可变借用冲突。

## 10. 当前已知限制与未来工作

当前进行中的工作流 `REQ-38f782cc` 正在补齐以下能力：

- `EffectExecutor` 的 `CallLLM` effect 接入 Provider。
- Anthropic Provider 的 SSE 解析。
- CLI 二进制的 stdin/stdout 交互循环。
- Middleware 与完整 Hook 生命周期。
- DeepSeek Provider（后续可选）。

## 11. 给 AI 助手的快速上手清单

1. 先 `cargo test --all-targets` 与 `cargo clippy --all-targets` 确认基线。
2. 如果涉及 DevFlow：先 `devflow current`，严格按步骤执行，用 `devflow done` 推进。
3. 修改 System 时保持纯函数；修改 World 时通过 Effect。
4. 提交前确保 `cargo clippy` 零警告（项目硬性约束）。
