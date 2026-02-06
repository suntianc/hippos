# 代码风格规则

> 加载时机：PreToolUse（使用 Write/Edit 工具前）
> 适用范围：所有代码编写和修改操作

---

## 代码风格

<code_style>
遵循以下代码风格规则，确保 AI 友好、可维护的代码。

### 文件长度限制（强制 MUST）

| 文件类型 | 最大行数 | 说明 |
|----------|----------|------|
| 组件文件 (.tsx, .vue, .svelte) | **300 行** | 超出必须立即告警 |
| 服务/工具文件 (.ts, .js) | **500 行** | 超出必须立即告警 |
| 类型定义文件 (.types.ts, .d.ts) | **500 行** | 超出必须立即告警 |
| 测试文件 | 无限制 | 但必须包含单元测试和集成测试 |

**当文件超出限制时，必须立即向用户告警并提出 1-3 个重构方案。**

### 嵌套深度（强制 MUST）

- 代码嵌套深度**不得超过 4 层**
- 如果需要更深的嵌套，必须提取到单独的函数中

### AI 友好代码模式（强制 MUST）

| 模式 | 要求 |
|------|------|
| 显式导入 | **永远不要**使用 `import *`，必须显式列出所有导入 |
| 完整类型 | 所有导出必须有显式的类型定义 |
| 单一职责 | 每个文件/函数应该只做**一件事**，并做好 |
| 清晰命名 | 变量/函数名必须自解释，避免缩写 |
| 意图注释 | 复杂逻辑必须有解释**意图**的注释，而非实现细节 |

### 代码组织（强制 MUST）

使用混合组织模式：

```
src/
├── components/          # 顶层：按功能组织
│   ├── user/           # 内部：按领域组织
│   ├── product/
│   └── order/
├── services/
│   ├── user/
│   └── order/
├── hooks/
└── types/
```

### 命名约定（强制 MUST）

| 元素 | 约定 | 示例 |
|------|------|------|
| 组件 | PascalCase | `UserProfile.tsx` |
| Hooks | camelCase + "use" 前缀 | `useUserData.ts` |
| 服务 | PascalCase + "Service" 后缀 | `UserService.ts` |
| 类型/接口 | PascalCase | `UserConfig`, `IUserProps` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_RETRY_COUNT` |
| 函数 | camelCase，动词开头 | `getUserById`, `calculateTotal` |
| 布尔变量 | camelCase + is/has/can 前缀 | `isLoading`, `hasError`, `canSubmit` |
</code_style>
