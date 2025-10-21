# 正确的 Python 集成方案

## 问题回顾

之前尝试使用 Python embeddable 包失败了，因为：

- ❌ 无法安装需要编译的包（memory service）
- ❌ 缺少构建工具
- ❌ 不是为这个用途设计的

## 正确的解决方案：tauri-plugin-python

我找到了官方支持的 Tauri Python 插件：**`tauri-plugin-python`**

### 特点

✅ **官方支持** - Tauri 生态系统的一部分
✅ **两种模式** - RustPython（纯 Rust）或 PyO3（CPython）
✅ **自动打包** - 处理所有打包细节
✅ **虚拟环境支持** - 可以打包 venv
✅ **简单集成** - 一个命令即可添加

### 工作原理

```
Tauri App
├── Rust Backend
│   └── tauri-plugin-python
│       ├── Python Runtime (embedded)
│       └── Python Functions
└── JavaScript Frontend
    └── tauri-plugin-python-api
        └── Call Python functions
```

## 实现步骤

### 1. 添加插件

```bash
# 自动设置所有必要的配置
npm run tauri add python
```

这会：

- 添加 Rust 依赖
- 添加 JS 依赖
- 创建 `src-tauri/src-python/` 目录
- 配置权限

### 2. 创建 Python 代码

**src-tauri/src-python/main.py**:

```python
# 注册可从 JS 调用的函数
_tauri_plugin_functions = ["start_memory_service", "search_memory", "store_conversation"]

def start_memory_service(kb_path):
    """启动内存服务"""
    import memory_service
    # 实现逻辑
    return {"status": "success", "message": "Memory service started"}

def search_memory(query, limit=5):
    """搜索记忆"""
    # 实现语义搜索
    return {"results": [...]}

def store_conversation(conversation_id, user_msg, ai_msg):
    """存储对话"""
    # 存储到知识库
    return {"stored": True}
```

### 3. 在 Rust 中注册

**src-tauri/src/lib.rs**:

```rust
fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_python::init([
            "start_memory_service",
            "search_memory",
            "store_conversation"
        ]))
        // ... 其他插件
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 4. 从 JavaScript 调用

**src/services/pythonMemory.ts**:

```typescript
import { callFunction } from 'tauri-plugin-python-api';

export async function startMemoryService(kbPath: string) {
  return await callFunction('start_memory_service', [kbPath]);
}

export async function searchMemory(query: string, limit: number = 5) {
  return await callFunction('search_memory', [query, limit]);
}

export async function storeConversation(conversationId: string, userMsg: string, aiMsg: string) {
  return await callFunction('store_conversation', [conversationId, userMsg, aiMsg]);
}
```

### 5. 安装 Python 依赖

```bash
# 创建虚拟环境
python3 -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate

# 安装依赖
pip install memory-service txtai
```

### 6. 配置打包

**src-tauri/tauri.conf.json**:

```json
{
  "bundle": {
    "resources": ["src-python/", "../.venv/include/", "../.venv/lib/"]
  }
}
```

## 两种模式选择

### 模式 1: RustPython（默认）

**优点**:

- ✅ 纯 Rust 实现
- ✅ 无外部依赖
- ✅ 更小的二进制
- ✅ 更容易部署

**缺点**:

- ⚠️ 不支持所有 Python 库
- ⚠️ 性能可能较慢

**配置**:

```toml
# Cargo.toml
tauri-plugin-python = "0.3"
```

### 模式 2: PyO3（CPython）

**优点**:

- ✅ 完整的 Python 兼容性
- ✅ 所有库都能用
- ✅ 更好的性能

**缺点**:

- ⚠️ 需要打包 Python 运行时
- ⚠️ 更大的二进制

**配置**:

```toml
# Cargo.toml
tauri-plugin-python = { version = "0.3", features = ["pyo3"] }
```

## 推荐方案

### 对于 CogniCal

使用 **PyO3 模式** + **虚拟环境打包**：

1. 完整的 Python 兼容性（memory-service 能用）
2. 虚拟环境打包（所有依赖都包含）
3. 零配置部署（用户无需安装 Python）

### 实施计划

```bash
# 1. 添加插件
npm run tauri add python

# 2. 切换到 PyO3
# 编辑 src-tauri/Cargo.toml
tauri-plugin-python = { version = "0.3", features = ["pyo3"] }

# 3. 创建虚拟环境
python3 -m venv .venv
source .venv/bin/activate

# 4. 安装依赖
pip install memory-service txtai

# 5. 配置打包
# 编辑 tauri.conf.json 添加 venv 路径

# 6. 测试
npm run tauri dev

# 7. 构建
npm run tauri build
```

## 优势

相比之前的方案：

| 特性       | Embeddable Python | tauri-plugin-python |
| ---------- | ----------------- | ------------------- |
| 官方支持   | ❌                | ✅                  |
| 安装复杂包 | ❌                | ✅                  |
| 自动打包   | ❌                | ✅                  |
| 文档完善   | ❌                | ✅                  |
| 社区支持   | ❌                | ✅                  |
| 零配置     | ❌                | ✅                  |

## 预期结果

### 开发体验

```bash
npm run tauri add python  # 一次性设置
npm run tauri dev         # 直接运行
```

### 用户体验

```
下载安装包 → 安装 → 运行 → 一切正常工作
```

无需：

- ❌ 安装 Python
- ❌ 安装 pip 包
- ❌ 配置环境变量
- ❌ 任何手动步骤

## 下一步

1. 清理之前的尝试（已完成 ✅）
2. 安装 tauri-plugin-python
3. 重写内存服务集成使用 Python 插件
4. 测试打包
5. 验证零配置部署

---

**这才是正确的方案！** 🎯
