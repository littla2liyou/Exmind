# ExMind PRD（深入版）

> 基于：`design.md`（2026-04-14）  
> 更新点：前端采用 **React + Mantine**；版本管理 **暂不引入 Git**；“卡片系统（Zettelkasten）”作为后续扩展能力，不纳入 MVP 核心范围。  
> 文档目的：用于产品/研发/设计评审，统一目标、范围、数据模型与 MVP 验收标准。  
> 日期：2026-04-17

---

## 0. 术语表

- **Workspace**：工作区，原始资料存放与浏览编辑区域（文件系统）。
- **My Wiki**：我的 Wiki，用户主导整理与确认的知识库。
- **Agent Wiki**：Agent Wiki，AI 生成的整理结果（候选知识库），需用户审计。
- **页面（Page）**：Wiki 中的一个 Markdown 文件（含元数据）。
- **Revision（版本）**：页面内容的历史快照（按时间戳保存）。
- **Diff**：AI 建议的改动差异展示（新增/删除/修改）。
- **LLM**：大语言模型（OpenAI/Anthropic 等）。

---

## 1. 背景与问题

目标用户（学习者/研究者/知识工作者）常见痛点：
1) 原始资料散落（PDF/MD/网页/笔记），难以沉淀到可复用的知识体系。  
2) 现有“第二大脑”工具往往鼓励“收集”，但难保证知识流经用户思考。  
3) AI 辅助整理很强，但直接自动写入会带来：幻觉、不可追溯、用户不再参与思考。  

**ExMind 的核心机会点**：用“可审阅 Diff + 人机分离知识库（My/Agent）”机制，让 AI 成为整理助手，而知识最终由用户确认沉淀为 Wiki。

---

## 2. 产品目标与非目标

### 2.1 产品目标（Goals）

1) 支持用户在本地文件系统中管理学习资料（Workspace）。  
2) 支持用户维护自己的 Markdown Wiki（My Wiki），并可在页面间跳转。  
3) 支持 AI 对话与 AI 提议改动（Diff），用户可 Accept/Reject。  
4) 支持知识流转：Workspace 选中内容 → AI 整理建议 → 写入 My Wiki 或 Agent Wiki。  
5) 支持基础版本管理：每次写入/接受变更可产生历史版本与回滚。  
6) 设计“LLM 友好”的内容结构与索引机制，保证后续可扩展更强的自动整理/链接/搜索能力。

### 2.2 非目标（Non-Goals，MVP 不做）

- 不引入 Git 作为默认版本系统（后续可选）。  
- 不做完整卡片盒（Zettelkasten）工作流（作为扩展方向）。  
- 不做复杂权限体系（本地单用户场景）。  
- 不做跨设备同步（后续可与 Git/云盘结合）。  

---

## 3. 目标用户与典型场景

### 3.1 目标用户

1) **系统学习者**：需要把课程/书籍/视频资料沉淀成体系化知识。  
2) **研究者/工程师**：希望从论文/文档中提炼结论并形成可检索 Wiki。  
3) **写作者/产品经理**：需要从材料中形成文章/方案，并保留来源与演化历史。

### 3.2 核心场景（Top Scenarios）

S1. 从资料到知识：打开 Workspace 文件 → 选择片段 → 让 AI 整理 → 审阅 Diff → 写入 My Wiki。  
S2. AI 批量候选：将资料直接“AI 自动整理”写入 Agent Wiki → 用户浏览审计 → 同步到 My Wiki。  
S3. Wiki 内编辑与回溯：编辑 My Wiki 页面 → 查看历史版本 → 回滚到某次版本。  

---

## 4. 信息架构与导航（IA）

### 4.1 顶层区域（MVP）

- Workspace（工作区）
- My Wiki（我的 Wiki）
- Agent Wiki（Agent Wiki）
- Settings（设置）

> 说明：卡片系统（Card Box）未来可作为独立入口，但 MVP 不引入第四入口。

### 4.2 三栏布局（MVP）

| 区域 | 位置 | 内容 |
|---|---|---|
| 左栏 | Tree/列表 | Workspace 文件树 或 当前 Wiki 的目录树/页面列表 |
| 中栏 | 阅读/编辑 | Markdown 渲染 + 编辑器（同屏切换或分屏） |
| 右栏 | AI 协作 | 对话 + Diff 预览 + Accept/Reject/写入目标选择 |

### 4.3 Mantine 组件策略（UI 实现约束）

