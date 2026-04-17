// WikiMind Frontend Application

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// State
let currentFile = null;
let currentFolderPath = null;
let chatHistory = [];
let agentTypingElement = null;

// DOM Elements
const editor = document.getElementById('editor');
const fileList = document.getElementById('fileList');
const chatMessages = document.getElementById('chatMessages');
const chatInput = document.getElementById('chatInput');
const sendBtn = document.getElementById('sendBtn');
const styleModal = document.getElementById('styleModal');
const progressModal = document.getElementById('progressModal');
const styleSelect = document.getElementById('styleSelect');
const wordCount = document.getElementById('wordCount');
const saveStatus = document.getElementById('saveStatus');

// File Panel Toggle
document.getElementById('toggleFilePanel').addEventListener('click', () => {
  document.getElementById('filePanel').classList.toggle('collapsed');
});

// Chat Panel Toggle
document.getElementById('toggleChatPanel').addEventListener('click', () => {
  document.getElementById('chatPanel').classList.toggle('collapsed');
});

// Editor: Update word count
editor.addEventListener('input', () => {
  const text = editor.innerText || '';
  const count = text.replace(/\s/g, '').length;
  wordCount.textContent = `${count} 字`;

  // Check for /update command
  if (text.includes('/update')) {
    showStyleModal();
  }
});

// Save file
document.getElementById('btnSave').addEventListener('click', async () => {
  if (!currentFile) {
    alert('请先选择或创建一个文件');
    return;
  }
  try {
    const content = editor.innerText;
    await invoke('write_file', { path: currentFile, content });
    saveStatus.textContent = '已保存';
    setTimeout(() => { saveStatus.textContent = ''; }, 2000);
  } catch (err) {
    saveStatus.textContent = '保存失败';
    console.error(err);
  }
});

// New file
document.getElementById('btnNewFile').addEventListener('click', () => {
  const name = prompt('输入文件名:', 'untitled.md');
  if (name) {
    editor.innerText = '';
    editor.focus();
    currentFile = name;
  }
});

// Update Wiki button
document.getElementById('btnUpdateWiki').addEventListener('click', () => {
  showStyleModal();
});

// Open Folder button
document.getElementById('btnOpenFolder').addEventListener('click', async () => {
  try {
    const selected = await invoke('pick_folder');

    if (selected) {
      currentFolderPath = selected;
      console.log('Selected folder:', currentFolderPath);
      await loadFileList(currentFolderPath);
    }
  } catch (err) {
    console.error('打开文件夹失败:', err);
  }
});

// Style Modal
document.getElementById('cancelStyle').addEventListener('click', () => {
  styleModal.classList.remove('active');
});

document.getElementById('confirmStyle').addEventListener('click', async () => {
  const selectedStyle = document.querySelector('input[name="wikiStyle"]:checked')?.value || '技术风格';
  styleModal.classList.remove('active');
  await runUpdateWiki(selectedStyle);
});

function showStyleModal() {
  // Set current style
  const currentStyle = styleSelect.value;
  const radio = document.querySelector(`input[name="wikiStyle"][value="${currentStyle}"]`);
  if (radio) radio.checked = true;
  styleModal.classList.add('active');
}

// Close modal on background click
styleModal.addEventListener('click', (e) => {
  if (e.target === styleModal) styleModal.classList.remove('active');
});

progressModal.addEventListener('click', (e) => {
  e.stopPropagation();
});

// Chat: Send message
sendBtn.addEventListener('click', sendChatMessage);
chatInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') sendChatMessage();
});

async function sendChatMessage() {
  const text = chatInput.value.trim();
  if (!text) return;

  // Add user message
  addChatMessage('user', text);
  chatHistory.push({ role: 'user', content: text });
  chatInput.value = '';

  // Show agent typing indicator
  agentTypingElement = addChatMessage('agent', '...');

  try {
    const response = await invoke('chat', {
      messages: chatHistory,
      workspacePath: currentFolderPath
    });

    // Replace typing indicator with actual response
    if (agentTypingElement) {
      agentTypingElement.querySelector('.message-content').textContent = response.content;
      agentTypingElement.classList.remove('typing');
    }
    chatHistory.push({ role: 'assistant', content: response.content });
  } catch (err) {
    if (agentTypingElement) {
      agentTypingElement.querySelector('.message-content').textContent = `错误: ${err}`;
      agentTypingElement.classList.remove('typing');
    }
    console.error(err);
  }
}

