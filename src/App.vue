<script setup lang="ts">
import { ref, computed, onMounted } from "vue";

interface ClipboardItem {
  id: number;
  text: string;
  timestamp: string;
  pinned: boolean;
}

const items = ref<ClipboardItem[]>([]);
const searchQuery = ref("");
const nextId = ref(1);

// 模拟剪贴板数据（后续会接 Rust 后端）
const mockData: ClipboardItem[] = [
  { id: 1, text: "npm install @tauri-apps/api", timestamp: "10:32", pinned: true },
  { id: 2, text: "git commit -m \"feat: add clipboard history\"", timestamp: "10:28", pinned: false },
  { id: 3, text: "https://github.com/zdev0x/clipstash", timestamp: "10:15", pinned: false },
  { id: 4, text: "console.log(\"Hello, Tauri!\")", timestamp: "09:55", pinned: false },
];

onMounted(() => {
  items.value = mockData;
  nextId.value = mockData.length + 1;
});

const filteredItems = computed(() => {
  if (!searchQuery.value) return items.value;
  const q = searchQuery.value.toLowerCase();
  return items.value.filter(item => item.text.toLowerCase().includes(q));
});

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
  // TODO: 后续加个 toast 提示
}

function togglePin(item: ClipboardItem) {
  item.pinned = !item.pinned;
}

function deleteItem(id: number) {
  items.value = items.value.filter(item => item.id !== id);
}

function clearAll() {
  items.value = items.value.filter(item => item.pinned);
}

// TODO: 后续从 Rust 后端读取真实剪贴板
// async function loadClipboardHistory() {
//   const history = await invoke("get_clipboard_history");
//   items.value = history;
// }
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>📋 ClipStash</h1>
      <p class="subtitle">剪贴板管理器 - Tauri 学习项目</p>
    </header>

    <div class="toolbar">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="🔍 搜索剪贴板内容..."
        class="search-input"
      />
      <button class="btn btn-danger" @click="clearAll" title="清除未固定项">
        🗑️ 清除
      </button>
    </div>

    <div class="stats">
      <span>共 {{ items.length }} 条记录</span>
      <span>{{ items.filter(i => i.pinned).length }} 条已固定</span>
    </div>

    <div class="clip-list">
      <div
        v-for="item in filteredItems"
        :key="item.id"
        class="clip-item"
        :class="{ pinned: item.pinned }"
      >
        <div class="clip-content" @click="copyToClipboard(item.text)">
          <div class="clip-text">{{ item.text }}</div>
          <div class="clip-meta">
            <span class="time">🕐 {{ item.timestamp }}</span>
            <span v-if="item.pinned" class="pin-badge">📌 已固定</span>
          </div>
        </div>
        <div class="clip-actions">
          <button class="btn-icon" @click="togglePin(item)" :title="item.pinned ? '取消固定' : '固定'">
            {{ item.pinned ? '📌' : '📍' }}
          </button>
          <button class="btn-icon" @click="deleteItem(item.id)" title="删除">
            ❌
          </button>
        </div>
      </div>

      <div v-if="filteredItems.length === 0" class="empty-state">
        <p>📭 没有找到匹配的剪贴板内容</p>
      </div>
    </div>

    <footer class="footer">
      <p>⚡ Built with Tauri + Vue 3 | 按 Cmd/Ctrl+Shift+V 唤出 (TODO)</p>
    </footer>
  </div>
</template>

<style scoped>
.app {
  max-width: 600px;
  margin: 0 auto;
  padding: 20px;
}

.header {
  text-align: center;
  padding: 20px 0;
}

.header h1 {
  font-size: 1.8rem;
  color: var(--accent);
}

.subtitle {
  color: var(--text-secondary);
  font-size: 0.9rem;
  margin-top: 4px;
}

.toolbar {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
}

.search-input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 0.95rem;
  outline: none;
  transition: border-color 0.2s;
}

.search-input:focus {
  border-color: var(--accent);
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background 0.2s;
}

.btn-danger {
  background: var(--danger);
  color: white;
}

.btn-danger:hover {
  opacity: 0.85;
}

.stats {
  display: flex;
  justify-content: space-between;
  color: var(--text-secondary);
  font-size: 0.85rem;
  margin-bottom: 12px;
  padding: 0 4px;
}

.clip-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.clip-item {
  display: flex;
  align-items: center;
  padding: 12px 14px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 10px;
  transition: all 0.2s;
  cursor: pointer;
}

.clip-item:hover {
  border-color: var(--accent);
  transform: translateY(-1px);
}

.clip-item.pinned {
  border-left: 3px solid var(--warning);
}

.clip-content {
  flex: 1;
  min-width: 0;
}

.clip-text {
  font-family: "SF Mono", "Fira Code", monospace;
  font-size: 0.9rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.clip-meta {
  display: flex;
  gap: 12px;
  margin-top: 4px;
  font-size: 0.8rem;
  color: var(--text-secondary);
}

.pin-badge {
  color: var(--warning);
}

.clip-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}

.clip-item:hover .clip-actions {
  opacity: 1;
}

.btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1rem;
  padding: 4px;
  border-radius: 4px;
}

.btn-icon:hover {
  background: var(--bg-tertiary);
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
  font-size: 1.1rem;
}

.footer {
  text-align: center;
  padding: 20px 0;
  color: var(--text-secondary);
  font-size: 0.8rem;
}
</style>