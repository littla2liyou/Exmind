# ExMind - 个人知识库沉淀应用

## 1. 项目概述

**项目名称**：ExMind  
**项目类型**：桌面端应用（Tauri + React）  
**核心定位**：新一代个人知识库沉淀应用，区分用户整理知识与 AI 整理知识，支持人机协作学习

**核心功能**：
- 工作区：原始文件资料存储（文件系统）
- 我的Wiki：用户主导整理的知识库
- Agent Wiki：AI 自动整理的知识库

**目标用户**：需要系统性学习、沉淀知识的学习者、研究人员

---

## 2. 技术选型

| 技术 | 选择 | 理由 |
|------|------|------|
| 桌面框架 | Tauri 2.x | 轻量、跨平台、直接访问文件系统 |
| 前端框架 | React 18+ | 生态成熟，组件丰富 |
| 状态管理 | Zustand | 轻量、简单、适合桌面应用 |
| 样式方案 | Tailwind CSS | 开发效率高 |
| AI 接入 | OpenAI API / Anthropic API | 用户提供 API Key |
| 存储 | 文件系统（Markdown） | 文件即知识库 |

---

## 3. 系统架构

```
┌─────────────────────────────────────────────────────┐
│                    ExMind 桌面端                     │
├─────────────────────────────────────────────────────┤
│  React 前端                                          │
│  ├── Layout (三栏布局)                                │
│  ├── WorkspacePanel (工作区)                         │
│  ├── WikiPanel (我的Wiki / Agent Wiki)              │
│  └── AIChatPanel (AI 对话 + Diff 展示)              │
├─────────────────────────────────────────────────────┤
│  Zustand 状态层                                      │
│  ├── fileStore (文件状态)                            │
│  ├── wikiStore (Wiki 状态)                          │
│  └── aiStore (AI 会话状态)                          │
├─────────────────────────────────────────────────────┤
│  Tauri Rust 后端                                    │
│  ├── file_ops (文件 CRUD)                           │
│  ├── ai_proxy (API 代理)                            │
│  └── version_ctrl (版本管理)                        │
└─────────────────────────────────────────────────────┘
```

---

## 4. 数据目录结构

```
exmind-root/           # 用户选择的根目录
├── workspace/         # 工作区 - 原始文件
│   └── .exmind-workspace/
│       └── config.json
├── my-wiki/          # 我的Wiki - 用户整理
│   └── .exmind-mywiki/
│       └── config.json
├── agent-wiki/       # Agent Wiki - AI 整理
│   └── .exmind-agentwiki/
│       └── config.json
└── .exmind/
    └── config.json   # 全局配置（API Key 等）
```

**说明**：
- 每个区域有独立目录，`.exmind-*/` 目录存放元数据
- 配置文件使用 JSON 格式
- 主要内容为 Markdown 文件

---

## 5. 核心模块设计

### 5.1 工作区（Workspace）

**职责**：管理原始学习资料

**功能**：
- 文件/文件夹浏览（树形结构）
- 文件内容查看（Markdown 渲染）
- 文件编辑（支持 Markdown）
- 选中文件/内容，发起 AI 整理

**关键 API**：
```typescript
// 前端
workspace.listDir(path: string): FileItem[]
workspace.readFile(path: string): string
workspace.writeFile(path: string, content: string): void
workspace.selectFiles(paths: string[]): Selection

// Rust 后端
list_dir(path) -> Vec<FileInfo>
read_file(path) -> String
write_file(path, content)
create_file(path, content)
delete_file(path)
```

### 5.2 Wiki 模块（我的Wiki / Agent Wiki）

**职责**：管理已整理的知识

**功能**：
- Wiki 页面浏览（支持链接跳转）
- 页面编辑
- 版本历史查看（简化版：记录修改时间）
- 知识流转：工作区内容导入

**版本管理（简化版）**：
```
.exmind-wiki/versions/
├── page-1/
│   ├── 2026-04-14-10-30.md
│   ├── 2026-04-14-11-45.md
│   └── latest -> 2026-04-14-11-45.md
└── page-2/
    └── ...
```

**关键 API**：
```typescript
wiki.listPages(): WikiPage[]
wiki.getPage(path: string): WikiPage
wiki.updatePage(path: string, content: string): void
wiki.getHistory(path: string): Version[]
wiki.rollback(path: string, versionId: string): void
```

### 5.3 AI 协作模块

**职责**：AI 对话、生成修改建议

**功能**：
- AI 对话（流式输出）
- 生成修改 Diff
- 用户审阅 Diff，accept/reject
- 知识流转：整理后写入目标 Wiki

