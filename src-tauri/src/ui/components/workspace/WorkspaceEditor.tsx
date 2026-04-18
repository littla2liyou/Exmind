import React, { useState, useEffect, useRef } from 'react';
import { Box, Group, Button, Title, Text, ActionIcon, Tooltip } from '@mantine/core';
import { IconDeviceFloppy, IconEye, IconEdit, IconWand } from '@tabler/icons-react';
import Editor, { useMonaco } from '@monaco-editor/react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useAppStore } from '../../store';
import { notifications } from '@mantine/notifications';
import { MOCK_FILE_CONTENT } from '../../api/mockWorkspaceData';

interface WorkspaceEditorProps {
  filePath: string;
}

export const WorkspaceEditor: React.FC<WorkspaceEditorProps> = ({ filePath }) => {
  const [content, setContent] = useState('');
  const [isEditing, setIsEditing] = useState(true);
  const { selectedText, setSelectedText, setCurrentProposal } = useAppStore();
  const editorRef = useRef<any>(null);

  useEffect(() => {
    // Mock loading file content
    const loadedContent = MOCK_FILE_CONTENT[filePath] || '';
    setContent(loadedContent);
    // Determine default mode based on extension
    if (filePath.endsWith('.md')) {
      setIsEditing(false);
    } else {
      setIsEditing(true);
    }
  }, [filePath]);

  const handleEditorDidMount = (editor: any, monaco: any) => {
    editorRef.current = editor;

    // Listen to selection changes to capture selected text for AI context
    editor.onDidChangeCursorSelection((e: any) => {
      const selection = editor.getSelection();
      const model = editor.getModel();
      if (selection && model && !selection.isEmpty()) {
        const text = model.getValueInRange(selection);
        setSelectedText(text);
      } else {
        setSelectedText('');
      }
    });
  };

  const handleSave = () => {
    // Mock save logic
    if (editorRef.current) {
      const value = editorRef.current.getValue();
      MOCK_FILE_CONTENT[filePath] = value;
      setContent(value);
      notifications.show({
        title: '已保存',
        message: `文件 ${filePath} 保存成功`,
        color: 'green',
        icon: <IconDeviceFloppy size={18} />
      });
    }
  };

  const handleOrganizeToWiki = () => {
    if (!selectedText) {
      notifications.show({ title: '提示', message: '请先在编辑器中选中文本', color: 'yellow' });
      return;
    }
    
    // Mock triggering a Diff Proposal
    setCurrentProposal({
      id: Date.now().toString(),
      filePath: 'my-wiki/Organized_Notes.md',
      oldContent: '# AI 整理笔记\n这里是之前的内容。',
      newContent: `# AI 整理笔记\n这里是之前的内容。\n\n## 新增提取片段\n\`\`\`\n${selectedText}\n\`\`\`\n\nAI已对该代码进行了注释说明。`
    });
    
    notifications.show({
      title: 'AI 处理中',
      message: '正在将选中文本提取并整理到 Wiki 提案...',
      color: 'grape',
      icon: <IconWand size={18} />
    });
  };

  // Determine language for monaco
  let language = 'plaintext';
  if (filePath.endsWith('.js') || filePath.endsWith('.jsx')) language = 'javascript';
  if (filePath.endsWith('.ts') || filePath.endsWith('.tsx')) language = 'typescript';
  if (filePath.endsWith('.json')) language = 'json';
  if (filePath.endsWith('.md')) language = 'markdown';
  if (filePath.endsWith('.rs')) language = 'rust';
  if (filePath.endsWith('.toml')) language = 'toml';

  return (
    <Box h="100%" display="flex" style={{ flexDirection: 'column' }}>
      {/* Header toolbar */}
      <Group justify="space-between" p="xs" style={{ borderBottom: '1px solid var(--mantine-color-default-border)', backgroundColor: 'var(--mantine-color-body)' }}>
        <Title order={5} fw={500}>{filePath}</Title>
        <Group gap="xs">
          {selectedText && (
            <Button 
              size="compact-sm" 
              variant="light" 
              color="grape" 
              leftSection={<IconWand size={16} />} 
              onClick={handleOrganizeToWiki}
            >
              整理到 Wiki
            </Button>
          )}
          {filePath.endsWith('.md') && (
            <ActionIcon 
              variant={isEditing ? "light" : "filled"} 
              color="blue" 
              onClick={() => setIsEditing(!isEditing)}
              title={isEditing ? "切换到预览" : "切换到编辑"}
            >
              {isEditing ? <IconEye size={18} /> : <IconEdit size={18} />}
            </ActionIcon>
          )}
          <Button 
            size="compact-sm" 
            leftSection={<IconDeviceFloppy size={16} />} 
            onClick={handleSave}
            disabled={!isEditing}
          >
            保存
          </Button>
        </Group>
      </Group>

      {/* Editor Area */}
      <Box style={{ flex: 1, position: 'relative', overflow: 'auto' }}>
        {!isEditing && filePath.endsWith('.md') ? (
          <Box p="xl" className="markdown-body">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>
              {content}
            </ReactMarkdown>
          </Box>
        ) : (
          <Editor
            height="100%"
            language={language}
            theme="vs-light"
            value={content}
            onMount={handleEditorDidMount}
            options={{
              minimap: { enabled: false },
              wordWrap: 'on',
              fontSize: 14,
              fontFamily: 'ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace',
            }}
          />
        )}
      </Box>
    </Box>
  );
};
