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
const copiedId = ref<number | null>(null);

const mockData: ClipboardItem[] = [
  { id: 1, text: "npm install @tauri-apps/api", timestamp: "10:32", pinned: true },
  { id: 2, text: 'git commit -m "feat: add clipboard history"', timestamp: "10:28", pinned: false },
  { id: 3, text: "https://github.com/zdev0x/clipstash", timestamp: "10:15", pinned: false },
  { id: 4, text: 'console.log("Hello, Tauri!")', timestamp: "09:55", pinned: false },
  { id: 5, text: "SELECT * FROM users WHERE active = true;", timestamp: "09:30", pinned: false },
  { id: 6, text: "border-radius: 12px; background: #1a1b26;", timestamp: "09:15", pinned: false },
];

onMounted(() => {
  items.value = mockData;
});

const filteredItems = computed(() => {
  if (!searchQuery.value) return items.value;
  const q = searchQuery.value.toLowerCase();
  return items.value.filter((item) => item.text.toLowerCase().includes(q));
});

const pinnedCount = computed(() => items.value.filter((i) => i.pinned).length);

function copyToClipboard(item: ClipboardItem) {
  navigator.clipboard.writeText(item.text);
  copiedId.value = item.id;
  setTimeout(() => (copiedId.value = null), 1500);
}

function togglePin(item: ClipboardItem) {
  item.pinned = !item.pinned;
}

function deleteItem(id: number) {
  items.value = items.value.filter((item) => item.id !== id);
}

function clearAll() {
  items.value = items.value.filter((item) => item.pinned);
}

// TODO: 后续接 Rust invoke
// async function loadHistory() {
//   items.value = await invoke("get_clipboard_history");
// }
</script>

<template>
  <div class="app">
    <header class="header">
      <div class="header-top">
        <h1>📋 ClipStash</h1>
        <span class="version">v0.2</span>
      </div>
      <div class="shortcut-hint">
        ⌨️ <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>V</kbd> 唤出/隐藏
      </div>
    </header>

    <div class="toolbar">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="🔍 搜索..."
        class="search-input"
        autofocus
      />
      <button class="btn btn-secondary" @click="clearAll" title="清除未固定项">🗑️</button>
    </div>

    <div class="stats">
      <span>{{ items.length }} 条</span>
      <span v-if="pinnedCount > 0">{{ pinnedCount }} 条固定</span>
    </div>

    <div class="clip-list">
      <TransitionGroup name="list">
        <div
          v-for="item in filteredItems"
          :key="item.id"
          class="clip-item"
          :class="{ pinned: item.pinned, copied: copiedId === item.id }"
          @click="copyToClipboard(item)"
        >
          <div class="clip-content">
            <div class="clip-text">{{ item.text }}</div>
            <div class="clip-meta">
              <span class="time">🕐 {{ item.timestamp }}</span>
              <span v-if="item.pinned" class="pin-badge">📌</span>
              <span v-if="copiedId === item.id" class="copied-badge">✅ 已复制</span>
            </div>
          </div>
          <div class="clip-actions" @click.stop>
            <button class="btn-icon" @click="togglePin(item)" :title="item.pinned ? '取消固定' : '固定'">
              {{ item.pinned ? '📌' : '📍' }}
            </button>
            <button class="btn-icon" @click="deleteItem(item.id)" title="删除">❌</button>
          </div>
        </div>
      </TransitionGroup>

      <div v-if="filteredItems.length === 0" class="empty-state">
        <p>📭 没有匹配的内容</p>
        <p class="empty-sub">复制点东西试试？</p>
      </div>
    </div>

    <footer class="footer">
      <span>Tauri + Vue 3 学习项目</span>
    </footer>
  </div>
</template>

<style scoped>
.app {
  max-width: 650px;
  margin: 0 auto;
  padding: 16px 20px;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.header {
  text-align: center;
  padding: 8px 0 12px;
}

.header-top {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.header h1 {
  font-size: 1.5rem;
  color: var(--accent);
}

.version {
  font-size: 0.75rem;
  background: var(--bg-tertiary);
  padding: 2px 8px;
  border-radius: 10px;
  color: var(--text-secondary);
}

.shortcut-hint {
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin-top: 4px;
}

kbd {
  background: var(--bg-tertiary);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-family: monospace;
  border: 1px solid var(--border);
}

.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.search-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 0.9rem;
  outline: none;
  transition: border-color 0.2s;
}

.search-input:focus {
  border-color: var(--accent);
}

.btn {
  padding: 6px 12px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.85rem;
  transition: opacity 0.2s;
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.btn:hover {
  opacity: 0.8;
}

.stats {
  display: flex;
  gap: 12px;
  color: var(--text-secondary);
  font-size: 0.8rem;
  margin-bottom: 8px;
  padding: 0 2px;
}

.clip-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.clip-item {
  display: flex;
  align-items: center;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  transition: all 0.2s;
  cursor: pointer;
}

.clip-item:hover {
  border-color: var(--accent);
}

.clip-item.pinned {
  border-left: 3px solid var(--warning);
}

.clip-item.copied {
  border-color: var(--success);
  background: rgba(158, 206, 106, 0.1);
}

.clip-content {
  flex: 1;
  min-width: 0;
}

.clip-text {
  font-family: "SF Mono", "Fira Code", monospace;
  font-size: 0.85rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.clip-meta {
  display: flex;
  gap: 8px;
  margin-top: 3px;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.pin-badge {
  color: var(--warning);
}

.copied-badge {
  color: var(--success);
  font-weight: 500;
}

.clip-actions {
  display: flex;
  gap: 2px;
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
  font-size: 0.85rem;
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
}

.empty-sub {
  font-size: 0.85rem;
  margin-top: 4px;
}

.footer {
  text-align: center;
  padding: 10px 0 0;
  color: var(--text-secondary);
  font-size: 0.75rem;
  border-top: 1px solid var(--border);
  margin-top: 8px;
}

/* 列表动画 */
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from {
  opacity: 0;
  transform: translateX(-20px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>
