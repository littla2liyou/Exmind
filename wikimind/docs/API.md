# WikiMind API 文档

## 概述

WikiMind 后端基于 Tauri 2 (Rust) 实现，提供文件系统操作和 AI 对话能力。

## 目录结构

```
wikimind/
├── commands/src/
│   ├── chat.rs          # chat 命令，AI 流式对话
│   ├── update_wiki.rs   # update_wiki 命令，生成 wiki
│   └── lib.rs          # 文件操作命令 (list_dir, read_file, write_file, pick_folder)
├── api/providers/
│   └── anthropic.rs    # Anthropic/MiniMaxi API 客户端
└── docs/
    └── API.md          # 本文档
```

---

## 文件操作 API

### list_dir

列出目录下的文件和文件夹。

```typescript
await invoke('list_dir', { path: string })
```

**参数**:
- `path`: 目录路径（绝对路径或相对路径）

**返回**:
```typescript
Array<{
  name: string,      // 文件名
  path: string,      // 完整路径
  is_dir: boolean,   // 是否为目录
  size: number       // 文件大小(字节)
}>
```

**筛选规则**:
- 跳过以 `.` 或 `_` 开头的隐藏文件
- 只返回目录和 `.md` 结尾的 Markdown 文件

---

### read_file

读取文件内容。

```typescript
await invoke('read_file', { path: string, offset?: number, limit?: number })
```

**参数**:
- `path`: 文件路径（必填）
- `offset`: 读取起始位置（可选，默认 0）
- `limit`: 读取最大字节数（可选）

**返回**: `string` - 文件内容

---

### write_file

写入内容到文件（覆盖式）。

```typescript
await invoke('write_file', { path: string, content: string })
```

**参数**:
- `path`: 文件路径
- `content`: 文件内容

**返回**: `boolean` - 是否成功

---

### pick_folder

打开系统文件夹选择对话框。

```typescript
await invoke('pick_folder')
```

**返回**: `string | null` - 用户选择的文件夹路径，取消返回 null

---

## AI 对话 API

### chat

与 AI 进行流式对话，支持 tool_use 工具调用。

```typescript
await invoke('chat', {
  messages: Array<{ role: string, content: string }>,
  workspacePath?: string
})
```

**参数**:
- `messages`: 对话历史，`role` 为 `user` 或 `assistant`
- `workspacePath`: 工作区路径（用于 system prompt 注入上下文）

**返回**:
```typescript
{
  content: string,           // AI 回复文本
  usage?: {                  // Token 使用量
    input_tokens: number,
    output_tokens: number
  },
  tools_used?: string[]      // 使用的工具列表
}
```

**可用工具**:
| 工具 | 功能 |
|------|------|
| `Read` | 读取文件内容 |
| `Write` | 创建/覆盖文件 |
| `Edit` | 替换文件中指定文本 |
| `ListDir` | 列出目录文件 |
| `Glob` | 按模式搜索文件 |
| `Grep` | 正则搜索文件内容 |

**SSE 事件**:
```typescript
// 每个 token 的流式事件
listen('chat-token', (event) => {
  const { token } = JSON.parse(event.payload);
  // token 是增量文本片段
});

// 对话完成事件
listen('chat-complete', (event) => {
  // 对话结束
});
```

---

## Wiki 生成 API

### update_wiki

从 my-notes 生成 Wiki 知识库。

```typescript
await invoke('update_wiki', {
  docs: string[],           // Markdown 文件路径列表
  style?: string,          // 风格: "技术风格" | "文艺风格" | "简洁风格" | "详细风格"
  workspacePath?: string,   // 工作区根目录
  diffMode?: boolean       // true=对比模式, false=AI生成模式(默认)
})
```

**参数**:
- `docs`: 要处理的 Markdown 文件路径数组
- `style`: Wiki 输出风格（默认 "技术风格"）
- `workspacePath`: 工作区路径，AI 会读取此路径下的 my-notes/ 目录
- `diffMode`:
  - `false`（默认）: AI 分析 my-notes 生成 wiki 内容
  - `true`: 对比 my-notes/ 和 wiki/ 目录差异

**返回**:
```typescript
{
  sections: Array<{
    title: string,   // 章节标题
    content: string  // 章节内容（Markdown 格式）
  }>
}
```

**SSE 事件**:
```typescript
// 进度更新
listen('wiki-progress', (event) => {
  const { phase, status, progress } = JSON.parse(event.payload);
  // phase: 1=读取文档, 2=生成内容, 3=合并结果
  // status: 当前状态描述
  // progress: 0.0-1.0 进度百分比
});

// Wiki 生成完成
listen('wiki-complete', (event) => {
  const { sections_count } = JSON.parse(event.payload);
});

// Diff 模式结果（仅 diffMode=true 时）
listen('wiki-diff-result', (event) => {
  const { files, summary } = JSON.parse(event.payload);
  // files: 文件差异列表
  // summary: 差异摘要
});
```

**文件写入位置**:
生成完成后，wiki 文件写入 `{workspacePath}/notes/{workspaceName}-wiki-notes/` 目录。

---

## 功能映射

根据 `功能设计梳理.md`，核心功能与 API 对应：

| 功能需求 | 实现 API |
|---------|---------|
| 打开工作区文件夹 | `pick_folder` |
| 列出笔记文件 | `list_dir` |
| 读取笔记内容 | `read_file` |
| 保存编辑内容 | `write_file` |
| AI 问答助手 | `chat` |
| 初始生成 Wiki（多 Agent 并行） | `update_wiki` (diffMode=false) |
| 对比 my-notes 与 wiki 差异 | `update_wiki` (diffMode=true) |
| 流式显示 AI 回复 | `chat-token` 事件 |

---

## 事件订阅示例

```typescript
import { listen } from '@tauri-apps/api/event';

// 订阅 chat token 流
const unlistenToken = await listen('chat-token', (event) => {
  const { token } = JSON.parse(event.payload);
  appendToChat(token);
});

// 订阅 wiki 进度
const unlistenProgress = await listen('wiki-progress', (event) => {
  const { phase, status, progress } = JSON.parse(event.payload);
  updateProgressUI(phase, status, progress);
});

// 取消订阅
unlistenToken();
unlistenProgress();
```

---

## 状态码与错误处理

所有 API 失败时返回 `String` 类型的错误信息。

常见错误:
- `"API error {status}: {body}"` - AI API 调用失败
- `"Failed to read file: ..."` - 文件读取失败
- `"Invalid tool input JSON: ..."` - 工具参数格式错误
