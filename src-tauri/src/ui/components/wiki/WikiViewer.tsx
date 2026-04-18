import React, { useEffect, useState } from 'react';
import { Box, Paper, Badge, Group, Title, Button } from '@mantine/core';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import yaml from 'js-yaml';
import { IconCopy } from '@tabler/icons-react';

interface WikiViewerProps {
  content: string;
  isAgentWiki?: boolean;
  onSyncToMyWiki?: () => void;
}

export const WikiViewer: React.FC<WikiViewerProps> = ({ content, isAgentWiki, onSyncToMyWiki }) => {
  const [markdownContent, setMarkdownContent] = useState('');
  const [metadata, setMetadata] = useState<any>({});

  useEffect(() => {
    try {
      // Parse frontmatter
      const match = content.match(/^---\s*\n([\s\S]*?)\n---\s*\n/);
      if (match) {
        const yamlStr = match[1];
        const data = yaml.load(yamlStr) as any;
        const textContent = content.slice(match[0].length);
        setMetadata(data || {});
        setMarkdownContent(textContent);
      } else {
        setMetadata({});
        setMarkdownContent(content);
      }
    } catch (e) {
      console.error('Failed to parse frontmatter', e);
      setMarkdownContent(content);
      setMetadata({});
    }
  }, [content]);

  return (
    <Box p="md" style={{ height: '100%', overflowY: 'auto' }}>
      <Paper p="xl" shadow="sm" radius="md" withBorder>
        {/* Header Section */}
        <Group justify="space-between" mb="lg" pb="sm" style={{ borderBottom: '1px solid var(--mantine-color-default-border)' }}>
          <Box>
            <Title order={2}>{metadata.title || 'Untitled Document'}</Title>
            <Group mt="xs" gap="xs">
              {isAgentWiki ? (
                <Badge color="grape" variant="light">Agent AI</Badge>
              ) : (
                <Badge color="blue" variant="light">User</Badge>
              )}
              {metadata.date && <Badge color="gray" variant="outline">{metadata.date}</Badge>}
              {metadata.tags && Array.isArray(metadata.tags) && metadata.tags.map((tag: string) => (
                <Badge key={tag} color="teal" variant="dot">{tag}</Badge>
              ))}
            </Group>
          </Box>
          
          {isAgentWiki && (
            <Button 
              leftSection={<IconCopy size={16} />} 
              variant="light" 
              color="blue"
              onClick={onSyncToMyWiki}
            >
              同步到 My Wiki
            </Button>
          )}
        </Group>

        {/* Content Section */}
        <Box className="markdown-body">
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
            {markdownContent}
          </ReactMarkdown>
        </Box>
      </Paper>
    </Box>
  );
};
