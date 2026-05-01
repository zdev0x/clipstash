<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

// ===== 类型定义 =====

interface ClipboardItem {
  id: number;
  content: string;
  timestamp: string;
  pinned: boolean;
  content_type: string;
}

// ===== 状态 =====

const items = ref<ClipboardItem[]>([]);
const searchQuery = ref("");
const copiedId = ref<number | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

// ===== 计算属性 =====

const filteredItems = computed(() => {
  if (!searchQuery.value) return items.value;
  const q = searchQuery.value.toLowerCase();
  return items.value.filter(
    (item) =>
      item.content.toLowerCase().includes(q) ||
      item.content_type.toLowerCase().includes(q)
  );
});

const pinnedCount = computed(() => items.value.filter((i) => i.pinned).length);
const totalCount = computed(() => items.value.length);

// ===== Tauri Commands =====

async function loadHistory() {
  try {
    loading.value = true;
    error.value = null;
    items.value = await invoke("get_clipboard_history");
  } catch (e) {
    error.value = String(e);
    console.error("Failed to load history:", e);
  } finally {
    loading.value = false;
  }
}

async function copyToClipboard(item: ClipboardItem) {
  try {
    await navigator.clipboard.writeText(item.content);
    copiedId.value = item.id;
    setTimeout(() => (copiedId.value = null), 1500);
  } catch (e) {
    console.error("Copy failed:", e);
  }
}

async function togglePin(item: ClipboardItem) {
  try {
    const newPinned: boolean = await invoke("toggle_pin", { id: item.id });
    item.pinned = newPinned;
  } catch (e) {
    console.error("Toggle pin failed:", e);
  }
}

async function deleteItem(item: ClipboardItem) {
  try {
    await invoke("delete_entry", { id: item.id });
    items.value = items.value.filter((i) => i.id !== item.id);
  } catch (e) {
    console.error("Delete failed:", e);
  }
}

async function clearAll() {
  try {
    await invoke("clear_unpinned");
    items.value = items.value.filter((item) => item.pinned);
  } catch (e) {
    console.error("Clear failed:", e);
  }
}

async function addTestEntry() {
  try {
    const texts = [
      "console.log('Hello from Tauri!');",
      "docker-compose up -d",
      "ssh user@server.example.com",
      "curl -X POST http://localhost:3000/api/data",
      "git push origin main --force-with-lease",
      "SELECT COUNT(*) FROM users WHERE active = true;",
    ];
    const text = texts[Math.floor(Math.random() * texts.length)];
    const entry: ClipboardItem = await invoke("add_clipboard_entry", {
      content: text,
      contentType: "text",
    });
    items.value.unshift(entry);
  } catch (e) {
    console.error("Add entry failed:", e);
  }
}

// ===== 生命周期 =====

onMounted(() => {
  loadHistory();
});
</script>

<template>
  <div class="app">
    <header class="header">
      <div class="header-top">
        <h1>📋 ClipStash</h1>
        <span class="version">v0.4</span>
      </div>
      <div class="shortcut-hint">
        ⌨️ <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>V</kbd> · 数据已持久化到 SQLite
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
      <button class="btn btn-secondary" @click="addTestEntry" title="添加测试数据">➕</button>
      <button class="btn btn-danger" @click="clearAll" title="清除未固定项">🗑️</button>
    </div>

    <div class="stats">
      <span>{{ totalCount }} 条</span>
      <span v-if="pinnedCount > 0">· {{ pinnedCount }} 固定</span>
      <span class="db-badge">💾 SQLite</span>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading">
      <div class="spinner"></div>
      <p>加载中...</p>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="error-state">
      <p>❌ 加载失败</p>
      <p class="error-msg">{{ error }}</p>
      <button class="btn btn-secondary" @click="loadHistory">重试</button>
    </div>

    <!-- 剪贴板列表 -->
    <div v-else class="clip-list">
      <TransitionGroup name="list">
        <div
          v-for="item in filteredItems"
          :key="item.id"
          class="clip-item"
          :class="{ pinned: item.pinned, copied: copiedId === item.id }"
          @click="copyToClipboard(item)"
        >
          <div class="clip-type">
            {{ item.content_type === 'text' ? '📝' : item.content_type === 'image' ? '🖼️' : '📎' }}
          </div>
          <div class="clip-content">
            <div class="clip-text">{{ item.content }}</div>
            <div class="clip-meta">
              <span class="time">🕐 {{ item.timestamp }}</span>
              <span v-if="item.pinned" class="pin-badge">📌</span>
              <span v-if="copiedId === item.id" class="copied-badge">✅ 已复制</span>
            </div>
          </div>
          <div class="clip-actions" @click.stop>
            <button
              class="btn-icon"
              @click="togglePin(item)"
              :title="item.pinned ? '取消固定' : '固定'"
            >
              {{ item.pinned ? '📌' : '📍' }}
            </button>
            <button class="btn-icon" @click="deleteItem(item)" title="删除">❌</button>
          </div>
        </div>
      </TransitionGroup>

      <div v-if="filteredItems.length === 0 && !loading" class="empty-state">
        <p>📭 {{ searchQuery ? '没有匹配的内容' : '剪贴板是空的' }}</p>
        <p class="empty-sub" v-if="!searchQuery">复制点东西试试？或点击 ➕ 添加测试数据</p>
      </div>
    </div>

    <footer class="footer">
      <span>Tauri + Vue 3 + Rust + SQLite 🚀</span>
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

.header h1 { font-size: 1.5rem; color: var(--accent); }

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

.search-input:focus { border-color: var(--accent); }

.btn {
  padding: 6px 10px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.85rem;
  transition: opacity 0.2s;
}

.btn-secondary { background: var(--bg-tertiary); color: var(--text-primary); }
.btn-danger { background: var(--danger); color: white; }
.btn:hover { opacity: 0.8; }

.stats {
  display: flex;
  gap: 8px;
  align-items: center;
  color: var(--text-secondary);
  font-size: 0.8rem;
  margin-bottom: 8px;
  padding: 0 2px;
}

.db-badge {
  margin-left: auto;
  background: var(--bg-tertiary);
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 0.7rem;
}

.loading, .error-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--text-secondary);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

.error-msg {
  font-size: 0.8rem;
  color: var(--danger);
  max-width: 400px;
  word-break: break-all;
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
  gap: 10px;
}

.clip-item:hover { border-color: var(--accent); }
.clip-item.pinned { border-left: 3px solid var(--warning); }
.clip-item.copied {
  border-color: var(--success);
  background: rgba(158, 206, 106, 0.1);
}

.clip-type {
  font-size: 1.2rem;
  flex-shrink: 0;
  width: 28px;
  text-align: center;
}

.clip-content { flex: 1; min-width: 0; }

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

.pin-badge { color: var(--warning); }
.copied-badge { color: var(--success); font-weight: 500; }

.clip-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.2s;
}

.clip-item:hover .clip-actions { opacity: 1; }

.btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 0.85rem;
  padding: 4px;
  border-radius: 4px;
}

.btn-icon:hover { background: var(--bg-tertiary); }

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
}

.empty-sub { font-size: 0.85rem; margin-top: 4px; }

.footer {
  text-align: center;
  padding: 10px 0 0;
  color: var(--text-secondary);
  font-size: 0.75rem;
  border-top: 1px solid var(--border);
  margin-top: 8px;
}

.list-enter-active, .list-leave-active { transition: all 0.3s ease; }
.list-enter-from { opacity: 0; transform: translateX(-20px); }
.list-leave-to { opacity: 0; transform: translateX(20px); }
</style>