MVP UI 以 Mantine 为基础，减少自研控件：
- Layout：`AppShell` / `Grid` / `ScrollArea`
- 导航与切换：`Tabs` / `SegmentedControl`
- 弹窗与确认：`Modal` / `Drawer` / `Confirm`
- 表单与设置：`TextInput` / `PasswordInput` / `Select` / `Switch`
- 提示：`Notifications` / `LoadingOverlay`

非 Mantine 重点控件（可用专用库）：
- 文件树：建议 `react-arborist` 或 `rc-tree`（支持大目录性能与交互）
- Diff：建议 `react-diff-viewer` 或 Monaco Diff（若编辑器用 Monaco）

---

## 5. MVP 功能范围

### 5.1 MVP 必做（Must Have）

1) 数据根目录选择与初始化（创建目录结构与配置）。  
2) Workspace 文件树浏览 + 读取 + 编辑（Markdown）。  
3) My Wiki 页面列表/目录树 + 页面创建/编辑/跳转。  
4) Agent Wiki 页面列表/浏览（可从 AI 写入）。  
5) AI 对话（流式输出）。  
6) AI 生成 Diff（基于原文 + 指令）。  
7) 用户审阅 Diff：Accept/Reject。  
8) 知识流转：Workspace →（AI）→ 写入 My/Agent Wiki。  
9) 版本管理（时间戳快照）：历史列表 + 回滚。  
10) 错误处理与可恢复（网络失败/文件失败/无 Key）。  

### 5.2 Should Have（MVP+1）

- Wiki 内部链接自动补全与跳转增强（至少支持 `[[...]]` 或 Markdown link）。  
- 基础搜索（按标题/全文）。  
- Backlinks（反向链接）索引与展示（强烈建议作为 MVP+1，LLM 价值极高）。  

### 5.3 Could Have（后续）

- Tag 系统（用于筛选/状态/领域）。  
- 卡片盒（Zettelkasten）思考层 + 结构页（MOC）工作流。  
- 跨 Wiki 智能同步（Agent → My 的半自动合并与冲突提示）。  
- 版本管理（历史、同步、协作）。  

---

## 6. 关键用户流程（MVP）

### 6.1 首次启动与初始化

1) 进入应用 → 若未配置根目录，弹出引导。  
2) 用户选择 `exmind-root/` 路径。  
3) 应用创建目录结构与 `.exmind/config.json`。  
4) 引导用户进入 Settings 配置 API Key（可跳过，但 AI 功能不可用）。  

验收：
- 根目录结构被创建；重复选择已存在根目录不破坏用户内容；缺失目录可自动补齐。

### 6.2 Workspace → My Wiki（“整理到 Wiki”）

1) 用户在 Workspace 打开文件并选中片段（或选择整个文件）。  
2) 点击「整理到 Wiki」。  
3) AI 在右侧生成：整理建议 + Diff（针对目标页面：新建或更新）。  
4) 用户在 Diff 中审阅，必要时编辑指令再次生成。  
5) 用户选择写入目标：My Wiki（默认）/Agent Wiki，选择目标路径与页面。  
6) Accept：写入目标页面；记录版本；在 UI 提示成功并可跳转到该页面。  
7) Reject：不写入，仅保留对话记录（可选）。  

关键定义（MVP 决策）：
- MVP 默认输出为“**页面级**”整理（生成/更新一个页面），不做“自动拆成多卡片”。  

### 6.3 Workspace → Agent Wiki（“AI 自动整理”）

1) 用户选中文件/内容 → 点击「AI 自动整理」。  
2) AI 生成一份整理结果（可不弹 Diff，或弹可选 Diff）。  
3) 写入 Agent Wiki，页面元数据标记 `created_by: ai`。  

验收：
- Agent Wiki 页面可浏览；并能从该页面触发“同步到 My Wiki”（MVP 可以是简单复制）。

### 6.4 版本管理（时间戳快照）

触发时机（MVP）：
- 当页面被更新（用户保存 / AI Accept 写入）时，产生新版本快照。

流程：
1) 用户在页面操作菜单选择「历史版本」。  
2) 展示版本列表（时间、来源：manual/ai、可选备注）。  
3) 选择两个版本查看 Diff；或选择某版本回滚。  

---

## 7. 功能需求（按模块）

### 7.1 Workspace 模块

#### 7.1.1 文件树与浏览

- 支持展开/折叠目录、点击打开文件。  
- 支持常见 Markdown 渲染（标题、列表、代码块、链接）。  
- 支持大目录性能：至少 5k 文件不卡死（可用虚拟滚动）。  

#### 7.1.2 编辑

- 支持编辑 Markdown 文件并保存。  
- 保存前检测写入权限与路径合法性。  

#### 7.1.3 选区整理

