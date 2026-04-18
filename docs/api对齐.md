# API 对齐分析：PRD 要求 vs 当前后端实现

本文档对比了《ExMind PRD（深入版）》中第 10 节定义的 API 契约与当前 Rust 后端（`wikimind/commands` 及 `wikimind/api`）的实际实现情况。

## 1. 概览

* **匹配度**：极低（约 20%）。
* **核心矛盾**：前端 PRD 需要的是一组**“受控的 Wiki 业务接口（CRUD + 快照回滚）”**，而当前后端提供的是**“底层文件读写 + AI 智能体自由对话”**。
* **主要缺失**：完全缺失 `wiki_ops`（元数据解析与版本管理）相关的所有 API。

---

## 2. 文件操作 (File Ops)

**PRD 定位**：基础的文件系统浏览与操作。

| PRD 要求 API | 当前后端 Tauri Command | 状态 | 差异与缺失说明 |
| :--- | :--- | :--- | :--- |
| `list_dir(path)` | `list_dir(path)` | ✅ 已实现 | 基本对齐，返回 `FileInfo` 数组。 |
| `read_file(path)` | `read_file(path)` | ✅ 已实现 | 基本对齐，返回字符串。 |
| `write_file(path, content)`| `write_file(path, content)`| ✅ 已实现 | 基本对齐，无备份逻辑的底层覆写。 |
| `create_file(path, content)`| **无** | ❌ 缺失 | 需补充，或前端自行通过 `write_file` 替代（需后端检查路径是否存在）。 |
| `delete_file(path)` | **无** | ❌ 缺失 | 需补充，用于文件树的删除操作。 |
| `rename(path, new_path)` | **无** | ❌ 缺失 | MVP 可选，但对于整理 Wiki 极其重要。 |

---

## 3. Wiki 业务逻辑 (Wiki Ops)

**PRD 定位**：专门针对 My Wiki / Agent Wiki 的高阶 API，需处理 YAML Frontmatter 元数据和版本快照。

| PRD 要求 API | 当前后端 Tauri Command | 状态 | 差异与缺失说明 |
| :--- | :--- | :--- | :--- |
| `wiki_list_pages(wiki)` | **无** | ❌ 缺失 | 当前只能用 `list_dir`，无法解析 `title`、`created_by` 等页面元数据。 |
| `wiki_get_page(wiki, path)`| **无** | ❌ 缺失 | 当前只能用 `read_file`，无法分离元数据与 Markdown 正文。 |
| `wiki_create_page(...)` | **无** | ❌ 缺失 | 需要在创建时自动注入时间戳、来源等 Frontmatter 头部。 |
| `wiki_update_page(...)` | **无** | ❌ 缺失 | 这是 Accept 的核心，更新文件内容并要求**产生历史版本快照**。 |
| `wiki_get_history(path)` | **无** | ❌ 缺失 | 需要读取 `.exmind-mywiki/versions/` 目录获取快照列表。 |
| `wiki_rollback(...)` | **无** | ❌ 缺失 | 需要用指定的历史快照覆盖当前文件。 |

---

## 4. AI 代理与协作 (AI Proxy)

**PRD 定位**：AI 对话流与 Diff 提案生成。

| PRD 要求 API | 当前后端 Tauri Command | 状态 | 差异与缺失说明 |
| :--- | :--- | :--- | :--- |
| `ai_chat(...)` | `chat(messages)` | ⚠️ 偏离 | 已实现 SSE 流式输出，但当前 AI 挂载了 `Write/Edit` 工具，会**直接篡改磁盘文件**，不符合 PRD 要求的“分离生成”。 |
| `ai_generate_proposal(...)`| `update_wiki(...)` (Mock)| ❌ 缺失 | `update_wiki` 目前是占位符。缺失一个能根据选中内容和指令，纯粹返回新文本（提案）供前端 Diff 的接口。 |

---

## 5. 改造建议（基于“AI直写+缓存恢复”模式）

鉴于当前后端已经具备 AI 工具直写能力，如果要快速对齐 PRD 且不大量重构后端 Agent，建议调整 API 契约如下：

1. **改造 `chat` (或新增 `ai_edit_with_snapshot`)**:
   - 当 AI 决定调用 `Edit` 或 `Write` 工具修改文件前，**后端必须自动将原文件备份**到 `.exmind-mywiki/versions/[hash]_pre_ai.md`。
   - 修改完成后，通过 Tauri Event 通知前端 `{"event": "file_modified", "path": "...", "snapshot_id": "..."}`。

2. **新增 `wiki_get_diff(path, snapshot_id)`**:
   - 前端收到事件后，调用此接口，后端返回修改后的最新内容与备份快照的旧内容，前端渲染 Diff。

3. **新增 `wiki_resolve_proposal(path, snapshot_id, accept: boolean)`**:
   - **Accept = true**：后端将临时快照转正为正式的历史版本记录（`manifest.json` 标记为 user accepted ai edit）。
   - **Accept = false (Reject)**：后端将快照内容覆盖回原文件（回滚），并删除快照，达到“撤销 AI 修改”的效果。
