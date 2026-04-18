import React, { useState, useRef, useEffect } from 'react';
import { Box, TextInput, ActionIcon, Text, ScrollArea, Avatar, Group, Paper } from '@mantine/core';
import { IconSend, IconRobot, IconUser } from '@tabler/icons-react';
import { useAppStore } from '../../store';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

export const ChatPanel: React.FC = () => {
  const { messages, addMessage, appendAssistantMessage, selectedText } = useAppStore();
  const [input, setInput] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    if (scrollRef.current) {
      scrollRef.current.scrollTo({ top: scrollRef.current.scrollHeight, behavior: 'smooth' });
    }
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSend = () => {
    if (!input.trim()) return;

    const userMsg = input;
    setInput('');
    addMessage({ id: Date.now().toString(), role: 'user', content: userMsg });
    setIsTyping(true);

    // Mock API Stream
    let mockResponse = "这是一个 Mock 的流式回答。";
    if (selectedText) {
      mockResponse += `\n\n您刚刚选中了这段代码：\n\`\`\`\n${selectedText}\n\`\`\`\n我可以为您解释或重构它。`;
    }

    let i = 0;
    const timer = setInterval(() => {
      if (i < mockResponse.length) {
        appendAssistantMessage(mockResponse[i]);
        i++;
      } else {
        clearInterval(timer);
        setIsTyping(false);
      }
    }, 50);
  };

  return (
    <Box h="100%" display="flex" style={{ flexDirection: 'column' }}>
      <Box p="xs" style={{ borderBottom: '1px solid var(--mantine-color-default-border)' }}>
        <Text fw={600} size="sm" c="dimmed">AI 助手</Text>
      </Box>

      <ScrollArea viewportRef={scrollRef} style={{ flex: 1 }} p="md">
        {messages.length === 0 ? (
          <Text c="dimmed" size="sm" ta="center" mt="xl">
            有什么我可以帮您的？您可以选中代码然后在这里向我提问。
          </Text>
        ) : (
          messages.map((msg) => (
            <Group key={msg.id} align="flex-start" wrap="nowrap" mb="lg">
              <Avatar color={msg.role === 'assistant' ? 'grape' : 'blue'} radius="xl">
                {msg.role === 'assistant' ? <IconRobot size="1.2rem" /> : <IconUser size="1.2rem" />}
              </Avatar>
              <Paper p="sm" radius="md" bg={msg.role === 'user' ? 'var(--mantine-color-blue-light)' : 'transparent'} w="100%">
                <Box className="markdown-body" style={{ fontSize: '14px' }}>
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>
                    {msg.content}
                  </ReactMarkdown>
                </Box>
              </Paper>
            </Group>
          ))
        )}
        {isTyping && (
          <Text size="xs" c="dimmed" mt="xs" ml="xl">AI 正在思考...</Text>
        )}
      </ScrollArea>

      <Box p="md" style={{ borderTop: '1px solid var(--mantine-color-default-border)' }}>
        <TextInput
          placeholder="输入您的问题..."
          value={input}
          onChange={(e) => setInput(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              handleSend();
            }
          }}
          rightSection={
            <ActionIcon color="blue" onClick={handleSend} disabled={!input.trim() || isTyping}>
              <IconSend size="1rem" />
            </ActionIcon>
          }
        />
      </Box>
    </Box>
  );
};
