# ExMind Backend Implementation Plan (MVP)

> 基于: `ExMind_Backend_Refactoring_and_Skill_Architecture.md` (2026-04-18)

本文档旨在为开发人员提供**可执行的**、**分阶段的**后端开发计划。将高层架构转化为具体的代码模块、数据结构、第三方库选择和实施步骤。

---

## 1. 基础依赖与项目配置 (Phase 1)

**目标:** 搭建基础的 `src-tauri` 工程结构，引入必要的第三方库。

### 1.1 依赖引入 (`Cargo.toml`)
- **Serde**: `serde`, `serde_json` (数据序列化)
- **异步运行时**: `tokio` (Tauri 默认自带，需确认版本和 features)
- **文件操作**: `std::fs`, `std::path`
- **时间处理**: `chrono` (生成时间戳版本号)
- **Diff 算法**: `similar` (用于后端生成文本 Diff)
- **AI 客户端**: `reqwest` (HTTP 客户端调用 OpenAI/Anthropic API), `eventsource-stream` (SSE 流式解析)
- **Frontmatter 解析**: `gray_matter` (或手动解析 YAML block)

### 1.2 目录结构重构
将现有的单体/扁平结构重构为:
```text
src-tauri/src/
├── core/
│   ├── mod.rs
│   ├── config.rs      # .exmind/config.json 读写
│   ├── workspace.rs   # 工作区初始化、文件树
│   ├── wiki.rs        # Wiki 页面读写、Frontmatter、历史版本
│   └── skills/        # Skill 引擎
│       ├── mod.rs
│       ├── registry.rs
│       ├── trait.rs
│       └── builtin/   # 内置 Skills
├── infra/
│   ├── mod.rs
│   ├── llm.rs         # OpenAI/Anthropic API 封装
│   └── diff.rs        # 基于 similar 的 Diff 封装
├── commands/
│   ├── mod.rs
│   ├── file_ops.rs    # Tauri commands
│   ├── wiki_ops.rs
│   ├── ai_proxy.rs
│   └── settings.rs
├── main.rs
└── lib.rs             # 注册所有 commands
```

---

## 2. 核心模块实现 (Phase 2)

**目标:** 实现底层核心业务逻辑，不涉及 Tauri Command 绑定。

### 2.1 基础设施层 (Infra)
1. **`infra::diff`**:
   - 封装 `similar` 库，提供函数 `generate_diff(old: &str, new: &str) -> String`，返回标准 Unified Diff 格式。
2. **`infra::llm`**:
   - 定义 `LLMProvider` Trait。
   - 实现 `OpenAIClient` 和 `AnthropicClient` (MVP 阶段可先选定一种，或做一个通用封装)。
   - 支持普通对话和流式输出。

### 2.2 配置与工作区 (Core - Config & Workspace)
1. **`core::config`**:
   - 定义 `AppConfig` 结构体 (包含 API Key, root_path, theme 等)。
   - 实现加载和保存逻辑 (`~/.exmind/config.json` 或 `root_path/.exmind/config.json`，需根据 PRD 确定全局配置位置，PRD 倾向于 `exmind-root/.exmind/config.json`)。
2. **`core::workspace`**:
   - 实现 `init_workspace(path)`: 创建 `workspace/`, `my-wiki/.exmind-mywiki/versions`, `agent-wiki/.exmind-agentwiki/versions` 等必要目录。
   - 实现文件树遍历逻辑，支持过滤非 Markdown 文件。

### 2.3 Wiki 服务 (Core - Wiki)
1. **页面解析**:
   - 引入 `gray_matter` 解析 Markdown 中的 YAML Frontmatter，提取 `uid`, `title`, `created_by`, `source` 等。
2. **版本快照机制 (核心)**:
   - `save_version(wiki_type, path, content, meta)`: 将内容以当前时间戳 (如 `2026-04-18-10-30-00.md`) 写入 `.exmind-<wiki_type>/versions/<safe_path_or_hash>/`。
3. **页面 CRUD**:
   - `create_page`, `update_page` (调用 `save_version` 后覆写原文件), `get_page`, `list_pages`。
4. **回滚机制**:
   - `rollback(wiki_type, path, version_id)`: 读取特定版本，调用 `update_page` 覆盖当前，并产生新的回滚版本。

---

## 3. Skill 引擎与内置技能 (Phase 3)

**目标:** 实现可插拔的 Skill 架构。

### 3.1 Skill 核心 Trait (`core::skills::trait`)
定义 `Skill`, `SkillContext`, `SkillOutput`，如重构文档中所述。

### 3.2 Skill 注册表 (`core::skills::registry`)
实现 `SkillRegistry`，支持在应用启动时注册技能。

### 3.3 内置技能实现 (`core::skills::builtin`)
1. **`OrganizeToWikiSkill`**:
   - 组装 Prompt: 包含用户选中的 `original_content` 和 `instruction`。
   - 调用 LLM 获取完整 Markdown。
   - 调用 `infra::diff::generate_diff` 生成 Diff。
   - 返回 `SkillOutput`。
2. **`AutoAgentSkill`**:
   - 类似上者，但目标写入位置为 `agent-wiki`，并自动附加 `created_by: ai` 的 Frontmatter。

---

## 4. Tauri API 绑定层 (Phase 4)

**目标:** 将 Core 逻辑暴露给前端。

1. **`commands::file_ops`**: 绑定基础文件读写。
2. **`commands::wiki_ops`**: 绑定 `core::wiki` 的 CRUD 和版本操作。
3. **`commands::settings`**: 绑定配置读写和 `init_workspace`。
4. **`commands::ai_proxy`**:
   - `list_skills()`: 返回 `SkillRegistry` 中的技能列表。
   - `execute_skill(skill_id, context)`: 调用指定技能并返回结果。
   - `ai_chat()`: 暴露基础的流式对话能力。

---

## 5. 测试与验证 (Phase 5)

1. **单元测试 (Unit Tests)**:
   - 测试 `infra::diff` 算法是否准确。
   - 测试 Frontmatter 解析与合成是否破坏原有内容。
   - 测试版本快照目录是否正确生成。
2. **集成测试 (Integration Tests)**:
   - 模拟初始化一个空工作区，执行完整的 `OrganizeToWikiSkill` 流程，验证文件写入和 Diff 生成。

---

## 6. 数据结构参考 (供前后端联调)

### `WikiPageMeta`
```json
{
  "path": "my-wiki/books/attention.md",
  "title": "注意力是可训练的资源",
  "created_by": "user",
  "updated_at": "2026-04-17T15:45:00Z"
}
```

### `SkillOutput`
```json
{
  "proposed_content": "---\ntitle: ...\n---\n\n正文内容...",
  "diff": "--- old\n+++ new\n@@ -1,3 +1,4 @@\n...",
  "title_suggestion": "新标题建议",
  "meta": {
    "source_path": "workspace/raw/note.txt"
  }
}
```
