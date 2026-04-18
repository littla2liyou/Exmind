# ExMind 前端 MVP 开发计划

基于《ExMind PRD（深入版）》与当前后端能力（以及规划的 API Mock/重构方向），制定本前端开发计划。采用“前端先行（Mock-first）”策略，确保在后端业务接口未完全就绪的情况下，前端仍能快速推进并跑通核心交互流。

---

## 阶段 0：基础设施与环境搭建 (Day 1)

**目标**：搭建好 React + Mantine 基础框架，配置好状态管理与 Tauri 通信层。

1. **项目初始化与依赖安装**
   - 确认 React 18 + Vite 模板已就绪。
   - 安装 Mantine (AppShell, Modal, Notifications, core 等)。
   - 安装 Zustand (用于全局状态管理：如当前选中的目录、打开的文件、对话历史等)。
   - 安装关键第三方组件：
     - 文件树：`react-arborist` (或 `rc-tree`)。
     - Diff 渲染：`react-diff-viewer-continued` (支持最新的 React 版本)。
     - Markdown 渲染：`react-markdown` + 语法高亮插件。
     - 编辑器：`@monaco-editor/react`。

2. **API Mock 层封装**
   - 建立 `src/api/` 目录。
   - 封装已有的 Tauri API (`list_dir`, `read_file`, `write_file`, `chat`)。
   - **建立 Mock 接口**：针对后端缺失的 Wiki 业务 API（如 `wiki_get_history`, `wiki_rollback` 以及带 Diff 提案的 `ai_generate_proposal`），在前端使用 `setTimeout` 和静态假数据（Hardcoded JSON/Markdown）进行模拟。

---

## 阶段 1：核心骨架与全局设置 (Day 2)

**目标**：实现 PRD 中定义的三栏布局骨架，以及首次启动的初始化流程。

1. **应用级路由与 Layout (AppShell)**
   - 实现 Mantine `AppShell` 结构。
   - 顶部/侧边 Navbar：包含 Workspace、My Wiki、Agent Wiki、Settings 导航入口。
   - 整体主题配置（Mantine Theme：深色/浅色支持）。

2. **全局设置模块 (Settings & Initialization)**
   - **首次引导**：检查是否配置了根目录，若无，弹出向导让用户选择 `exmind-root/`。
   - **Settings 面板**：API Key 配置输入框（本地存储）、根目录切换按钮。
   - 错误提示：当未配置根目录或无 API Key 时，触发相应的 Mantine Notification。

---

## 阶段 2：Workspace 模块 (Day 3-4)

**目标**：跑通原始文件的浏览与编辑，打通左栏与中栏。

1. **左栏：文件树组件 (Workspace View)**
   - 接入 `list_dir` API（或其 Mock），递归/按需加载目录结构。
   - 使用 `react-arborist` 渲染支持折叠、选中的树形控件。
   - 点击文件，将路径更新到 Zustand 全局状态（`currentOpenedFile`）。

2. **中栏：阅读与编辑视图**
   - **阅读模式**：使用 `react-markdown` 渲染选中的文件内容。
   - **编辑模式**：接入 Monaco Editor/CodeMirror。
   - 实现“保存”按钮或 `Ctrl+S` 快捷键，调用 `write_file` 保存更改。
   - **选区提取机制**：监听编辑器的选区事件（Selection），将选中的文本存入全局状态，为“整理到 Wiki”做准备。

---

## 阶段 3：Wiki 模块基础 (Day 5-6)

**目标**：搭建 My Wiki 和 Agent Wiki 的浏览与展示结构，并处理元数据展示。

1. **Wiki 文件列表/树结构**
   - 针对 `my-wiki/` 和 `agent-wiki/` 目录复用阶段 2 的文件树组件。
   - **前端元数据解析**：在 Mock 或正式 API 中，解析文件的 YAML Frontmatter。
   - 在文件列表或页面顶部展示来源标签（Badge）：`User` 或 `AI`。

2. **Wiki 页面内部导航**
   - 增强 Markdown 渲染，使其支持内部链接点击跳转。
   - 在 Agent Wiki 页面添加“同步到 My Wiki”的快速操作按钮（简单调用 Mock 的 `wiki_create_page` 或 `write_file` 复制过去）。

---

## 阶段 4：AI 协作与知识流转 (核心，Day 7-8)

**目标**：实现最核心的右栏对话与 Diff 审批流（Workspace -> My Wiki）。

1. **右栏：AI 对话面板**
   - 聊天界面（Message List + Input）。
   - 接入 `chat` API，处理 SSE 事件，实现流式打字机效果输出。

2. **核心业务流：整理到 Wiki (提案与 Diff)**
   - 中栏选中一段文本，点击“整理到 Wiki”按钮。
   - 调用 Mock 的 `ai_generate_proposal`，传入选中文本。
   - 右栏切换到 **Diff 视图**：
     - 左侧显示原文件内容（或空白），右侧显示 AI 生成的提案内容。
     - 使用 `react-diff-viewer-continued` 渲染高亮差异。
   - 底部渲染 **Accept** 和 **Reject** 按钮。
   - **操作处理**：
     - Accept：调用 Mock 的 `wiki_update_page`（这会触发后端快照机制），提示成功，清空 Diff 视图。
     - Reject：清除提案状态，恢复聊天面板。

---

## 阶段 5：版本管理与联调 (Day 9-10)

**目标**：完成快照回滚 UI，并与后端重构后的正式 API 进行全链路联调。

1. **版本历史视图 (History Modal/Drawer)**
   - 在 Wiki 页面的操作菜单中添加“历史版本”入口。
   - 弹出 Drawer，调用 Mock 的 `wiki_get_history` 展示时间戳列表。
   - 点击某历史版本，渲染该版本与当前版本的 Diff。
   - 添加“回滚到此版本”按钮（调用 `wiki_rollback` Mock）。

2. **前后端联调与去 Mock**
   - 将 `src/api/` 中的 Mock 实现逐步替换为真实的 Tauri `invoke`。
   - 配合后端调整参数结构（特别是前端依赖的“编辑前备份快照 -> Diff -> Accept 触发转正”的新流程）。
   - 跑通端到端的核心场景 S1、S2、S3。

3. **异常处理与边界测试**
   - 完善网络断开、文件被外部占用、API Key 欠费等情况下的全局错误提示（Mantine Notifications）。
   - 性能测试：加载大目录或长文档时的渲染表现优化。
