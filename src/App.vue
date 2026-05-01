<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

// ===== 类型定义 =====

interface ClipboardItem {
  id: number;
  content: string;
  timestamp: string;
  pinned: boolean;
  content_type: string;
  image_path?: string | null;
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
      "SELECT COUNT(*) FROM users WHERE active = true;",
      "git push origin main --force-with-lease",
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

/// 从系统剪贴板捕获内容
async function captureFromClipboard() {
  try {
    const entry: ClipboardItem = await invoke("capture_clipboard");
    items.value.unshift(entry);
    // 复制提示
    copiedId.value = entry.id;
    setTimeout(() => (copiedId.value = null), 1500);
  } catch (e) {
    error.value = String(e);
    setTimeout(() => (error.value = null), 3000);
  }
}

/// 通过文件选择器上传图片
async function uploadImage() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"],
      }],
    });

    if (selected) {
      const entry: ClipboardItem = await invoke("save_image_entry", {
        filePath: selected,
      });
      items.value.unshift(entry);
    }
  } catch (e) {
    console.error("Upload failed:", e);
  }
}

/// 生成图片的 file:// URL
function getImageUrl(imagePath: string | null | undefined): string {
  if (!imagePath) return "";
  // Tauri v2 支持通过 convertFileSrc 转换路径
  return `asset://localhost/${imagePath}`;
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
        <span class="version">v0.5</span>
      </div>
      <div class="shortcut-hint">
        ⌨️ <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>V</kbd> · 支持文本 + 图片
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
      <button class="btn btn-accent" @click="captureFromClipboard" title="从剪贴板捕获">
        📋
      </button>
      <button class="btn btn-secondary" @click="uploadImage" title="上传图片">
        🖼️
      </button>
      <button class="btn btn-secondary" @click="addTestEntry" title="添加测试文本">
        ➕
      </button>
      <button class="btn btn-danger" @click="clearAll" title="清除未固定项">
        🗑️
      </button>
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
    <div v-else-if="error" class="error-toast">
      ⚠️ {{ error }}
    </div>

    <!-- 剪贴板列表 -->
    <div v-if="!loading" class="clip-list">
      <TransitionGroup name="list">
        <div
          v-for="item in filteredItems"
          :key="item.id"
          class="clip-item"
          :class="{
            pinned: item.pinned,
            copied: copiedId === item.id,
            'is-image': item.content_type === 'image'
          }"
          @click="copyToClipboard(item)"
        >
          <!-- 图片预览 -->
          <div v-if="item.content_type === 'image' && item.image_path" class="clip-image-wrap">
            <img
              :src="getImageUrl(item.image_path)"
              class="clip-image"
              alt="clipboard image"
              @error="(e) => (e.target as HTMLImageElement).style.display='none'"
            />
          </div>

          <!-- 内容图标 -->
          <div v-else class="clip-type">
            {{ item.content_type === 'image' ? '🖼️' : '📝' }}
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
            <button class="btn-icon" @click="deleteItem(item)" title="删除">
              ❌
            </button>
          </div>
        </div>
      </TransitionGroup>

      <div v-if="filteredItems.length === 0 && !loading" class="empty-state">
        <p>📭 {{ searchQuery ? '没有匹配的内容' : '剪贴板是空的' }}</p>
        <p class="empty-sub" v-if="!searchQuery">
          点 📋 捕获剪贴板 · 点 🖼️ 上传图片 · 点 ➕ 添加测试
        </p>
      </div>
    </div>

    <footer class="footer">
      <span>Tauri + Vue 3 + Rust + SQLite · Phase 5 图片支持 🖼️</span>
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

.header { text-align: center; padding: 8px 0 12px; }

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
  gap: 6px;
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

.btn-accent { background: var(--accent); color: white; }
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
}

.db-badge {
  margin-left: auto;
  background: var(--bg-tertiary);
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 0.7rem;
}

.loading {
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

.error-toast {
  background: rgba(247, 118, 142, 0.15);
  border: 1px solid var(--danger);
  color: var(--danger);
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 0.85rem;
  margin-bottom: 8px;
  text-align: center;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn { from { opacity: 0; transform: translateY(-8px); } }

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

.clip-image-wrap {
  width: 48px;
  height: 48px;
  border-radius: 6px;
  overflow: hidden;
  flex-shrink: 0;
  background: var(--bg-tertiary);
}

.clip-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
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
