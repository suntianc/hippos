# /bootstrap - 项目初始化与需求对齐

作为项目初始化助手，完成项目设置、需求对齐和架构设计。

## 使用方式

```
/bootstrap [项目描述]
```

示例：
```
/bootstrap 开发一个AI驱动的Markdown编辑器
/bootstrap 创建电商后台管理系统
/bootstrap 构建个人博客网站
```

## 执行流程

### 阶段 1: 创建规划文件 (planning-with-files)

首先创建任务规划文件，采用 Manus 风格的文件规划模式：

```bash
# 检查 planning/ 目录是否存在
# 如不存在，创建以下文件：
planning/
├── task_plan.md      # 任务计划和阶段跟踪
├── findings.md       # 发现和研究记录
└── progress.md       # 进度日志
```

**重要**: 按照 planning-with-files 技能的模板创建这些文件。

### 阶段 2: 需求对齐 (spec-interview)

激活 `spec-interview` 技能进行深度需求访谈：

1. **扫描项目**
   - 检测现有文件和配置
   - 识别项目类型和技术栈
   - 分析项目结构

2. **深度访谈**
   - 技术选型的理由和替代方案
   - 用户交互流程
   - 架构决策和模块划分
   - 风险考量和权衡取舍

3. **记录结果**
   - 将访谈结果记录到 `docs/PROJECT.md`
   - 更新 `planning/findings.md`

### 阶段 3: 架构设计

根据需求访谈结果，生成架构设计文档：

1. **创建 `docs/ARCHITECTURE.md`**
   - 系统架构图
   - 模块划分和职责
   - 数据流向
   - 技术选型理由

2. **创建 `docs/TODO.md`**
   - 初始任务列表
   - 优先级排序
   - 里程碑规划

3. **更新 `docs/PROJECT_STATUS.md`**
   - 项目当前状态
   - 下一步计划

### 阶段 4: 配置验证

检查必需文件和配置：

```
必需文件检查：
- [ ] CLAUDE.md          # 全局 AI 指令
- [ ] AGENTS.md          # 项目规则
- [ ] docs/PROJECT.md    # 项目概览
- [ ] docs/ARCHITECTURE.md # 架构设计
- [ ] planning/task_plan.md # 任务计划
```

### 阶段 5: 完成初始化

1. 更新 `planning/task_plan.md` 标记阶段完成
2. 生成初始化报告

## 输出格式

```markdown
# ✅ 项目初始化完成

## 📁 创建的文件

### 规划文件
- [x] planning/task_plan.md
- [x] planning/findings.md
- [x] planning/progress.md

### 文档文件
- [x] docs/PROJECT.md
- [x] docs/ARCHITECTURE.md
- [x] docs/TODO.md
- [x] docs/PROJECT_STATUS.md

## 📊 项目信息

| 属性 | 值 |
|------|-----|
| **项目名称** | [识别的项目名] |
| **技术栈** | [识别的技术栈] |
| **项目类型** | [Web/桌面/全栈/...] |
| **初始任务数** | X 个 |

## 🏗️ 架构概览

[简要架构描述]

## 🎯 下一步操作

1. 运行 `/start` 查看项目状态概览
2. 运行 `/next` 获取下一步开发建议
3. 开始开发第一个功能

## 💡 可用命令

| 命令 | 描述 |
|------|------|
| `/start` | 快速了解项目 |
| `/progress` | 查看详细进度 |
| `/next` | 获取下一步建议 |
| `/feature-start` | 开始新功能开发 |
| `/update-status` | 更新项目状态 |
| `/commit` | 完成任务并提交 |
```

## 注意事项

1. **必须调用 spec-interview**: 需求访谈是核心，不能跳过
2. **planning-with-files 模式**: 所有重要信息写入文件，不依赖上下文
3. **架构设计优先**: 先设计后编码
4. **文档驱动**: 所有决策记录在文档中

## 与其他技能协同

| 技能 | 阶段 | 用途 |
|------|------|------|
| **spec-interview** | 阶段 2 | 深度需求访谈 |
| **planning-with-files** | 阶段 1 | 创建规划文件 |
| **react-frontend** | 后续 | 前端开发指导 |
| **electron-main** | 后续 | 后端开发指导 |
