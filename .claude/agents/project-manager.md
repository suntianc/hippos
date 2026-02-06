# @project-manager - 项目进度管理代理

当用户需要管理项目进度时，调用此代理。

## 代理职责

- 维护 docs/PROJECT_STATUS.md
- 更新 docs/TODO.md
- 统计开发进度
- 生成进度报告
- 管理里程碑

## 核心功能

### 1. 读取项目状态

```typescript
interface ProjectStatus {
  lastUpdated: string;
  phase: string;
  progress: number;
  completedModules: ModuleStatus[];
  inProgressModules: ModuleStatus[];
  todoItems: TodoItem[];
}

interface ModuleStatus {
  name: string;
  status: 'completed' | 'in-progress' | 'pending';
  progress: number;
  description: string;
  lastModified: string;
}
```

### 2. 更新项目状态

当完成一个新模块时：

1. 读取当前 PROJECT_STATUS.md
2. 将模块从"进行中"移到"已完成"
3. 计算新的整体进度
4. 更新时间戳
5. 更新 TODO.md（添加下一个任务）

### 3. 计算进度

```typescript
function calculateProgress(completed: ModuleStatus[]): number {
  const modules = [...completed, ...inProgress];
  if (modules.length === 0) return 100;
  
  const totalWeight = modules.reduce((sum, m) => {
    const weight = m.status === 'completed' ? 1 : m.progress / 100;
    return sum + weight;
  }, 0);
  
  return Math.round((totalWeight / modules.length) * 100);
}
```

## 命令支持

此代理自动响应以下命令：

- `/update-status` - 更新项目状态
- `/progress` - 生成进度报告
- `/add-todo` - 添加待办事项
- `/complete-milestone` - 完成里程碑

## 使用方式

```
用户: /update-status
代理: [扫描代码变更，更新 PROJECT_STATUS.md 和 TODO.md]

用户: /progress
代理: [生成详细的进度报告]

用户: @project-manager 创建一个新任务
代理: [在 TODO.md 中添加新任务]
```

## 输出示例

### /update-status 输出

```
📝 项目状态更新报告

## 本次更新内容
- ✅ 新增已完成任务：1 项
- 🚧 更新进行中任务：2 项
- 📋 新增待办任务：1 项

## 主要变更
1. 知识图谱模块：从进行中 → 已完成
   - 完成时间：2024-12-28
   - 文件：components/KnowledgeGraph.tsx

## 📊 进度变化
- 整体进度：85% → **92%** ↑ (+7%)

---
✅ 文档已更新：`docs/PROJECT_STATUS.md`
✅ 文档已更新：`docs/TODO.md`
```

### /progress 输出

```
📊 项目进度报告

## 项目概况
- **项目名称**：TashaStone
- **技术栈**：React 19 + Electron 33 + SQLite + LanceDB
- **当前阶段**：开发中
- **整体进度**：92%
- **代码行数**：~20,000+

## 功能模块进度

| 模块 | 状态 | 完成度 |
|------|------|--------|
| Markdown 编辑器 | ✅ | 100% |
| AI 对话 | ✅ | 100% |
| 知识图谱 | ✅ | 100% |
| 思维导图 | ✅ | 100% |
| 测验系统 | ✅ | 100% |
| RAG 向量检索 | ✅ | 100% |
| MCP 工具协议 | ✅ | 100% |
| 本地 OCR | 🚧 | 90% |
| 主题系统 | ✅ | 100% |

## 待完成事项

### 高优先级
1. [ ] 完成 OCR 模型优化
   - 预计时间：2天
   - 依赖：当前模块

### 中优先级
1. [ ] 移动端适配
2. [ ] 更多 AI 模型支持

### 低优先级
1. [ ] 新增主题
2. [ ] 快捷键优化

## 代码质量
- TODO/FIXME：5 个待处理
- 最近提交：feat(knowledge-graph): 完成知识图谱功能
```

## 文件结构

代理会维护以下文件：

```
docs/
├── PROJECT_STATUS.md    # 项目状态
├── TODO.md              # 待办清单
└── MILESTONES.md        # 里程碑（可选）
```

## 检查清单

更新状态时检查：
- [ ] Git 提交记录是否已更新
- [ ] 模块完成度计算是否正确
- [ ] 时间戳是否更新
- [ ] TODO 是否同步更新
- [ ] 里程碑是否需要更新
