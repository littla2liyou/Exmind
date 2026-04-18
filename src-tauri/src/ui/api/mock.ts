import { FileInfo } from './tauri';

// 延迟模拟函数
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// ==========================================
// Wiki Ops (Mock)
// ==========================================

export interface WikiPageMeta {
  title: string;
  created_by: 'user' | 'ai';
  updated_at: string;
}

export interface VersionSnapshot {
  id: string;
  timestamp: string;
  source: 'manual' | 'ai';
  content: string;
}

/**
 * 模拟获取页面历史版本列表
 */
export async function mock_wikiGetHistory(path: string): Promise<VersionSnapshot[]> {
  await delay(500);
  return [
    {
      id: 'v2',
      timestamp: '2026-04-17T10:30:00Z',
      source: 'ai',
      content: '# 原始标题\n\n这是 AI 修改后的第二版内容。包含了更多的细节和段落。'
    },
    {
      id: 'v1',
      timestamp: '2026-04-16T09:00:00Z',
      source: 'manual',
      content: '# 原始标题\n\n这是用户最初手写的第一版内容。'
    }
  ];
}

/**
 * 模拟回滚到某个历史版本
 */
export async function mock_wikiRollback(path: string, versionId: string): Promise<void> {
  await delay(800);
  console.log(`[Mock] 成功将 ${path} 回滚到了版本 ${versionId}`);
}

// ==========================================
// AI Proxy (Mock)
// ==========================================

export interface AIProposal {
  proposed_content: string;
  diff_summary?: string;
}

/**
 * 模拟 AI 生成提案 (不写磁盘，只返回字符串用于 Diff 渲染)
 */
export async function mock_aiGenerateProposal(
  original: string,
  instruction: string,
  context: any
): Promise<AIProposal> {
  await delay(1500); // 模拟大模型思考时间
  
  // 简单的 Mock 逻辑：在原文基础上加点东西
  const proposed = original 
    ? `${original}\n\n---\n\n> **AI 补充整理**：根据您的指令 "${instruction}"，我提取了以下关键点...\n1. 核心观点一\n2. 核心观点二`
    : `# AI 自动生成的新页面\n\n根据您的指令 "${instruction}" 整理的内容。`;

  return {
    proposed_content: proposed,
    diff_summary: '新增了 AI 补充整理的段落'
  };
}

/**
 * 模拟审批 AI 提案 (Accept 触发快照，Reject 丢弃)
 */
export async function mock_wikiResolveProposal(
  path: string, 
  snapshot_id: string, 
  accept: boolean
): Promise<void> {
  await delay(600);
  if (accept) {
    console.log(`[Mock] 提案已 Accept！已更新 ${path} 并生成了历史版本记录。`);
  } else {
    console.log(`[Mock] 提案已 Reject。已回滚 ${path}。`);
  }
}
