import React, { useState, useEffect, useRef } from 'react';
import { Box, Group, Button, Title, Text, ActionIcon, Tooltip, Loader, Center } from '@mantine/core';
import { IconDeviceFloppy, IconEye, IconEdit, IconWand, IconCheck, IconX } from '@tabler/icons-react';
import Editor, { DiffEditor, useMonaco } from '@monaco-editor/react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useAppStore } from '../../store';
import { notifications } from '@mantine/notifications';
import { readFile, writeFile, executeSkill } from '../../api/tauri';

interface WorkspaceEditorProps {
  filePath: string;
}

export const WorkspaceEditor: React.FC<WorkspaceEditorProps> = ({ filePath }) => {
  const [content, setContent] = useState('');
  const [isEditing, setIsEditing] = useState(true);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [isOrganizing, setIsOrganizing] = useState(false);
  const { selectedText, setSelectedText, currentProposal, setCurrentProposal, workspaceDir, apiKey, rootDir } = useAppStore();
  const editorRef = useRef<any>(null);
  const diffEditorRef = useRef<any>(null);

  useEffect(() => {
    const loadFile = async () => {
      setLoading(true);
      try {
        const loadedContent = await readFile(filePath);
        setContent(loadedContent);
      } catch (error) {
        console.error("Failed to read file:", error);
        notifications.show({
          title: '读取失败',
          message: `无法读取文件: ${error}`,
          color: 'red'
        });
        setContent('');
      } finally {
        setLoading(false);
      }
    };

    loadFile();

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

  const handleDiffEditorDidMount = (editor: any, monaco: any) => {
    diffEditorRef.current = editor;
  };

  const handleAcceptProposal = async () => {
    if (currentProposal && currentProposal.filePath === filePath) {
      try {
        setSaving(true);
        await writeFile(filePath, currentProposal.newContent);
        setContent(currentProposal.newContent);
        setCurrentProposal(null);
        notifications.show({
          title: '已采纳',
          message: 'AI 的修改已应用到当前文件',
          color: 'green',
          icon: <IconCheck size={18} />
        });
      } catch (error) {
        console.error("Failed to write file:", error);
        notifications.show({
          title: '采纳失败',
          message: `无法写入文件: ${error}`,
          color: 'red'
        });
      } finally {
        setSaving(false);
      }
    }
  };

  const handleRejectProposal = () => {
    if (currentProposal && currentProposal.filePath === filePath) {
      setCurrentProposal(null);
      notifications.show({
        title: '已拒绝',
        message: '已取消 AI 的修改提议',
        color: 'gray',
        icon: <IconX size={18} />
      });
    }
  };

  const handleSave = async () => {
    if (editorRef.current) {
      const value = editorRef.current.getValue();
      try {
        setSaving(true);
        await writeFile(filePath, value);
        setContent(value);
        notifications.show({
          title: '已保存',
          message: `文件 ${filePath} 保存成功`,
          color: 'green',
          icon: <IconDeviceFloppy size={18} />
        });
      } catch (error) {
        console.error("Failed to save file:", error);
        notifications.show({
          title: '保存失败',
          message: `无法保存文件: ${error}`,
          color: 'red'
        });
      } finally {
        setSaving(false);
      }
    }
  };

  const handleOrganizeToWiki = async () => {
    if (!selectedText) {
      notifications.show({ title: '提示', message: '请先在编辑器中选中文本', color: 'yellow' });
      return;
    }

    setIsOrganizing(true);
    notifications.show({
      id: 'organize-wiki',
      title: 'AI 处理中',
      message: '正在生成 Wiki 页面...',
      color: 'grape',
      icon: <IconWand size={18} />,
      loading: true,
      autoClose: false
    });

    try {
      const response = await executeSkill({
        skill_id: 'organize_to_wiki',
        original_content: selectedText,
        source_path: filePath,
        target_wiki: 'my-wiki'
      });

      const newFileName = response.meta?.uid ? `${response.meta.uid}.md` : `new-page-${Date.now()}.md`;
      const wikiRootPath = workspaceDir || rootDir || '';
      
      setCurrentProposal({
        id: Date.now().toString(),
        filePath: `my-wiki/${newFileName}`,
        oldContent: '',
        newContent: response.proposed_content,
        type: 'wiki',
        wikiMeta: response.meta,
        wikiRoot: `${wikiRootPath}/my-wiki`
      });

      notifications.update({
        id: 'organize-wiki',
        title: '提案已生成',
        message: '请在 Diff 视图中审阅并确认',
        color: 'green',
        icon: <IconCheck size={18} />,
        loading: false,
        autoClose: 5000
      });
    } catch (error) {
      console.error("Failed to organize to wiki:", error);
      notifications.update({
        id: 'organize-wiki',
        title: '处理失败',
        message: `无法生成提案: ${error}`,
        color: 'red',
        icon: <IconX size={18} />,
        loading: false,
        autoClose: 3000
      });
    } finally {
      setIsOrganizing(false);
    }
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
              loading={isOrganizing}
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
            disabled={!isEditing || saving}
            loading={saving}
          >
            保存
          </Button>
        </Group>
      </Group>

      {/* Editor Area */}
      <Box style={{ flex: 1, position: 'relative', overflow: 'auto' }}>
        {loading ? (
          <Center h="100%">
            <Loader color="blue" />
          </Center>
        ) : currentProposal && currentProposal.filePath === filePath ? (
          <>
            <Box style={{ position: 'absolute', top: 10, right: 30, zIndex: 10 }}>
              <Group gap="sm">
                <Button size="xs" color="gray" onClick={handleRejectProposal} leftSection={<IconX size={14} />}>Reject</Button>
                <Button size="xs" color="green" onClick={handleAcceptProposal} leftSection={<IconCheck size={14} />}>Accept</Button>
              </Group>
            </Box>
            <DiffEditor
              height="100%"
              language={language}
              theme="vs-light"
              original={currentProposal.oldContent}
              modified={currentProposal.newContent}
              onMount={handleDiffEditorDidMount}
              options={{
                renderSideBySide: true,
                minimap: { enabled: false },
                wordWrap: 'on',
                fontSize: 14,
                fontFamily: 'ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace',
                readOnly: true, // Typically diff editors for proposals are read-only until accepted
              }}
            />
          </>
        ) : !isEditing && filePath.endsWith('.md') ? (
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