- 支持在阅读/编辑视图选择文本作为 AI 输入；  
- 若不支持精确选区，MVP 可退化为“整文件整理”。

---

### 7.2 Wiki 模块（My Wiki / Agent Wiki）

#### 7.2.1 页面列表与目录树

- 左侧展示页面目录（按文件夹/或按索引排序）。  
- 支持创建页面、重命名、删除（删除需二次确认）。  

#### 7.2.2 页面阅读与编辑

- 支持 Markdown 渲染与编辑；  
- 支持链接跳转（最小支持 Markdown 链接；`[[...]]` 可作为增强）。  

#### 7.2.3 My/Agent 差异展示（最小要求）

- 在页面顶部展示来源标记：`User` 或 `AI`（从 frontmatter 读取）。  
- Agent Wiki 页面提供“同步到 My Wiki”按钮（MVP 复制实现）。  

---

### 7.3 AI 协作模块

#### 7.3.1 AI 对话

- 支持流式输出；  
- 会话与页面/文件绑定（MVP：绑定当前打开内容即可）。  

#### 7.3.2 Diff 生成（核心）

输入：
- `original`（原文/目标页面内容；新建页面可为空）  
- `instruction`（用户指令）  
- `context`（来源信息：workspace path、选区、目标 wiki 等）  

输出（MVP）：
- `proposed_content`（完整新内容，便于写入与版本快照）  
- `diff`（用于 UI 展示）  
- `title_suggestion`（可选）  
- `target_path_suggestion`（可选）  

> 说明：MVP 可采用“生成完整内容 + 本地 diff 算法展示差异”的方式，降低模型输出格式要求。

#### 7.3.3 Accept/Reject

- Accept：写入目标页面；生成版本快照；写入 AI 生成元数据（created_by/updated_by/ai_model 等）。  
- Reject：不落盘；保留对话上下文（MVP 可选）。  

---

### 7.4 Settings 模块

- 配置 API Key（OpenAI/Anthropic 二选一或多选）。  
- 选择数据根目录。  
- 主题：浅色/深色（Mantine theme）。  

安全要求（MVP）：
- API Key 不进入 Wiki 内容文件；只保存在 `.exmind/config.json`（后续可做加密/系统钥匙串）。  

---

## 8. 数据与存储设计（LLM 友好）

### 8.1 根目录结构（MVP）

```text
exmind-root/
  workspace/
    ...用户原始文件...
    .exmind-workspace/config.json

  my-wiki/
    ...用户整理的 wiki 页面（.md）...
    .exmind-mywiki/
      config.json
      versions/               # 版本快照（按页面）
      index/                  # 索引缓存（MVP 可最小化）

  agent-wiki/
    ...AI 整理的 wiki 页面（.md）...
    .exmind-agentwiki/
      config.json
      versions/
      index/

  .exmind/
    config.json               # 全局配置（API Key、根目录标记等）
```

### 8.2 页面格式（建议，MVP 可逐步引入）

建议采用 Markdown + YAML frontmatter，便于：
- LLM 解析（元数据与正文分离）
- 后续索引（uid/title/type/source）

示例：

```md
---
uid: "202604171530"
title: "注意力是可训练的资源"
type: "topic"              # topic / reference / candidate
created_by: "user"         # user / ai
created_at: "2026-04-17T15:30:00Z"
updated_at: "2026-04-17T15:45:00Z"
source:
  kind: "workspace"
  path: "workspace/books/attention.md"
---

正文...
```

MVP 最小要求：
- 即使不引入 frontmatter，也必须能稳定定位页面（path）。  
建议 MVP 就引入 `created_by` 和 `source`，对审计 AI 内容非常关键。

### 8.3 索引缓存（MVP 最小化建议）

即便 MVP 不做搜索，也建议维护一个最小 `pages.json`：
- `path -> {title, updated_at, created_by}`  

原因：
- 页面列表更快；  
- LLM 后续做“推荐写入位置/查找相关页面”需要它；  
- 也方便实现重命名/移动后的维护（MVP 可先不做 uid）。  

---

## 9. 版本管理（不引入 Git 的 MVP 方案）

### 9.1 存储结构

```text
.exmind-mywiki/versions/
  <page_path_hash_or_safe_name>/
    2026-04-14-10-30.md
    2026-04-14-11-45.md
    manifest.json   # 可选：记录来源（ai/user）、备注、hash
```

关键点：
- 不建议用 symlink `latest`（跨平台风险）；用 `manifest.json` 或按时间排序取最新。  
- 页面标识建议用“路径安全化/哈希”避免目录名非法字符。

