export const MOCK_MY_WIKI_DATA = [
  {
    id: 'my-wiki-root',
    name: 'My Wiki',
    isDir: true,
    children: [
      {
        id: 'mw-1',
        name: 'Project Ideas.md',
        isDir: false,
        content: `---
title: "Project Ideas for ExMind"
date: "2026-04-18"
tags: ["planning", "brainstorming"]
---

# ExMind Features to Build
1. **Workspace File Tree**: Real-time sync with local OS.
2. **Wiki Management**: A dual-wiki system.
3. **AI Chat & Diff Proposals**: AI generates diffs instead of direct writes.

This is a paragraph with *italic* and **bold** text.
Here is a list:
- React
- Rust
- Tauri
`
      },
      {
        id: 'mw-2',
        name: 'Daily Notes',
        isDir: true,
        children: [
          {
            id: 'mw-2-1',
            name: '2026-04-17.md',
            isDir: false,
            content: `---
title: "Notes: 2026-04-17"
date: "2026-04-17"
tags: ["daily"]
---
Today we successfully completed Phase 1 of the frontend UI.`
          }
        ]
      }
    ]
  }
];

export const MOCK_AGENT_WIKI_DATA = [
  {
    id: 'agent-wiki-root',
    name: 'Agent Wiki',
    isDir: true,
    children: [
      {
        id: 'aw-1',
        name: 'Architecture Review.md',
        isDir: false,
        content: `---
title: "ExMind Architecture Review"
date: "2026-04-18"
tags: ["ai-generated", "architecture"]
---
# Architecture Overview
The current system consists of a Tauri backend written in Rust, and a React frontend.
The Agent proposes a *snapshot-based* approach to handle AI modifications.

## Proposed Changes
- Implement \`.exmind-mywiki/versions/\`
- Intercept \`write_file\` calls from the AI.
`
      }
    ]
  }
];
