import React, { useEffect, useState } from 'react';
import { AppShell, Burger, Group, NavLink, Title, Text, Button, Modal, TextInput, Stack, Center, Box, Tooltip, UnstyledButton } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { IconFolder, IconBook, IconRobot, IconSettings } from '@tabler/icons-react';
import { useAppStore } from './store';
import { notifications } from '@mantine/notifications';
import { WikiSidebar } from './components/wiki/WikiSidebar';
import { WikiViewer } from './components/wiki/WikiViewer';
import { WorkspaceSidebar } from './components/workspace/WorkspaceSidebar';
import { WorkspaceEditor } from './components/workspace/WorkspaceEditor';
import { MOCK_MY_WIKI_DATA, MOCK_AGENT_WIKI_DATA } from './api/mockWikiData';

function App() {
  const [opened, { toggle }] = useDisclosure();
  const { rootDir, apiKey, setRootDir, setApiKey, activeTab, setActiveTab, currentWikiFile, setCurrentWikiFile, currentOpenedFile, setCurrentOpenedFile } = useAppStore();
  const [setupModalOpened, setSetupModalOpened] = useState(false);
  const [tempDir, setTempDir] = useState('');

  // 检查是否初始化
  useEffect(() => {
    if (!rootDir) {
      setSetupModalOpened(true);
    }
  }, [rootDir]);

  const handleSetupComplete = () => {
    if (!tempDir.trim()) {
      notifications.show({
        title: '错误',
        message: '根目录不能为空',
        color: 'red'
      });
      return;
    }
    setRootDir(tempDir);
    setSetupModalOpened(false);
    notifications.show({
      title: '初始化成功',
      message: '欢迎使用 ExMind',
      color: 'green'
    });
  };

  const navItems = [
    { icon: <IconFolder size="1.4rem" stroke={1.5} />, label: 'Workspace', id: 'workspace' },
    { icon: <IconBook size="1.4rem" stroke={1.5} />, label: 'My Wiki', id: 'my-wiki' },
    { icon: <IconRobot size="1.4rem" stroke={1.5} />, label: 'Agent Wiki', id: 'agent-wiki' },
    { icon: <IconSettings size="1.4rem" stroke={1.5} />, label: 'Settings', id: 'settings' },
  ] as const;

  return (
    <>
      <AppShell
        header={{ height: 60 }}
        navbar={{
          width: 60, // Shrink to activity bar size
          breakpoint: 'sm',
          collapsed: { mobile: !opened },
        }}
        aside={{ width: 300, breakpoint: 'md', collapsed: { desktop: false, mobile: true } }}
        padding={0} // Changed from "md" to 0 to manage layout cleanly
      >
        <AppShell.Header>
          <Group h="100%" px="md">
            <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
            <Title order={3}>ExMind</Title>
          </Group>
        </AppShell.Header>

        <AppShell.Navbar p={0} style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', paddingTop: '16px' }}>
          {navItems.map((item) => (
            <Tooltip key={item.id} label={item.label} position="right" transitionProps={{ duration: 0 }}>
              <UnstyledButton
                onClick={() => {
                  setActiveTab(item.id);
                  if (item.id !== 'my-wiki' && item.id !== 'agent-wiki') setCurrentWikiFile(null);
                  if (item.id !== 'workspace') setCurrentOpenedFile(null);
                }}
                style={{
                  width: '48px',
                  height: '48px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  borderRadius: '8px',
                  color: activeTab === item.id ? 'var(--mantine-color-blue-filled)' : 'var(--mantine-color-gray-6)',
                  backgroundColor: activeTab === item.id ? 'var(--mantine-color-blue-light)' : 'transparent',
                  marginBottom: '8px',
                  transition: 'background-color 0.2s',
                }}
              >
                {item.icon}
              </UnstyledButton>
            </Tooltip>
          ))}
        </AppShell.Navbar>

        <AppShell.Main h="100vh" pt={60} display="flex">
          {activeTab === 'settings' ? (
            <Box p="md" w="100%">
              <Stack gap="lg" maw={500}>
                <Title order={2}>设置</Title>
                <TextInput 
                  label="数据根目录 (Root Directory)" 
                  description="Workspace 与 Wiki 文件的存储位置"
                  value={rootDir || ''} 
                  onChange={(e) => setRootDir(e.currentTarget.value)}
                />
                <TextInput 
                  label="API Key" 
                  description="输入用于大语言模型的 API Key"
                  placeholder="sk-..."
                  value={apiKey || ''} 
                  onChange={(e) => setApiKey(e.currentTarget.value)}
                  type="password"
                />
                <Button onClick={() => notifications.show({ title: '保存成功', message: '设置已更新' })}>
                  保存设置
                </Button>
              </Stack>
            </Box>
          ) : activeTab === 'my-wiki' || activeTab === 'agent-wiki' ? (
            <Group align="flex-start" wrap="nowrap" gap={0} h="100%" w="100%">
              {/* Secondary Sidebar for Wiki File List */}
              <Box w={250} h="100%" style={{ borderRight: '1px solid var(--mantine-color-default-border)', backgroundColor: 'var(--mantine-color-body)' }}>
                <WikiSidebar 
                  title={activeTab === 'my-wiki' ? 'My Wiki' : 'Agent Wiki'}
                  data={activeTab === 'my-wiki' ? MOCK_MY_WIKI_DATA : MOCK_AGENT_WIKI_DATA}
                  activeFileId={currentWikiFile?.id || null}
                  onFileSelect={setCurrentWikiFile}
                />
              </Box>
              {/* Markdown Viewer */}
              <Box style={{ flex: 1, height: '100%', backgroundColor: 'var(--mantine-color-gray-0)' }}>
                {currentWikiFile ? (
                  <WikiViewer 
                    content={currentWikiFile.content || ''} 
                    isAgentWiki={activeTab === 'agent-wiki'}
                    onSyncToMyWiki={() => {
                      notifications.show({ title: '同步成功', message: '已复制到 My Wiki' });
                    }}
                  />
                ) : (
                  <Center h="100%">
                    <Text c="dimmed">请在左侧选择一个 Wiki 页面</Text>
                  </Center>
                )}
              </Box>
            </Group>
          ) : activeTab === 'workspace' ? (
            <Group align="flex-start" wrap="nowrap" gap={0} h="100%" w="100%">
              {/* Secondary Sidebar for Workspace File Tree */}
              <Box w={250} h="100%" style={{ borderRight: '1px solid var(--mantine-color-default-border)', backgroundColor: 'var(--mantine-color-body)' }}>
                <WorkspaceSidebar />
              </Box>
              {/* Code/Markdown Editor Viewer */}
              <Box style={{ flex: 1, height: '100%', backgroundColor: 'var(--mantine-color-body)' }}>
                {currentOpenedFile ? (
                  <WorkspaceEditor filePath={currentOpenedFile} />
                ) : (
                  <Center h="100%">
                    <Text c="dimmed">请在左侧选择一个文件进行编辑</Text>
                  </Center>
                )}
              </Box>
            </Group>
          ) : (
            <Center h="100%" w="100%">
              <Text c="dimmed">这里是 {activeTab} 的主阅读/编辑区域（中栏）。</Text>
            </Center>
          )}
        </AppShell.Main>

        {activeTab !== 'settings' && (
          <AppShell.Aside p="md" pt={76}>
            <Title order={4} mb="md">AI 协作</Title>
            <Text size="sm" c="dimmed">
              右栏：这里将展示对话界面与 Diff 审批视图。
            </Text>
          </AppShell.Aside>
        )}
      </AppShell>

      {/* 初始化向导弹窗 */}
      <Modal
        opened={setupModalOpened}
        onClose={() => {}}
        withCloseButton={false}
        closeOnClickOutside={false}
        closeOnEscape={false}
        title="欢迎使用 ExMind"
        centered
      >
        <Stack>
          <Text size="sm">请首先配置您的 ExMind 根目录，这是存储您的 Workspace 和 Wiki 笔记的物理位置。</Text>
          <TextInput
            label="Root Directory"
            placeholder="例如：C:\Users\Name\Documents\ExMind"
            value={tempDir}
            onChange={(e) => setTempDir(e.currentTarget.value)}
            required
          />
          <Button onClick={handleSetupComplete} fullWidth>完成初始化</Button>
        </Stack>
      </Modal>
    </>
  );
}

export default App;