### 9.2 版本产生策略（MVP）

- 用户保存页面：产生版本（可配置：每次保存都存 or 节流）。  
- AI Accept 写入：必须产生版本，并写入 manifest 记录模型与指令摘要（便于审计）。  

---

## 10. API 与模块边界（Tauri Command 级别）

> 目标：明确前后端职责，保证可测试、可扩展。

### 10.1 file_ops（文件 CRUD）

- `list_dir(path) -> FileInfo[]`
- `read_file(path) -> string`
- `write_file(path, content) -> void`
- `create_file(path, content) -> void`
- `delete_file(path) -> void`
- `rename(path, new_path) -> void`（MVP 可选，但很实用）

### 10.2 wiki_ops（Wiki 逻辑）

- `wiki_list_pages(wiki: my|agent) -> WikiPageMeta[]`
- `wiki_get_page(wiki, path) -> {meta, content}`
- `wiki_update_page(wiki, path, content, metaPatch?) -> void`
- `wiki_create_page(wiki, path, content, meta) -> void`
- `wiki_get_history(wiki, path) -> Version[]`
- `wiki_rollback(wiki, path, versionId) -> void`

### 10.3 ai_proxy（AI 调用）

- `ai_chat(messages, model, options) -> stream`
- `ai_generate_proposal(original, instruction, context) -> {proposed_content, ...}`

> 注意：建议让模型输出“完整内容”，Diff 由本地生成，减少模型输出格式错误。

---

## 11. 非功能需求（NFR）

### 11.1 性能

- Workspace/ Wiki 文件树：5k 文件可用；滚动不卡顿（虚拟列表）。  
- 打开 1MB Markdown 文件渲染可用（可做延迟渲染/分段）。  

### 11.2 可靠性

- 所有写入操作必须 atomic（临时文件写入后替换）。  
- 写入失败要可恢复（提示 + 不破坏原文件）。  

### 11.3 安全与隐私

- 默认不上传任何文件内容（除非用户触发 AI 整理）。  
- AI 请求必须明确告知“将发送选中内容/文件内容到第三方模型”。  
- API Key 本地存储，不进入 Wiki 页面文件。  

### 11.4 可移植性

- 用户可直接用其他编辑器打开 `my-wiki/` 与 `agent-wiki/`（Markdown 可读）。  

---

## 12. 错误处理与提示（MVP）

| 场景 | 处理 |
|---|---|
| 未配置 API Key | 阻止 AI 调用，引导去 Settings |
| AI 调用失败/超时 | 展示错误 + 重试；保留原文不变 |
| 文件读写失败 | 明确提示路径/权限；不丢数据 |
| 根目录不可写 | 初始化失败提示；允许重新选择目录 |

---

## 13. 验收标准（MVP Checklist）

1) 应用可启动；首次启动能完成根目录初始化。  
2) Workspace：可浏览目录、打开 Markdown、编辑并保存。  
3) My Wiki：可创建/打开/编辑页面；页面间可通过链接跳转（至少支持 Markdown 链接）。  
4) Agent Wiki：可写入并浏览 AI 生成页面，页面标记为 AI。  
5) AI 对话：可发送与流式接收。  
6) Diff：可展示新增/删除；Accept/Reject 工作正常。  
7) 知识流转：Workspace 内容可经 AI 整理写入 My Wiki。  
8) 版本管理：My Wiki 页面可查看历史与回滚。  
9) Settings：API Key/根目录/主题设置可用。  
10) 错误场景：断网/无 key/无权限时有清晰提示且不破坏数据。  

---

## 14. 里程碑建议（按阶段）

> 你可按人力调整。这里按“能跑通端到端流程”为主线。

Phase 1（框架）：Tauri + React + Mantine + 三栏布局 + 路由/状态骨架  
Phase 2（本地文件）：file_ops + 文件树 + Markdown 渲染/编辑/保存  
Phase 3（Wiki）：my/agent 两套目录 + 页面 CRUD + 版本快照  
Phase 4（AI 协作）：ai_proxy + 流式 chat + proposal + diff + accept/reject  
Phase 5（知识流转）：Workspace → Wiki 写入闭环 + 错误处理完善  

---

## 15. 风险与开放问题（待确认）

1) 编辑器选型：CodeMirror（会影响 diff 与选区能力）。  
2) 链接规范：MVP 是否只支持 Markdown link，还是引入 `[[...]]`？  
3) 索引/Backlinks：作为 MVP+1 必做。  
4) AI 输出策略：允许模型输出 patch（patch 可靠性更难），进行局部修改确认，除非需要创建新文件。  

