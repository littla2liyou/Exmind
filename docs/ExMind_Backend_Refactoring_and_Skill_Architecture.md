# ExMind 后端重构与 Skill 架构设计文档

> 基于：`ExMind_PRD_深入版.md` (2026-04-17)
> 目标：将现有的 Tauri 后端重构为模块化、高内聚的系统，并设计支持“能力可插拔”的 Skill 架构。

---

## 1. 架构目标与重构原则

根据 PRD，ExMind MVP 放弃了 Git 版本控制和复杂的卡片盒流程，聚焦于**“本地文件管理 + AI 协作整理 + 简单版本快照”**。
为此，后端需要：
1. **明确领域边界**：将文件操作、Wiki 逻辑、配置管理、AI 调用严格分层。
2. **可插拔的 AI 能力 (Skill Architecture)**：避免在 Command 中硬编码不同的 AI 提示词和处理逻辑，采用统一的 Skill 注册与调度机制，为后续扩展（如摘要提取、翻译、结构化等）提供基础。
3. **轻量级版本管理**：实现基于文件系统的无 Git 版本快照机制。

---

## 2. 后端整体分层架构 (Architecture Layers)

建议采用以下分层结构：

```text
src-tauri/
├── src/
│   ├── main.rs
│   ├── lib.rs                  # 注册所有 Tauri Commands
│   ├── commands/               # Tauri API 层 (直接与前端交互)
│   │   ├── file_ops.rs
│   │   ├── wiki_ops.rs
│   │   ├── ai_proxy.rs         # 暴露聊天和 Skill 执行接口
│   │   └── settings.rs
│   ├── core/                   # 核心领域逻辑
│   │   ├── wiki/               # 页面 CRUD、版本快照、Frontmatter 解析
│   │   ├── workspace/          # 工作区文件树、初始化
│   │   └── skills/             # 【核心】Skill 引擎与具体技能实现
│   └── infra/                  # 基础设施层
│       ├── config/             # .exmind/config.json 读写
│       ├── llm/                # OpenAI/Anthropic 客户端封装
│       └── utils/              # 路径安全化、Diff 算法 (similar 库)
```

---

## 3. Skill 架构设计 (可插拔能力核心)

为了实现 PRD 中 6.2（整理到 Wiki）和 6.3（AI 自动整理）的能力，并保持未来扩展性，引入 `Skill` 架构。

### 3.1 核心概念

- **Skill (技能)**：一个实现了特定业务逻辑和 Prompt 构建的插件。
- **SkillRegistry (注册表)**：全局的技能管理器，启动时注册所有可用技能。
- **SkillContext (上下文)**：包含用户的指令、选中内容、目标路径等运行时信息。
- **SkillOutput (输出)**：标准化的输出格式，包含 AI 生成的完整内容、本地计算的 Diff、建议标题等。

### 3.2 核心 Rust Trait 定义

```rust
use async_trait::async_trait;
use serde_json::Value;

/// 技能执行上下文
pub struct SkillContext {
    pub instruction: Option<String>,
    pub original_content: String,
    pub source_path: Option<String>,
    pub target_wiki: Option<String>, // "my_wiki" | "agent_wiki"
}

/// 标准化技能输出
pub struct SkillOutput {
    pub proposed_content: String,
    pub diff: Option<String>,        // 后端基于 similar 库生成的文本差异
    pub title_suggestion: Option<String>,
    pub meta: Option<Value>,
}

#[async_trait]
pub trait Skill: Send + Sync {
    /// 技能唯一标识符 (例如: "organize_to_wiki", "auto_agent_extract")
    fn id(&self) -> &'static str;
    
    /// 技能展示名称
    fn name(&self) -> &'static str;
    
    /// 执行该技能的业务逻辑
    async fn execute(
        &self,
        context: SkillContext,
        llm_client: &dyn LLMProvider,
    ) -> Result<SkillOutput, String>;
}
```

### 3.3 Skill 注册与调度机制

```rust
use std::collections::HashMap;

pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: HashMap::new() }
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) {
        self.skills.insert(skill.id().to_string(), skill);
    }

    pub fn get_skill(&self, id: &str) -> Option<&Box<dyn Skill>> {
        self.skills.get(id)
    }
}
```

