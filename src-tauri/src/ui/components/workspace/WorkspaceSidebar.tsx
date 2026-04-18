import React, { useState } from 'react';
import { Box, Text, ActionIcon, Group, Tooltip, Menu } from '@mantine/core';
import { Tree, NodeRendererProps } from 'react-arborist';
import { IconFolder, IconFolderOpen, IconFileText, IconFolderPlus, IconFilePlus, IconDotsVertical, IconTrash } from '@tabler/icons-react';
import { useAppStore } from '../../store';
import { MOCK_WORKSPACE_DATA } from '../../api/mockWorkspaceData';
import { notifications } from '@mantine/notifications';

interface TreeNode {
  id: string;
  name: string;
  isDir?: boolean;
  children?: TreeNode[];
}

export const WorkspaceSidebar: React.FC = () => {
  const { currentOpenedFile, setCurrentOpenedFile, workspaceDir, setWorkspaceDir } = useAppStore();
  const [data, setData] = useState<TreeNode[]>(MOCK_WORKSPACE_DATA);

  const handleOpenFolder = async () => {
    // In real app, call Tauri dialog.open
    const mockNewPath = "E:/code/another-project";
    setWorkspaceDir(mockNewPath);
    notifications.show({ title: '已打开文件夹', message: `当前工作区：${mockNewPath}`, color: 'blue' });
  };

  const handleCreateFile = () => {
    notifications.show({ title: '新建文件', message: '由于目前为 Mock 数据，功能正在开发中', color: 'yellow' });
  };

  const handleCreateFolder = () => {
    notifications.show({ title: '新建文件夹', message: '由于目前为 Mock 数据，功能正在开发中', color: 'yellow' });
  };

  const handleDelete = (id: string) => {
    notifications.show({ title: '删除项', message: `删除 ${id}，Mock暂不支持实际删除`, color: 'red' });
  };

  const Node = ({ node, style, dragHandle }: NodeRendererProps<TreeNode>) => {
    const isDir = node.data.isDir || node.data.children;
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
                <Menu.Item leftSection={<IconFilePlus size={14} />} onClick={handleCreateFile}>新建文件</Menu.Item>
                <Menu.Item leftSection={<IconFolderPlus size={14} />} onClick={handleCreateFolder}>新建文件夹</Menu.Item>
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
    <Box h="100%" display="flex" style={{ flexDirection: 'column' }}>
      <Group justify="space-between" p="xs" style={{ borderBottom: '1px solid var(--mantine-color-default-border)' }}>
        <Text fw={600} size="sm" c="dimmed">
          {workspaceDir ? workspaceDir.split('/').pop()?.split('\\').pop() : 'EXPLORER'}
        </Text>
        <Group gap={2}>
          <Tooltip label="新建文件">
            <ActionIcon size="sm" variant="subtle" color="gray" onClick={handleCreateFile}><IconFilePlus size={16} /></ActionIcon>
          </Tooltip>
          <Tooltip label="新建文件夹">
            <ActionIcon size="sm" variant="subtle" color="gray" onClick={handleCreateFolder}><IconFolderPlus size={16} /></ActionIcon>
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
      </Box>
    </Box>
  );
};