**Diff 展示**：
```
+ 新增内容（绿色）
- 删除内容（红色）
```

**关键 API**：
```typescript
ai.sendMessage(prompt: string): StreamResponse
ai.generateDiff(original: string, instruction: string): DiffResult
ai.acceptDiff(diffId: string): void
ai.rejectDiff(diffId: string): void

// 知识流转
knowledge.importToWiki(source: string, targetWiki: 'my' | 'agent', targetPath: string): void
```

### 5.4 设置模块

**功能**：
- 配置 AI API Key
- 选择数据根目录
- 主题设置（浅色/深色）

---

## 6. UI 布局设计

```
┌──────────────────────────────────────────────────────────────────┐
│  ExMind                                    [工作区][Wiki][Agent] │
├──────────────┬─────────────────────────────────┬────────────────┤
│              │                                 │                │
│   文件树      │      主内容区                   │   AI 协作面板   │
│   (左栏)      │      (中央)                     │   (右栏)        │
│              │                                 │                │
│   250px      │      flex-1                     │   350px         │
│              │                                 │                │
│              │  - 文件内容 / Wiki 页面          │  - 对话框       │
│              │  - Markdown 渲染                 │  - Diff 展示    │
│              │  - 编辑器                        │  - 操作按钮     │
│              │                                 │                │
└──────────────┴─────────────────────────────────┴────────────────┘
```

**说明**：
- 左栏：文件树（工作区）或 Wiki 目录（Wiki 视图）
- 中央：内容区（文件查看/编辑/Wiki 页面）
- 右栏：AI 协作（对话 + Diff 展示 + 操作）

---

## 7. 知识流转流程

### 7.1 工作区 → 我的Wiki

```
1. 用户在工作区选中文件/内容
2. 点击「整理到Wiki」
3. AI 分析内容，生成整理建议
4. 用户选择目标位置：
   a. 跳转到现有相关页面
   b. 指定新位置
5. 用户审阅 AI 生成的文档
6. Accept → 写入我的Wiki，同时记录到 Agent Wiki
```

### 7.2 工作区 → Agent Wiki

```
1. 用户在工作区选中文件/内容
2. 点击「AI 自动整理」
3. AI 自动整理内容
4. 写入 Agent Wiki（标记为 AI 整理）
```

### 7.3 Agent Wiki → 我的Wiki

```
1. 用户浏览 Agent Wiki
2. 发现有价值的内容
3. 点击「同步到我的Wiki」
4. 复制到目标位置
```

---

## 8. MVP 功能范围

| 功能 | MVP | 后续 |
|------|-----|------|
| 三个区域切换 | ✓ | |
| 工作区文件浏览/编辑 | ✓ | |
| Wiki 页面浏览/编辑 | ✓ | |
| AI 对话 | ✓ | |
| AI 生成修改 Diff | ✓ | |
| 知识流转（工作区→Wiki） | ✓ | |
| 版本管理（仅时间戳） | ✓ | |
| 设置（API Key、根目录） | ✓ | |
| Agent Wiki 自动整理 | - | 后续版本 |
| 跨 Wiki 智能同步 | - | 后续版本 |
| 文件搜索 | - | 后续版本 |
| 标签系统 | - | 后续版本 |

---

## 9. 错误处理

| 场景 | 处理 |
|------|------|
| API Key 未配置 | 弹窗提示去设置 |
| API 调用失败 | 显示错误信息，提供重试 |
| 文件读写失败 | 显示错误，提示检查权限 |
| 网络超时 | 重试 3 次，提示用户 |
| 文件格式错误 | 提示不支持的格式 |

---

## 10. 开发计划

### Phase 1：基础框架
- Tauri 项目初始化
- React + Tailwind 环境搭建
- 基础布局实现

### Phase 2：核心功能
- 文件系统操作
- Wiki 页面管理
- AI 对话功能
- Diff 展示

### Phase 3：知识流转
- 工作区 → Wiki 导入
- 版本管理基础功能

### Phase 4：完善
- 设置功能
- UI 优化
- Bug 修复

---

## 11. 验收标准

- [ ] 应用可以正常启动
- [ ] 可以选择数据目录
- [ ] 工作区文件可以正常浏览和编辑
- [ ] 我的Wiki / Agent Wiki 可以正常切换和编辑
- [ ] AI 对话可以正常发送和接收
- [ ] Diff 可以正常展示和接受/拒绝
- [ ] 知识可以从工作区流转到 Wiki
- [ ] 版本历史可以查看

---

*设计日期：2026-04-14*