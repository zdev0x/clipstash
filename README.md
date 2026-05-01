# 📋 ClipStash

> 剪贴板管理器 — Tauri 2 + Vue 3 + Rust 学习项目

一个跨平台桌面应用，记录你复制过的所有内容，再也不怕找不到！

## 🖼️ 效果预览

```
┌─────────────────────────────────┐
│     📋 ClipStash                │
│  剪贴板管理器                    │
├─────────────────────────────────┤
│  🔍 搜索剪贴板内容...    🗑️ 清除 │
├─────────────────────────────────┤
│ 📌 npm install @tauri-apps/api  │
│    🕐 10:32 | 📌 已固定         │
├─────────────────────────────────┤
│ 📍 git commit -m "feat: ..."   │
│    🕐 10:28                     │
├─────────────────────────────────┤
│ 📍 https://github.com/...       │
│    🕐 10:15                     │
└─────────────────────────────────┘
```

## 🛠️ 技术栈

- **Tauri 2** — 跨平台桌面框架（Rust 后端）
- **Vue 3** + TypeScript — 前端 UI
- **Vite** — 开发构建工具
- **Rust** — 系统级操作（剪贴板监听、文件存储等）

## 🚀 快速开始

### 前置要求

1. **Node.js** >= 18
2. **Rust** — 安装 rustup: https://rustup.rs
3. **系统依赖**:
   - macOS: `xcode-select --install`
   - Windows: 安装 Visual Studio Build Tools + WebView2

### 安装 & 运行

```bash
# 克隆仓库
git clone https://github.com/zdev0x/clipstash.git
cd clipstash

# 安装前端依赖
npm install

# 启动开发模式
npm run tauri dev
```

### 打包发布

```bash
npm run tauri build
```

## 📚 学习路线图

这个项目设计为**边做边学**，按以下阶段推进：

### Phase 1: 跑起来 ✅
- [x] 项目脚手架
- [x] Vue 3 基础 UI
- [x] 理解 Tauri 项目结构

### Phase 2: 前后端通信
- [ ] 理解 `#[tauri::command]` 和 `invoke`
- [ ] 从 Rust 后端读取数据
- [ ] 前端调用 Rust 函数
- [ ] 实现数据的增删改查

### Phase 3: 系统集成
- [ ] 监听系统剪贴板变化
- [ ] 全局快捷键 (Cmd/Ctrl+Shift+V)
- [ ] 系统托盘图标

### Phase 4: 数据持久化
- [ ] 本地 SQLite 存储
- [ ] 应用重启后恢复历史
- [ ] 搜索和过滤

### Phase 5: 高级功能
- [ ] 图片剪贴板支持
- [ ] 快捷短语/模板
- [ ] 自动分类

## 🗂️ 项目结构

```
clipstash/
├── src/                    # 前端代码 (Vue)
│   ├── App.vue            # 主界面组件
│   ├── main.ts            # 入口文件
│   └── style.css          # 全局样式
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── lib.rs         # 核心逻辑（Commands）
│   │   └── main.rs        # 入口
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
├── index.html             # HTML 入口
├── package.json           # Node.js 依赖
└── vite.config.ts         # Vite 配置
```

## 🧩 Rust 知识点速查

在 `src-tauri/src/lib.rs` 中你会看到：

| 概念 | 代码 | 一句话解释 |
|------|------|-----------|
| 结构体 | `struct ClipboardEntry {}` | 类似 JS 的 class，定义数据形状 |
| 序列化 | `#[derive(Serialize)]` | 自动把 Rust 数据转成 JSON |
| 命令 | `#[tauri::command]` | 标记这个函数可以被前端调用 |
| 状态管理 | `State<AppState>` | 全局共享状态，类似 React Context |
| Mutex | `Mutex<Vec<...>>` | 保证多线程安全地读写数据 |
| Option | `Option<String>` | 可能有也可能没有的值（类似 null） |

## 📖 推荐学习资源

- [Tauri 官方文档](https://v2.tauri.app/)
- [Rust Book 中文版](https://kaisery.github.io/trpl-zh-cn/)
- [Vue 3 文档](https://cn.vuejs.org/)

## 📄 License

MIT
