#  CogniCal - 智能任务与时间管理

[English](#english) | [中文](#中文)

---

<a name="中文"></a>
## 中文

> **版本**: v1.0.1014 | **状态**:  正式版

CogniCal 是一款基于 **Tauri + React + Rust** 构建的智能桌面应用，通过 AI 驱动的任务解析、智能规划和数据分析，帮助你更高效地管理时间和任务。

###  核心功能

| 功能 | 描述 |
|------|------|
|  **生产力评分** | 0-100 综合评分，多维度分析（完成率、专注时间、工作负载） |
|  **AI 智能规划** | 多方案推荐、冲突检测、离线降级算法 |
|  **工作量预测** | 7/14/30 天容量预测，风险预警 |
|  **健康提醒** | 专注时长检测、休息提醒、工作节奏分析 |
|  **AI 对话助手** | 自然语言任务管理、工具调用、上下文记忆 |
|  **智能日历** | 月视图、时间块可视化、任务依赖关系图 |

###  技术栈

- **后端**: Rust + Tauri 2 + SQLite
- **前端**: React 19 + TypeScript 5.8 + Tailwind CSS
- **状态管理**: Zustand + React Query
- **AI**: DeepSeek API

###  快速开始

```bash
# 克隆仓库
git clone https://github.com/M-24rjgc/CogniCal.git
cd CogniCal

# 安装依赖
pnpm install

# 开发运行
pnpm tauri dev
```

###  环境要求

- Node.js 18+
- Rust 1.70+
- pnpm

###  配置

在设置中配置 DeepSeek API 密钥即可启用 AI 功能。

---

<a name="english"></a>
## English

> **Version**: v1.0.1014 | **Status**:  Production Ready

CogniCal is an intelligent desktop application built with **Tauri + React + Rust**, featuring AI-powered task parsing, smart planning, and data analytics to help you manage time and tasks more efficiently.

###  Core Features

| Feature | Description |
|---------|-------------|
|  **Productivity Score** | 0-100 composite score with multi-dimensional analysis |
|  **AI Planning** | Multi-option recommendations, conflict detection, offline fallbacks |
|  **Workload Forecast** | 7/14/30-day capacity prediction with risk alerts |
|  **Wellness Reminders** | Focus streak detection, break reminders, rhythm analysis |
|  **AI Chat Assistant** | Natural language task management with tool calling |
|  **Smart Calendar** | Month view, time block visualization, dependency graphs |

###  Tech Stack

- **Backend**: Rust + Tauri 2 + SQLite
- **Frontend**: React 19 + TypeScript 5.8 + Tailwind CSS
- **State Management**: Zustand + React Query
- **AI**: DeepSeek API

###  Quick Start

```bash
# Clone repository
git clone https://github.com/M-24rjgc/CogniCal.git
cd CogniCal

# Install dependencies
pnpm install

# Run in development
pnpm tauri dev
```

###  Requirements

- Node.js 18+
- Rust 1.70+
- pnpm

###  Configuration

Configure your DeepSeek API key in Settings to enable AI features.

---

##  License

MIT License - See [LICENSE](./LICENSE) for details.

##  Links

- **GitHub**: [https://github.com/M-24rjgc/CogniCal](https://github.com/M-24rjgc/CogniCal)
- **Issues**: [Report Bug / Request Feature](https://github.com/M-24rjgc/CogniCal/issues)

---

**Built with  using Tauri + React + Rust**