function addChatMessage(role, content) {
  const div = document.createElement('div');
  div.className = `message ${role}`;
  if (content === '...') div.classList.add('typing');
  div.innerHTML = `<div class="message-content">${escapeHtml(content)}</div>`;
  chatMessages.appendChild(div);
  chatMessages.scrollTop = chatMessages.scrollHeight;
  return div;
}

// Listen for chat token events
listen('chat-token', (event) => {
  const { token } = JSON.parse(event.payload);
  if (agentTypingElement) {
    const content = agentTypingElement.querySelector('.message-content');
    content.textContent += token;
    chatMessages.scrollTop = chatMessages.scrollHeight;
  }
});

listen('chat-complete', () => {
  agentTypingElement = null;
});

// Update Wiki flow
async function runUpdateWiki(style) {
  styleSelect.value = style;
  progressModal.classList.add('active');

  try {
    // Get all markdown files from current directory
    const files = await listMarkdownFiles();

    // Start update_wiki
    const result = await invoke('update_wiki', {
      docs: files,
      style: style,
      workspacePath: currentFolderPath
    });

    // Show result
    editor.innerText = result.sections.map(s => `# ${s.title}\n\n${s.content}`).join('\n\n');

    addChatMessage('agent', `Wiki 已更新！生成了 ${result.sections.length} 个章节（${style}）`);
  } catch (err) {
    console.error(err);
    addChatMessage('agent', `Wiki 更新失败: ${err}`);
  } finally {
    progressModal.classList.remove('active');
  }
}

// Listen for wiki progress events
listen('wiki-progress', (event) => {
  const { phase, status, progress } = JSON.parse(event.payload);
  const progressFill = document.getElementById('progressFill');
  const progressText = document.getElementById('progressText');

  if (phase === 1 && status === 'reading') {
    progressFill.style.width = '20%';
    progressText.textContent = 'Phase 1: 阅读文档...';
  } else if (phase === 1 && status === 'outline_ready') {
    progressFill.style.width = '40%';
    progressText.textContent = 'Phase 1: 大纲已生成';
  } else if (phase === 2) {
    progressFill.style.width = `${40 + progress * 50}%`;
    progressText.textContent = `Phase 2: 生成内容 ${Math.round(progress * 100)}%`;
  } else if (phase === 3) {
    progressFill.style.width = '95%';
    progressText.textContent = 'Phase 3: 合并结果...';
  }
});

listen('wiki-complete', () => {
  const progressFill = document.getElementById('progressFill');
  const progressText = document.getElementById('progressText');
  progressFill.style.width = '100%';
  progressText.textContent = '完成！';
});

// File listing
async function loadFileList(folderPath) {
  if (!folderPath) {
    fileList.innerHTML = '<div class="loading">点击 📂 打开文件夹</div>';
    return;
  }

  try {
    const entries = await invoke('list_dir', { path: folderPath });
    const mdFiles = entries.filter(e => !e.is_dir && e.name.endsWith('.md'));

    if (mdFiles.length === 0) {
      fileList.innerHTML = `<div class="loading">文件夹: ${folderPath}<br>暂无 Markdown 文件</div>`;
      return;
    }

    fileList.innerHTML = `<div class="folder-path">📂 ${folderPath}</div>` +
      mdFiles.map(f => `
      <div class="file-item" data-path="${f.path}">
        <span class="file-icon">📄</span>
        <span class="file-name">${f.name}</span>
      </div>
    `).join('');

    // Add click handlers
    fileList.querySelectorAll('.file-item').forEach(item => {
      item.addEventListener('click', async () => {
        const path = item.dataset.path;
        await openFile(path);
      });
    });
  } catch (err) {
    fileList.innerHTML = `<div class="loading">加载失败: ${err}</div>`;
    console.error(err);
  }
}

async function openFile(path) {
  try {
    const content = await invoke('read_file', { path });
    editor.innerText = content;
    currentFile = path;

    // Highlight active file
    fileList.querySelectorAll('.file-item').forEach(item => {
      item.classList.toggle('active', item.dataset.path === path);
    });
  } catch (err) {
    console.error(err);
    alert(`打开文件失败: ${err}`);
  }
}

async function listMarkdownFiles() {
  if (!currentFolderPath) {
    return [];
  }
  try {
    const entries = await invoke('list_dir', { path: currentFolderPath });
    const mdFiles = entries
      .filter(e => !e.is_dir && e.name.endsWith('.md'))
      .map(f => f.path);
    return mdFiles;
  } catch (err) {
    console.error(err);
    return [];
  }
}

// Utilities
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Initialize - show prompt to open folder
loadFileList(null);
