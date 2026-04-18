import React, { useState, useEffect } from 'react';
import { Box, Text, ActionIcon, Group, Tooltip, Menu, Loader, Center, Modal, TextInput, Button, Stack } from '@mantine/core';
import { Tree, NodeRendererProps } from 'react-arborist';
import { IconFolder, IconFolderOpen, IconFileText, IconFolderPlus, IconFilePlus, IconDotsVertical, IconTrash } from '@tabler/icons-react';
import { useAppStore } from '../../store';
import { listDir, FileInfo, createDir, writeFile, deletePath } from '../../api/tauri';
import { invoke } from '@tauri-apps/api/core';
import { notifications } from '@mantine/notifications';

interface TreeNode {
  id: string;
  name: string;
  isDir?: boolean;
  children?: TreeNode[];
}

export const WorkspaceSidebar: React.FC = () => {
  const { currentOpenedFile, setCurrentOpenedFile, workspaceDir, setWorkspaceDir } = useAppStore();
  const [data, setData] = useState<TreeNode[]>([]);
  const [loading, setLoading] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [modalType, setModalType] = useState<'file' | 'folder'>('file');
  const [newItemName, setNewItemName] = useState('');
  const [targetParentPath, setTargetParentPath] = useState<string | null>(null);

  useEffect(() => {
    if (workspaceDir) {
      loadDirectory(workspaceDir);
    }
  }, [workspaceDir]);

  const loadDirectory = async (dirPath: string) => {
    setLoading(true);
    try {
      const files = await listDir(dirPath);
      const treeData = buildTreeData(files, dirPath);
      setData(treeData);
    } catch (error) {
      console.error("Failed to load directory:", error);
      notifications.show({
        title: '错误',
        message: `无法读取目录: ${error}`,
        color: 'red'
      });
    } finally {
      setLoading(false);
    }
  };

  const buildTreeData = (files: FileInfo[], parentPath: string): TreeNode[] => {
    return files
      .sort((a, b) => {
        // Folders first
        if (a.is_dir && !b.is_dir) return -1;
        if (!a.is_dir && b.is_dir) return 1;
        return a.name.localeCompare(b.name);
      })
      .map(file => ({
        id: file.path,
        name: file.name,
        isDir: file.is_dir,
        children: file.is_dir ? [] : undefined // Empty array indicates it's a directory that can be loaded
      }));
  };

  const handleOpenFolder = async () => {
    try {
      const selectedPath = await invoke<string | null>('pick_folder');
      if (selectedPath) {
        setWorkspaceDir(selectedPath);
        notifications.show({ title: '已打开文件夹', message: `当前工作区：${selectedPath}`, color: 'blue' });
      }
    } catch (error) {
      console.error("Failed to pick folder:", error);
    }
  };

  const openCreateModal = (type: 'file' | 'folder', parentPath?: string) => {
    setModalType(type);
    setTargetParentPath(parentPath || workspaceDir);
    setNewItemName('');
    setModalOpen(true);
  };

  const submitCreate = async () => {
    if (!newItemName.trim() || !targetParentPath) return;
    
    // Simple path join (assuming windows uses \ and others / but let's use / as generic and fallback to \ if needed)
    // To be safe, we can just append a generic separator. In JS we can check if path contains \
    const separator = targetParentPath.includes('\\') ? '\\' : '/';
    const newPath = `${targetParentPath}${separator}${newItemName.trim()}`;

    try {
      if (modalType === 'file') {
        await writeFile(newPath, '');
        notifications.show({ title: '成功', message: `文件已创建`, color: 'green' });
        setCurrentOpenedFile(newPath);
      } else {
        await createDir(newPath);
        notifications.show({ title: '成功', message: `文件夹已创建`, color: 'green' });
      }
      
      // Refresh tree (either entirely or just the parent)
      // For simplicity, just reload the workspace if parent is root, or we can just reload root
      if (workspaceDir) {
        loadDirectory(workspaceDir);
      }
    } catch (error) {
      notifications.show({ title: '创建失败', message: `${error}`, color: 'red' });
    } finally {
      setModalOpen(false);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deletePath(id);
      notifications.show({ title: '成功', message: `已删除`, color: 'green' });
      if (currentOpenedFile === id) {
        setCurrentOpenedFile(null);
      }
      if (workspaceDir) {
        loadDirectory(workspaceDir);
      }
    } catch (error) {
      notifications.show({ title: '删除失败', message: `${error}`, color: 'red' });
    }
  };

  const onToggle = async (id: string, isOpen: boolean) => {
    if (isOpen) {
      try {
        const files = await listDir(id);
        const children = buildTreeData(files, id);
        
        // Recursive function to update the tree node
        const updateNode = (nodes: TreeNode[]): TreeNode[] => {
          return nodes.map(node => {
            if (node.id === id) {
              return { ...node, children };
            }
            if (node.children) {
              return { ...node, children: updateNode(node.children) };
            }
            return node;
          });
        };
        
        setData(prev => updateNode(prev));
      } catch (error) {
        console.error(`Failed to load directory ${id}:`, error);
        notifications.show({ title: '错误', message: `无法读取子目录`, color: 'red' });
      }
    }
  };

  const Node = ({ node, style, dragHandle }: NodeRendererProps<TreeNode>) => {
    const isDir = node.data.isDir || node.data.children !== undefined;
    const isSelected = node.id === currentOpenedFile;

    return (
      <Box 
        style={{
          ...style,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          cursor: 'pointer',
          padding: '0 8px',
          backgroundColor: isSelected ? 'var(--mantine-color-blue-light)' : 'transparent',
          color: isSelected ? 'var(--mantine-color-blue-filled)' : 'inherit',
          borderRadius: '4px',
          userSelect: 'none'
        }}
        onClick={() => {
          if (isDir) {
            node.toggle();
            onToggle(node.id, !node.isOpen);
          } else {
            setCurrentOpenedFile(node.id);
          }
        }}
        ref={dragHandle}
      >
        <Box style={{ display: 'flex', alignItems: 'center', overflow: 'hidden' }}>
          <Box mr={6} style={{ display: 'flex', alignItems: 'center' }}>
            {isDir ? (
              node.isOpen ? <IconFolderOpen size={16} stroke={1.5} /> : <IconFolder size={16} stroke={1.5} />
            ) : (
              <IconFileText size={16} stroke={1.5} />
            )}
          </Box>
          <Text size="sm" truncate>
            {node.data.name}
          </Text>
        </Box>
        <Menu shadow="md" width={150} position="right-start" withArrow>
          <Menu.Target>
            <ActionIcon 
              size="sm" 
              variant="subtle" 
              color="gray"
              onClick={(e) => {
                e.stopPropagation();
              }}
              style={{ opacity: 0, transition: 'opacity 0.2s' }}
              className="node-actions"
            >
              <IconDotsVertical size={14} />
            </ActionIcon>
          </Menu.Target>
          <Menu.Dropdown onClick={(e) => e.stopPropagation()}>
            {isDir && (
              <>
                <Menu.Item leftSection={<IconFilePlus size={14} />} onClick={() => openCreateModal('file', node.id)}>新建文件</Menu.Item>
                <Menu.Item leftSection={<IconFolderPlus size={14} />} onClick={() => openCreateModal('folder', node.id)}>新建文件夹</Menu.Item>
                <Menu.Divider />
              </>
            )}
            <Menu.Item color="red" leftSection={<IconTrash size={14} />} onClick={() => handleDelete(node.id)}>删除</Menu.Item>
          </Menu.Dropdown>
        </Menu>
      </Box>
    );
  };

  return (
    <>
      <Box h="100%" display="flex" style={{ flexDirection: 'column' }}>
      <Group justify="space-between" p="xs" style={{ borderBottom: '1px solid var(--mantine-color-default-border)' }}>
        <Text fw={600} size="sm" c="dimmed">
          {workspaceDir ? workspaceDir.split('/').pop()?.split('\\').pop() : 'EXPLORER'}
        </Text>
        <Group gap={2}>
          <Tooltip label="新建文件">
            <ActionIcon size="sm" variant="subtle" color="gray" onClick={() => openCreateModal('file')}><IconFilePlus size={16} /></ActionIcon>
          </Tooltip>
          <Tooltip label="新建文件夹">
            <ActionIcon size="sm" variant="subtle" color="gray" onClick={() => openCreateModal('folder')}><IconFolderPlus size={16} /></ActionIcon>
          </Tooltip>
          <Tooltip label="打开文件夹">
            <ActionIcon size="sm" variant="subtle" color="gray" onClick={handleOpenFolder}><IconFolderOpen size={16} /></ActionIcon>
          </Tooltip>
        </Group>
      </Group>
      <Box style={{ flex: 1, padding: '8px', overflow: 'hidden' }}>
        <style>{`
          .node-actions { opacity: 0; }
          div[role="treeitem"]:hover .node-actions { opacity: 1; }
        `}</style>
        {loading ? (
          <Center h="100%">
            <Loader size="sm" color="gray" />
          </Center>
        ) : (
          <Tree<TreeNode>
            initialData={data}
            width="100%"
            height={600}
            rowHeight={30}
            indent={16}
            padding={4}
          >
            {Node}
          </Tree>
          )}
        </Box>
      </Box>

      <Modal opened={modalOpen} onClose={() => setModalOpen(false)} title={`新建${modalType === 'file' ? '文件' : '文件夹'}`}>
        <Stack>
          <TextInput 
            label="名称" 
            placeholder={modalType === 'file' ? '例如: index.ts' : '例如: src'}
            value={newItemName}
            onChange={(e) => setNewItemName(e.currentTarget.value)}
            onKeyDown={(e) => e.key === 'Enter' && submitCreate()}
            autoFocus
          />
          <Button onClick={submitCreate}>创建</Button>
        </Stack>
      </Modal>
    </>
  );
};
