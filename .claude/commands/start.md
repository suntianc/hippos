# /start - 快速了解项目

作为项目引导助手，请帮我快速了解这个项目的当前状态。

## 你需要做的：

1. 识别项目类型和技术栈
2. 读取 docs/PROJECT_STATUS.md 获取当前状态
3. 扫描主要模块目录（components/、services/、electron/）
4. 统计 TODO/FIXME 数量
5. 展示已安装的 Claude Code 插件能力

## Git 可用时（加分项）：
- 查看最近 3 条 Git 提交
- 检查代码状态（是否有未提交修改）

## Git 不可用时（降级方案）：
- 跳过"最近动态"部分
- 基于 PROJECT_STATUS.md 提供状态概述

## 输出格式：

# 👋 欢迎回到项目

## 项目信息
- **项目名称**：TashaStone
- **技术栈**：React 19 + Electron 33 + SQLite + LanceDB + TypeScript
- **当前版本**：1.7.0

## 🔌 已安装插件

| 插件 | 能力 |
|------|------|
| **context7** | 官方文档查询（1000+ 库） |
| **gopls-lsp** | Go 语言语义分析 |
| **rust-analyzer-lsp** | Rust 语言语义分析 |
| **code-review** | 自动化代码审查 |
| **feature-dev** | 结构化功能开发流程 |

## 🛠️ 内置技能

| 技能 | 描述 |
|------|------|
| **spec-interview** | 深度访谈，完善技术规格 |
| **ai-integration** | AI 服务集成 |
| **bug-debug** | Bug 调试和问题排查 |
| **electron-main** | Electron 主进程开发 |
| **mcp-tools** | MCP 工具协议开发 |
| **platform-build** | 平台构建和打包 |
| **rag-vectordb** | RAG 向量数据库 |
| **react-frontend** | React 前端开发 |

## 📊 当前状态
- 代码状态：正常 / 有未提交修改
- **待办任务**：X 个（来自 docs/TODO.md）

## 🎯 可用命令

| 命令 | 描述 |
|------|------|
| `/start` | 快速了解项目 |
| `/progress` | 查看详细项目进度 |
| `/next` | 获取下一步开发建议 |
| `/update-status` | 更新项目状态 |
| `/feature-start` | 开始新功能开发 |

## 💡 技能使用示例

```
# 查询文档
How do I use React useEffect? use context7

# 代码审查
@code-reviewer 审查最近的代码变更

# 功能开发
/feature-start 实现文件搜索功能

# 完善规格
帮我完善这个 plan
@spec-interview 细化这个技术规格
```

## 🎯 你可以：
1. 输入 `/progress` - 查看详细的项目进度
2. 输入 `/next` - 获取下一步开发建议
3. 输入 `/feature-start` - 开始新功能开发
4. 输入 `@spec-interview` - 完善技术规格
5. 直接告诉我具体任务
