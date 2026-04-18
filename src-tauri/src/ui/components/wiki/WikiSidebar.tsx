import React, { useState, useEffect } from 'react';
import { NavLink, ScrollArea, Box, Text, Loader, Center } from '@mantine/core';
import { IconFileText, IconFolder, IconFolderOpen } from '@tabler/icons-react';
import { listDir, readFile, createDir, FileInfo } from '../../api/tauri';
import { notifications } from '@mantine/notifications';

interface FileNode {
  id: string;
  name: string;
  isDir: boolean;
  children?: FileNode[];
  content?: string;
}

interface WikiSidebarProps {
  basePath: string;
  activeFileId: string | null;
  onFileSelect: (file: FileNode) => void;
  title: string;
}

export const WikiSidebar: React.FC<WikiSidebarProps> = ({ basePath, activeFileId, onFileSelect, title }) => {
  const [data, setData] = useState<FileNode[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (basePath) {
      loadDirectory(basePath);
    }
  }, [basePath]);

  const loadDirectory = async (dirPath: string) => {
    setLoading(true);
    try {
      const files = await listDir(dirPath);
      const treeData = buildTreeData(files);
      setData(treeData);
    } catch (error: any) {
      // If the directory does not exist, attempt to create it automatically
      if (String(error).includes('does not exist') || String(error).includes('系统找不到指定的文件') || String(error).includes('The system cannot find the file specified')) {
        try {
          await createDir(dirPath);
          // Try loading again after creation
          const files = await listDir(dirPath);
          const treeData = buildTreeData(files);
          setData(treeData);
        } catch (createError) {
          console.error(`Failed to auto-create wiki dir ${dirPath}:`, createError);
          notifications.show({ title: '初始化失败', message: `尝试自动创建Wiki目录失败: ${createError}`, color: 'red' });
        }
      } else {
        console.error(`Failed to load wiki dir ${dirPath}:`, error);
        notifications.show({ title: '错误', message: `无法读取Wiki目录: ${error}`, color: 'red' });
      }
    } finally {
      setLoading(false);
    }
  };

  const buildTreeData = (files: FileInfo[]): FileNode[] => {
    return files
      .sort((a, b) => {
        if (a.is_dir && !b.is_dir) return -1;
        if (!a.is_dir && b.is_dir) return 1;
        return a.name.localeCompare(b.name);
      })
      .map(file => ({
        id: file.path,
        name: file.name,
        isDir: file.is_dir,
        children: file.is_dir ? [] : undefined
      }));
  };

  const handleToggle = async (nodeId: string, isOpened: boolean) => {
    if (isOpened) {
      try {
        const files = await listDir(nodeId);
        const children = buildTreeData(files);
        
        const updateNode = (nodes: FileNode[]): FileNode[] => {
          return nodes.map(n => {
            if (n.id === nodeId) {
              return { ...n, children };
            }
            if (n.children) {
              return { ...n, children: updateNode(n.children) };
            }
            return n;
          });
        };
        
        setData(prev => updateNode(prev));
      } catch (error) {
        console.error(`Failed to load sub directory ${nodeId}:`, error);
      }
    }
  };

  const handleSelect = async (node: FileNode) => {
    if (node.isDir) return;
    try {
      const content = await readFile(node.id);
      onFileSelect({ ...node, content });
    } catch (error) {
      notifications.show({ title: '错误', message: `读取文件失败: ${error}`, color: 'red' });
    }
  };

  const renderTree = (
    nodes: FileNode[], 
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
            onChange={(opened) => handleToggle(node.id, opened)}
          >
            {node.children && renderTree(node.children, depth + 1)}
          </NavLink>
        );
      }

      return (
        <NavLink
          key={node.id}
          active={node.id === activeFileId}
          label={node.name}
          leftSection={<IconFileText size={16} stroke={1.5} />}
          onClick={() => handleSelect(node)}
          variant="light"
          color="blue"
        />
      );
    });
  };

  return (
    <Box h="100%" display="flex" style={{ flexDirection: 'column' }}>
      <Box p="xs" style={{ borderBottom: '1px solid var(--mantine-color-default-border)' }}>
        <Text fw={600} size="sm" c="dimmed">{title}</Text>
      </Box>
      <ScrollArea style={{ flex: 1 }}>
        <Box p="xs">
          {loading ? (
            <Center h={100}><Loader size="sm" color="gray" /></Center>
          ) : (
            renderTree(data)
          )}
        </Box>
      </ScrollArea>
    </Box>
  );
};
