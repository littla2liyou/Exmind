import React from 'react';
import { NavLink, ScrollArea, Box, Text } from '@mantine/core';
import { IconFileText, IconFolder, IconFolderOpen } from '@tabler/icons-react';

interface FileNode {
  id: string;
  name: string;
  isDir: boolean;
  children?: FileNode[];
  content?: string;
}

interface WikiSidebarProps {
  data: FileNode[];
  activeFileId: string | null;
  onFileSelect: (file: FileNode) => void;
  title: string;
}

const renderTree = (
  nodes: FileNode[], 
  activeFileId: string | null, 
  onFileSelect: (file: FileNode) => void,
  depth = 0
): React.ReactNode => {
  return nodes.map((node) => {
    if (node.isDir) {
      return (
        <NavLink
          key={node.id}
          label={node.name}
          leftSection={<IconFolder size={16} stroke={1.5} />}
          childrenOffset={28}
          defaultOpened
        >
          {node.children && renderTree(node.children, activeFileId, onFileSelect, depth + 1)}
        </NavLink>
      );
    }

    return (
      <NavLink
        key={node.id}
        active={node.id === activeFileId}
        label={node.name}
        leftSection={<IconFileText size={16} stroke={1.5} />}
        onClick={() => onFileSelect(node)}
        variant="light"
        color="blue"
      />
    );
  });
};

export const WikiSidebar: React.FC<WikiSidebarProps> = ({ data, activeFileId, onFileSelect, title }) => {
  return (
    <Box h="100%" display="flex" style={{ flexDirection: 'column' }}>
      <Box p="xs" style={{ borderBottom: '1px solid var(--mantine-color-default-border)' }}>
        <Text fw={600} size="sm" c="dimmed">{title}</Text>
      </Box>
      <ScrollArea style={{ flex: 1 }}>
        <Box p="xs">
          {renderTree(data, activeFileId, onFileSelect)}
        </Box>
      </ScrollArea>
    </Box>
  );
};
