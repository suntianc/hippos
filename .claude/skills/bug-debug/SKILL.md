---
name: bug-debug
description: |
  问题排查调试指南。

  触发场景：
  - 排查代码错误
  - 调试运行问题
  - 分析崩溃原因
  - 性能问题定位

  触发词：Bug、报错、错误、异常、调试、排查、问题、崩溃、performance、慢
---

# 问题排查调试指南

## 常见问题分类

### 1. Electron 相关问题

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 应用无法启动 | 原生模块加载失败 | 检查 afterPack 配置 |
| IPC 调用失败 | preload 未正确加载 | 检查 contextBridge |
| 数据库连接失败 | 路径错误 | 检查 userData 路径 |
| 打包后功能失效 | asar 读取问题 | 检查 asarUnpack |

### 2. React 相关问题

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 组件不渲染 | 状态未更新 | 检查 useEffect 依赖 |
| hooks 警告 | 规则违反 | 检查 hooks 规则 |
| 类型错误 | TypeScript 配置 | 检查 tsconfig |
| 性能问题 | 过多重渲染 | 使用 useMemo/useCallback |

### 3. 构建相关问题

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 原生模块编译失败 | 缺少编译工具 | 安装 node-gyp |
| 打包后崩溃 | 路径问题 | 检查资源路径 |
| 图标不显示 | 格式问题 | 转换正确格式 |

## 调试技巧

### 1. 开发模式调试

```bash
# 启动开发模式，查看详细日志
npm run dev

# 启动 Electron 开发模式
npm run dev:electron
```

### 2. 使用开发者工具

```typescript
// 在代码中添加调试日志
console.log('[Debug] State:', state);
console.log('[Debug] Props:', props);

// 使用断点
debugger;
```

### 3. 日志文件位置

| 环境 | 日志路径 |
|------|----------|
| Windows | `%APPDATA%/tashanstone/logs/` |
| macOS | `~/Library/Application Support/tashanstone/logs/` |
| Linux | `~/.config/tashanstone/logs/` |

### 4. Memory 调试

Memory 文件存储在 `%APPDATA%/tashanstone/.memories/`：

```typescript
// 调试 Memory 保存
console.log('[Memory] 调用 update IPC, id:', memory.id);
console.log('[Memory] update 返回值:', JSON.stringify(result));

// 常见问题：保存成功但内容未更新
// 原因：保存后未同步更新本地状态
// 解决：在 IPC 成功后调用 setPreviewMemory()
if (result?.success) {
  setPreviewMemory(prev => prev ? {
    ...prev,
    content: memory.content,
    updatedAt: Date.now()
  } : null);
}
```

### 5. 常见错误排查

#### 错误：Cannot find module 'better-sqlite3'

```bash
# 解决方案：重新安装
npm rebuild better-sqlite3
```

#### 错误：Electron failed to install correctly

```bash
# 解决方案：删除重新安装
rm -rf node_modules
npm install
```

#### 错误：Lancedb initialization failed

```javascript
// 检查打包环境路径配置
if (app.isPackaged) {
  const resourcesPath = process.resourcesPath;
  const unpackedPath = path.join(resourcesPath, 'app.asar.unpacked');
  module.paths.unshift(path.join(unpackedPath, 'node_modules'));
}
```

## 性能问题排查

### 使用 Chrome DevTools Profiler

1. 打开应用开发者工具
2. 进入 Performance 标签
3. 录制操作
4. 分析性能瓶颈

### 常见性能问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 内存泄漏 | 未清理监听器 | useEffect 返回清理函数 |
| 卡顿 | 大文件渲染 | 使用虚拟滚动 |
| 启动慢 | 过多同步操作 | 延迟加载非关键模块 |

## 代码审查清单

在提交代码前检查：

- [ ] 所有 console.log 是否有必要
- [ ] 是否正确处理了异步错误
- [ ] 是否避免了内存泄漏
- [ ] 是否有性能瓶颈
- [ ] 是否有未处理的边界情况

## 日志规范

```typescript
// 使用结构化日志
const logger = {
  info: (msg: string, data?: any) => 
    console.log(`[INFO] ${msg}`, data || ''),
  error: (msg: string, error?: any) => 
    console.error(`[ERROR] ${msg}`, error || ''),
  warn: (msg: string, data?: any) => 
    console.warn(`[WARN] ${msg}`, data || '')
};

// 使用
logger.info('File saved', { fileId, name });
logger.error('Failed to save', error);
```

## 参考文档

- [Electron 调试指南](https://www.electronjs.org/docs/latest/tutorial/debugging-main-process)
- [React DevTools](https://react.dev/learn/react-developer-tools)
- [Chrome DevTools](https://developer.chrome.com/docs/devtools/)

## 检查清单

- [ ] 是否提供了足够的错误上下文
- [ ] 是否区分了错误级别
- [ ] 是否避免了敏感信息泄露
- [ ] 是否有性能监控