**MVP 阶段需内置的两个基础 Skill**：
1. **`WikiOrganizeSkill` (整理到 Wiki)**：接收原始片段和用户指令，生成目标页面的结构化 Markdown。
2. **`AutoAgentSkill` (AI 自动整理)**：批量读取文件，生成结构化的知识点，自动打上 `created_by: ai` 标签并输出到 Agent Wiki。

### 3.4 为什么 Diff 在后端本地生成？

PRD 7.3.2 提到：“建议让模型输出‘完整内容’，Diff 由本地生成”。
在 Skill 执行完毕得到 `proposed_content` 后，后端调用 `similar` 或 `text_diff` 库，与原内容对比生成 Diff 字符串返回给前端，**极大降低了对 LLM 输出格式稳定性的依赖**。

---

## 4. API 模块重规划 (Tauri Commands)

严格对齐 PRD 第 10 节的要求，重构现有的 Tauri Commands：

### 4.1 file_ops (工作区文件 CRUD)
- `list_dir(path: &str) -> Vec<FileInfo>`
- `read_file(path: &str) -> String`
- `write_file(path: &str, content: &str)`
- `create_file(path: &str, content: &str)`
- `delete_file(path: &str)`
- `rename(path: &str, new_path: &str)`

### 4.2 wiki_ops (Wiki 与版本管理)
Wiki 操作需自动处理 Frontmatter 和版本快照：
- `wiki_list_pages(wiki_type: &str) -> Vec<WikiPageMeta>`
- `wiki_get_page(wiki_type: &str, path: &str) -> PageData`
- `wiki_update_page(wiki_type: &str, path: &str, content: &str, meta: Option<Value>)` 
  - **核心逻辑**：写入新内容前，将旧内容保存到 `.exmind-mywiki/versions/<hash>/<timestamp>.md`。
- `wiki_get_history(wiki_type: &str, path: &str) -> Vec<VersionInfo>`
- `wiki_rollback(wiki_type: &str, path: &str, version_id: &str)`

### 4.3 ai_proxy (AI 与 Skill 调度)
- `ai_chat(messages: Vec<Message>, model: &str) -> Stream`：基础对话流。
- `list_skills() -> Vec<SkillInfo>`：获取已注册的可用技能。
- `execute_skill(skill_id: &str, context: SkillContext) -> SkillOutput`：执行指定技能并返回含 Diff 的提议内容。

### 4.4 settings_ops (配置管理)
- `init_workspace(root_path: &str)`：初始化 `.exmind`、`my-wiki`、`agent-wiki` 及配置文件。
- `get_config() -> AppConfig` / `update_config(config: AppConfig)`

---

## 5. 版本管理存储设计 (非 Git 方案)

根据 PRD 9，弃用 Git，采用轻量级文件系统快照：

1. **目录结构**：
   ```text
   .exmind-mywiki/versions/
     <page_path_hash>/
       2026-04-17T10-30-00Z.md
       2026-04-17T11-45-00Z.md
       manifest.json (记录每个版本的来源 user/ai，以及摘要)
   ```
2. **触发时机**：在 `wiki_update_page` 或用户在 Diff 界面点击 `Accept` 时，系统先将原文件内容以时间戳命名复制到 `versions/` 对应哈希目录下，再写入新文件。
3. **回滚机制**：`wiki_rollback` 读取对应时间戳的 `.md` 文件，覆盖当前工作区文件，并产生一次新的回滚版本快照。

---

## 6. 实施路线图 (Roadmap)

1. **阶段一：基础设施改造**
   - 建立 `src/core` 和 `src/infra` 目录结构。
   - 实现 `.exmind` 根目录初始化和配置文件的持久化读写。
2. **阶段二：Wiki 领域服务与版本管理**
   - 实现 Markdown Frontmatter 解析。
   - 实现无 Git 的时间戳版本快照机制与回滚功能。
3. **阶段三：Skill 引擎落地**
   - 引入 `async_trait`，定义 `Skill` 接口与 `SkillRegistry`。
   - 封装 LLM 客户端（OpenAI/Anthropic）。
   - 引入 `similar` 库，实现后端本地 Diff 生成逻辑。
4. **阶段四：API 桥接与 MVP 技能实现**
   - 编写 `WikiOrganizeSkill`（整理到 Wiki）和 `AutoAgentSkill`（自动整理）。
   - 将所有 Core 逻辑暴露为 Tauri Commands (`file_ops`, `wiki_ops`, `ai_proxy`)。
