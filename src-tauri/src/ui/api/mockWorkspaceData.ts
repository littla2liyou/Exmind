export const MOCK_WORKSPACE_DATA = [
  {
    id: 'src',
    name: 'src',
    isDir: true,
    children: [
      {
        id: 'src/main.rs',
        name: 'main.rs',
        isDir: false,
      },
      {
        id: 'src/lib.rs',
        name: 'lib.rs',
        isDir: false,
      }
    ]
  },
  {
    id: 'Cargo.toml',
    name: 'Cargo.toml',
    isDir: false,
  },
  {
    id: 'README.md',
    name: 'README.md',
    isDir: false,
  }
];

export const MOCK_FILE_CONTENT: Record<string, string> = {
  'src/main.rs': `fn main() {
    println!("Hello, ExMind!");
}`,
  'src/lib.rs': `pub fn add(a: i32, b: i32) -> i32 {
    a + b
}`,
  'Cargo.toml': `[package]
name = "exmind"
version = "0.1.0"
edition = "2021"`,
  'README.md': `# ExMind
Welcome to ExMind Workspace.
This is a markdown file.`
};
