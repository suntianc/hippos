# 性能规则

> 加载时机：PreToolUse（使用 Write/Edit 工具前）
> 适用范围：所有代码编写和优化操作

---

## 性能意识

<performance_awareness>
编写代码时**必须**考虑性能影响。性能不是可选项。

### 数据库操作（强制 MUST）

| 规则 | 说明 |
|------|------|
| 禁止 N+1 查询 | **永远不要**创建 N+1 查询模式 - 始终批量处理数据库操作 |
| 分页处理 | 对大结果集使用分页（将查询限制在合理大小） |
| 添加索引 | 创建数据库查询时添加适当的索引 |
| 使用事务 | 对必须一起成功或失败的相关操作使用事务 |

**N+1 查询示例**：

```typescript
// ❌ 错误：N+1 查询
const users = await getUsers();
for (const user of users) {
  const orders = await getOrdersByUserId(user.id); // 每个用户一次查询
}

// ✅ 正确：批量查询
const users = await getUsers();
const userIds = users.map(u => u.id);
const orders = await getOrdersByUserIds(userIds); // 一次查询所有
```

### 前端性能（强制 MUST）

| 场景 | 解决方案 |
|------|----------|
| 列表 100+ 项 | 使用虚拟化 (react-virtual, tanstack-virtual) |
| 路由和重组件 | 实现代码分割 |
| 图片和大资源 | 懒加载 |
| 昂贵计算 | 使用 `useMemo`/`useCallback`（但不要滥用） |

### 内存管理（强制 MUST）

| 规则 | 说明 |
|------|------|
| 清理副作用 | 在 useEffect 清理函数中清理事件监听器、订阅和定时器 |
| 避免内存泄漏 | 正确释放资源 |
| 合理使用状态 | 不要不必要地在状态中存储大对象 |

**清理示例**：

```typescript
useEffect(() => {
  const handler = () => { /* ... */ };
  window.addEventListener('resize', handler);
  
  return () => {
    window.removeEventListener('resize', handler); // ✅ 清理
  };
}, []);
```

### 包大小（强制 MUST）

| 规则 | 说明 |
|------|------|
| 谨慎添加依赖 | 添加新依赖前检查包大小影响 |
| 优先 Tree-shakeable | 优先选择支持 tree-shaking 的库 |
| 大包告警 | 如果添加的依赖 > 50KB gzipped，必须告警用户 |

### 不需要过早优化的场景

以下情况**不需要**过早优化：

- 简单的值比较
- 小数组（< 100 项）
- 不频繁渲染的组件
- 一次性计算

### 性能检查清单

```
□ 无 N+1 查询模式
□ 大列表使用虚拟化
□ 资源正确清理
□ 新依赖大小合理
□ 适当使用缓存和记忆化
```
</performance_awareness>
