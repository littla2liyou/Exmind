import { create } from 'zustand';

interface AppState {
  rootDir: string | null;
  apiKey: string | null;
  setRootDir: (dir: string) => void;
  setApiKey: (key: string) => void;
  
  // Navigation
  activeTab: 'workspace' | 'my-wiki' | 'agent-wiki' | 'settings';
  setActiveTab: (tab: 'workspace' | 'my-wiki' | 'agent-wiki' | 'settings') => void;
  
  // Editor
  currentOpenedFile: string | null;
  setCurrentOpenedFile: (path: string | null) => void;
  selectedText: string;
  setSelectedText: (text: string) => void;
  
  // Wiki
  currentWikiFile: any | null;
  setCurrentWikiFile: (file: any | null) => void;
}

export const useAppStore = create<AppState>((set) => ({
  rootDir: localStorage.getItem('exmind_root_dir') || null,
  apiKey: localStorage.getItem('exmind_api_key') || null,
  
  setRootDir: (dir) => {
    localStorage.setItem('exmind_root_dir', dir);
    set({ rootDir: dir });
  },
  
  setApiKey: (key) => {
    localStorage.setItem('exmind_api_key', key);
    set({ apiKey: key });
  },
  
  activeTab: 'workspace',
  setActiveTab: (tab) => set({ activeTab: tab }),
  
  currentOpenedFile: null,
  setCurrentOpenedFile: (path) => set({ currentOpenedFile: path }),

  selectedText: '',
  setSelectedText: (text) => set({ selectedText: text }),

  currentWikiFile: null,
  setCurrentWikiFile: (file) => set({ currentWikiFile: file }),
}));
