---
name: platform-build
description: |
  平台构建打包开发规范。

  触发场景：
  - Electron 打包
  - 安装包生成
  - 原生模块编译
  - 跨平台构建

  触发词：打包、构建、electron-builder、安装包、dmg、exe、deb、asar、原生模块
---

# 平台构建打包开发规范

## 技术架构

```
electron-builder.yml        # electron-builder 配置
package.json               # 打包脚本
scripts/
├── afterPack.cjs           # 打包后处理
├── download-ocr-model.cjs  # OCR 模型下载
├── download-sherpa-model.cjs # 语音模型下载
└── generate-icons.js       # 图标生成
build/                      # 构建资源
├── icons/                  # 图标文件
└── ...
```

## 核心配置

### electron-builder.yml

```yaml
appId: com.zhangnote.app
productName: ZhangNote

directories:
  output: release

files:
  - from: dist
    to: .

asarUnpack:
  - "**/node_modules/@lancedb/**/*"
  - "**/node_modules/apache-arrow/**/*"
  - "**/node_modules/flatbuffers/**/*"
  - "**/node_modules/onnxruntime-node/**/*"
  - "**/node_modules/onnxruntime-common/**/*"
  - "**/node_modules/canvas/**/*"
  - "**/node_modules/@paddlejs-models/**/*"

win:
  target: nsis
  icon: build/icons/icon.ico
  artifactName: ZhangNote-${version}-${arch}.${ext}

mac:
  target: dmg
  icon: build/icons/icon.icns
  artifactName: ZhangNote-${version}-${arch}.${ext}

linux:
  target: AppImage
  icon: build/icons/256x256.png
```

### package.json 脚本

```json
{
  "scripts": {
    "build": "vite build",
    "build:electron": "tsc -p electron/tsconfig.json",
    "dev:electron": "npm run build && electron .",
    "dist:win": "npm run build:electron && electron-builder --win",
    "dist:mac": "npm run build:electron && electron-builder --mac",
    "dist:linux": "npm run build:electron && electron-builder --linux"
  }
}
```

## 核心规范

### 原生模块处理

**问题**：原生模块（better-sqlite3, canvas, onnxruntime, lancedb）在打包后可能无法加载

**解决方案**：使用 afterPack.cjs 钩子复制纯 JS 依赖

```javascript
// scripts/afterPack.cjs
const fs = require('fs');
const path = require('path');

const modulesToCopy = [
  'apache-arrow',
  'flatbuffers',
  'tslib',
  'onnxruntime-common',
  'json-bignum'
];

const criticalModules = ['apache-arrow', 'flatbuffers', 'onnxruntime-common'];

exports.default = async function(context) {
  const appOutDir = context.appOutDir;
  const resourcesDir = path.join(appOutDir, 'resources');
  const unpackedModules = path.join(resourcesDir, 'app.asar.unpacked', 'node_modules');
  const sourceModules = path.join(process.cwd(), 'node_modules');

  console.log('[afterPack] Copying pure JS modules...');

  for (const moduleName of modulesToCopy) {
    const src = path.join(sourceModules, moduleName);
    const dest = path.join(unpackedModules, moduleName);

    if (fs.existsSync(src)) {
      fs.cpSync(src, dest, { recursive: true });
      console.log(`[OK] ${moduleName}`);
    }
  }

  // 验证关键模块
  for (const mod of criticalModules) {
    if (fs.existsSync(path.join(unpackedModules, mod))) {
      console.log(`[VERIFIED] ${mod}`);
    } else {
      console.error(`[MISSING] ${mod}`);
    }
  }
};
```

### 资源文件打包

```typescript
// electron/main.ts
import { app } from 'electron';
import path from 'path';

// 打包后的资源路径处理
const getResourcePath = (...relativePath: string[]) => {
  if (app.isPackaged) {
    return path.join(process.resourcesPath, ...relativePath);
  }
  return path.join(process.cwd(), 'resources', ...relativePath);
};

// OCR 模型
const ocrModelPath = getResourcePath('ocr-models');

// 语音模型
const sherpaModelPath = getResourcePath('sherpa-models');
```

### 平台检测

```typescript
// src/services/platform/platformService.ts
export type OS = 'windows' | 'darwin' | 'linux' | 'android' | 'ios' | 'web';

export function getPlatform(): Platform {
  if (typeof window !== 'undefined' && window.electronAPI) {
    const osMap = { 'win32': 'windows', 'darwin': 'darwin', 'linux': 'linux' };
    return {
      isElectron: true,
      isMobile: false,
      isWeb: false,
      os: osMap[window.electronAPI.platform.os] || 'linux'
    };
  }
  // ...
}
```

## 构建产物

| 平台 | 格式 | 文件名 |
|------|------|--------|
| Windows | NSIS 安装包 | ZhangNote-1.6.0-x64.exe |
| macOS | DMG | ZhangNote-1.6.0-arm64.dmg |
| Linux | AppImage | ZhangNote-1.6.0-x86_64.AppImage |

## 禁止事项

- ❌ 禁止在 asar 中包含原生模块
- ❌ 禁止忽略 afterPack 钩子
- ❌ 禁止硬编码资源路径
- ❌ 禁止使用开发环境的绝对路径

## 参考代码

- `electron-builder.yml` - 打包配置
- `scripts/afterPack.cjs` - 打包后处理
- `electron/main.ts` - 主进程入口
- `src/services/platform/platformService.ts` - 平台检测

## 检查清单

- [ ] 是否配置了正确的 asarUnpack
- [ ] afterPack 是否正确复制所有依赖
- [ ] 资源路径是否使用跨平台写法
- [ ] 是否处理了开发/打包环境差异
- [ ] 是否测试了所有平台的打包产物
