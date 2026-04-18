import { invoke } from '@tauri-apps/api/core';

// --- 类型定义 ---

export interface FileInfo {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
}

export interface ChatMessage {
  role: string;
  content: string;
}

export interface ChatUsage {
  input_tokens: number;
  output_tokens: number;
}

export interface ChatResponse {
  content: string;
  usage?: ChatUsage;
  tools_used?: string[];
}

// --- 真实的 Tauri Backend 调用 ---

/**
 * 列出指定目录下的文件和文件夹
 */
export async function listDir(path: string): Promise<FileInfo[]> {
  return await invoke<FileInfo[]>('list_dir', { path });
}

/**
 * 读取文件内容
 */
export async function readFile(path: string): Promise<string> {
  return await invoke<string>('read_file', { path });
}

/**
 * 写入文件内容
 */
export async function writeFile(path: string, content: string): Promise<void> {
  return await invoke<void>('write_file', { path, content });
}

/**
 * 创建文件夹
 */
export async function createDir(path: string): Promise<void> {
  return await invoke<void>('create_dir', { path });
}

/**
 * 删除文件或文件夹
 */
export async function deletePath(path: string): Promise<void> {
  return await invoke<void>('delete_path', { path });
}

/**
 * 发送聊天消息 (流式结果需要通过 Tauri Event 'chat-token' 监听)
 */
export async function chat(messages: ChatMessage[]): Promise<ChatResponse> {
  return await invoke<ChatResponse>('chat', { messages });
}
