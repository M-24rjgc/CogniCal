# Git 使用指南

## 📋 Git 配置说明

本项目已完成 Git 初始化，以下是重要的配置说明：

### ✅ 已纳入版本控制的内容

#### 源代码
- 所有 TypeScript/JavaScript 源文件 (`src/**`)
- Rust 后端代码 (`src-tauri/src/**`)
- 测试文件 (`src/__tests__/**`, `e2e/**`, `src-tauri/tests/**`)

#### 配置文件
- 项目配置: `package.json`, `tsconfig.json`, `vite.config.ts`, 等
- 工具配置: `.eslintrc.*`, `.prettierrc`, `.lintstagedrc.json`
- Tauri 配置: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
- Git 配置: `.gitignore`, `.gitattributes`

#### 文档
- 项目文档: `README.md`, `CHANGELOG.md`, `CogniCal.md`
- 开发文档: 所有 `.md` 文件（包括问题跟踪和实现摘要）
- 规范文档: `.spec-workflow/` 目录（工作流模板和归档）

#### 资源文件
- 图标和静态资源: `public/**`, `src-tauri/icons/**`
- 数据库架构: `src-tauri/src/db/schema.sql`

#### 开发工具
- Husky Git hooks: `.husky/**`
- VS Code 推荐配置: `.vscode/**`

### ❌ 已忽略的内容

#### 依赖和构建产物
- `node_modules/` - Node.js 依赖包
- `dist/`, `dist-ssr/` - 前端构建输出
- `src-tauri/target/` - Rust 编译输出
- `.pnpm-store/` - pnpm 缓存

#### 临时和生成文件
- `test-results/`, `playwright-report/` - 测试结果
- `*.log` - 日志文件
- `.venv/` - Python 虚拟环境

#### IDE 和系统文件
- `.DS_Store` - macOS 系统文件
- `Thumbs.db`, `Desktop.ini` - Windows 系统文件
- `.kilocode/` - IDE 配置

#### 敏感信息
- `.env*` - 环境变量文件（可能包含 API 密钥）

## 🚀 常用 Git 命令

### 查看状态
```powershell
git status              # 查看当前状态
git status --short      # 简短格式
git log --oneline -10   # 查看最近 10 条提交记录
```

### 提交更改
```powershell
# 添加文件到暂存区
git add .                           # 添加所有更改
git add src/components/NewFile.tsx  # 添加特定文件

# 提交
git commit -m "feat: 添加新功能"
git commit -m "fix: 修复某个bug"
git commit -m "docs: 更新文档"
```

### 分支管理
```powershell
# 创建和切换分支
git branch feature/new-feature      # 创建新分支
git checkout feature/new-feature    # 切换到分支
git checkout -b feature/new-feature # 创建并切换

# 合并分支
git checkout master
git merge feature/new-feature

# 删除分支
git branch -d feature/new-feature   # 删除已合并的分支
```

### 远程仓库
```powershell
# 添加远程仓库
git remote add origin <远程仓库URL>

# 推送到远程
git push -u origin master           # 首次推送
git push                            # 后续推送

# 拉取更新
git pull origin master
```

## 📝 提交信息规范

建议使用以下前缀来组织提交信息：

- `feat:` - 新功能
- `fix:` - Bug 修复
- `docs:` - 文档更改
- `style:` - 代码格式（不影响代码运行）
- `refactor:` - 重构（既不是新功能也不是修复）
- `perf:` - 性能优化
- `test:` - 添加或修改测试
- `chore:` - 构建过程或辅助工具的变动
- `ci:` - CI/CD 配置更改

示例：
```
feat: 添加任务智能解析面板
fix: 修复日期选择器的边界情况
docs: 更新 API 使用说明
refactor: 优化数据库查询性能
```

## 🔄 工作流建议

### 开发新功能
```powershell
# 1. 创建功能分支
git checkout -b feature/ai-enhancement

# 2. 开发并提交
git add .
git commit -m "feat: 实现 AI 增强功能"

# 3. 合并回主分支
git checkout master
git merge feature/ai-enhancement

# 4. 删除功能分支
git branch -d feature/ai-enhancement
```

### 修复 Bug
```powershell
# 1. 创建修复分支
git checkout -b fix/task-parsing-error

# 2. 修复并提交
git add .
git commit -m "fix: 修复任务解析中的空值错误"

# 3. 合并回主分支
git checkout master
git merge fix/task-parsing-error

# 4. 删除修复分支
git branch -d fix/task-parsing-error
```

## 🛡️ .gitattributes 说明

项目已配置 `.gitattributes` 来确保跨平台一致性：
- 所有文本文件使用 LF 行尾（Unix 风格）
- 二进制文件（图片、字体等）正确标记
- Git 会自动处理 Windows (CRLF) 和 Unix (LF) 之间的转换

## 💡 最佳实践

1. **频繁提交** - 小步提交，每次只做一件事
2. **清晰的提交信息** - 让其他人（包括未来的你）能理解
3. **使用分支** - 为新功能或修复创建单独的分支
4. **定期推送** - 将本地更改推送到远程仓库备份
5. **代码审查** - 合并前检查更改内容
6. **避免提交敏感信息** - API 密钥、密码等应使用环境变量

## 🔍 检查忽略规则

如果不确定某个文件是否会被 Git 跟踪：

```powershell
git check-ignore -v <文件路径>
```

## 📚 其他资源

- [Git 官方文档](https://git-scm.com/doc)
- [GitHub 指南](https://guides.github.com/)
- [Pro Git 书籍（中文版）](https://git-scm.com/book/zh/v2)
