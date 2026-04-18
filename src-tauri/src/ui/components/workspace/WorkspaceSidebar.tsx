import React from 'react';
import { Box, Text } from '@mantine/core';
import { Tree, NodeRendererProps } from 'react-arborist';
import { IconFolder, IconFolderOpen, IconFileText } from '@tabler/icons-react';
import { useAppStore } from '../../store';
import { MOCK_WORKSPACE_DATA } from '../../api/mockWorkspaceData';

interface TreeNode {
  id: string;
  name: string;
  isDir?: boolean;
  children?: TreeNode[];
}

export const WorkspaceSidebar: React.FC = () => {
  const { currentOpenedFile, setCurrentOpenedFile } = useAppStore();

  const Node = ({ node, style, dragHandle }: NodeRendererProps<TreeNode>) => {
    const isDir = node.data.isDir || node.data.children;
    const isSelected = node.id === currentOpenedFile;

    return (
      <Box 
        style={{
          ...style,
          display: 'flex',
          alignItems: 'center',
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
    );
  };

  return (
    <Box h="100%" display="flex" style={{ flexDirection: 'column' }}>
      <Box p="xs" style={{ borderBottom: '1px solid var(--mantine-color-default-border)' }}>
        <Text fw={600} size="sm" c="dimmed">EXPLORER</Text>
      </Box>
      <Box style={{ flex: 1, padding: '8px' }}>
        <Tree<TreeNode>
          initialData={MOCK_WORKSPACE_DATA}
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
