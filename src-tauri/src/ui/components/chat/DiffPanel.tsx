import React from 'react';
import { Box, Group, Button, Title, Text, ActionIcon, ScrollArea } from '@mantine/core';
import { IconCheck, IconX, IconFileCode } from '@tabler/icons-react';
import ReactDiffViewer, { DiffMethod } from 'react-diff-viewer-continued';
import { useAppStore } from '../../store';
import { notifications } from '@mantine/notifications';
import { updateWikiPage, writeFile } from '../../api/tauri';

export const DiffPanel: React.FC = () => {
  const { currentProposal, setCurrentProposal } = useAppStore();

  if (!currentProposal) {
    return null;
  }

  const handleAccept = async () => {
    try {
      if (currentProposal.type === 'wiki') {
        const { wikiRoot, filePath, wikiMeta, newContent } = currentProposal;
        
        // filePath might be "my-wiki/new-page.md", we only need the relative part for the updateWikiPage API if it's already in the root path.
        // Actually, the API expects wikiRoot as the base path and filePath as the relative path.
        // If wikiRoot is the full path to my-wiki, then filePath should just be the filename or subpath.
        const relativePath = filePath.startsWith('my-wiki/') ? filePath.substring(8) : filePath;
        
        await updateWikiPage(wikiRoot || '', relativePath, wikiMeta || {}, newContent);
      } else {
        await writeFile(currentProposal.filePath, currentProposal.newContent);
      }

      notifications.show({
        title: '提案已采纳',
        message: `修改已应用到 ${currentProposal.filePath}`,
        color: 'green',
        icon: <IconCheck size={18} />
      });
      setCurrentProposal(null);
    } catch (error) {
      console.error("Failed to accept proposal:", error);
      notifications.show({
        title: '采纳失败',
        message: `无法保存修改: ${error}`,
        color: 'red',
        icon: <IconX size={18} />
      });
    }
  };

  const handleReject = () => {
    notifications.show({
      title: '提案已拒绝',
      message: '已丢弃该修改',
      color: 'gray',
      icon: <IconX size={18} />
    });
    setCurrentProposal(null);
  };

  return (
    <Box h="100%" display="flex" style={{ flexDirection: 'column' }}>
      <Box p="xs" style={{ borderBottom: '1px solid var(--mantine-color-default-border)' }}>
        <Text fw={600} size="sm" c="dimmed">DIFF PROPOSAL</Text>
      </Box>

      <Box p="md" style={{ borderBottom: '1px solid var(--mantine-color-default-border)', backgroundColor: 'var(--mantine-color-gray-0)' }}>
        <Group justify="space-between">
          <Group gap="xs">
            <IconFileCode size={18} color="var(--mantine-color-blue-filled)" />
            <Title order={6}>{currentProposal.filePath}</Title>
          </Group>
        </Group>
      </Box>

      <ScrollArea style={{ flex: 1 }} p={0}>
        <ReactDiffViewer
          oldValue={currentProposal.oldContent}
          newValue={currentProposal.newContent}
          splitView={false}
          compareMethod={DiffMethod.WORDS}
          styles={{
            variables: {
              light: {
                diffViewerBackground: 'transparent',
                diffViewerColor: 'inherit',
                addedBackground: '#e6ffed',
                addedColor: '#24292e',
                removedBackground: '#ffeef0',
                removedColor: '#24292e',
                wordAddedBackground: '#acf2bd',
                wordRemovedBackground: '#fdb8c0',
                addedGutterBackground: '#cdffd8',
                removedGutterBackground: '#ffdce0',
                gutterBackground: '#f7f7f7',
                gutterBackgroundDark: '#f3f1f1',
                highlightBackground: '#fffbdd',
                highlightGutterBackground: '#fff5b1',
                codeFoldGutterBackground: '#dbedff',
                codeFoldBackground: '#f1f8ff',
                emptyLineBackground: '#fafbfc',
                gutterColor: '#24292e',
                addedGutterColor: '#212529',
                removedGutterColor: '#212529',
                codeFoldContentColor: '#24292e',
                diffViewerTitleBackground: '#fafbfc',
                diffViewerTitleColor: '#24292e',
                diffViewerTitleBorderColor: '#eee',
              }
            }
          }}
        />
      </ScrollArea>

      <Group p="md" justify="flex-end" gap="sm" style={{ borderTop: '1px solid var(--mantine-color-default-border)' }}>
        <Button variant="default" onClick={handleReject} leftSection={<IconX size={16} />}>
          拒绝 (Reject)
        </Button>
        <Button color="green" onClick={handleAccept} leftSection={<IconCheck size={16} />}>
          采纳 (Accept)
        </Button>
      </Group>
    </Box>
  );
};
