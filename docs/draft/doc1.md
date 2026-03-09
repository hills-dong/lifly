# Litly Frame — System Architecture

## Overview

Litly Frame 是一个以用户自托管服务器为核心的多端数据采集与 LLM 处理框架。所有客户端将数据汇聚至中心服务器，由服务器完成存储与本地 LLM 推理。

-----

## Components

### 1. Self-Host Server（核心节点）

- **部署形式**：VPS / NAS Server（用户自建）
- **职责**：
  - Store Data：持久化存储来自所有客户端的数据
  - LLM Process：在本地运行大语言模型推理，无需外部 AI API
  - 统一接收所有客户端的数据写入请求
  - 向 Web / Desktop 客户端提供数据查询响应

### 2. Web Client

- **形态**：浏览器访问
- **职责**：
  - Collect Data：采集用户输入数据
  - Manage Data：提供数据管理界面
- **与服务端关系**：双向（读写）

### 3. Desktop Client

- **平台**：Linux / Windows / macOS
- **职责**：
  - Collect Data：本地数据采集
  - Manage Data：本地数据管理
- **与服务端关系**：双向（读写）

### 4. Remote Mobile Client

- **平台**：iPhone / Android
- **职责**：
  - Collect Data：移动端数据采集
- **与服务端关系**：单向（只写）

### 5. Software Plugins

- **形态**：Chrome Extension 及其他浏览器/系统插件
- **职责**：
  - Collect Data：从浏览器行为或系统事件中采集数据
  - 将数据推送至 Self-Host Server
- **与服务端关系**：单向（只写）

-----

## Data Flow

```
Web Client        ⇄  Self-Host Server   (读写双向)
Desktop Client    ⇄  Self-Host Server   (读写双向)
Mobile Client     →  Self-Host Server   (只写)
Software Plugin   →  Self-Host Server   (只写)
Self-Host Server  →  LLM Process        (内部调用)
```

-----

## Architecture Principles

1. **数据主权**：所有数据存储在用户自建服务器，不经过第三方云服务。
1. **本地推理**：LLM 运行在服务端本地，数据不外泄。
1. **多端采集**：四类客户端覆盖 Web、桌面、移动、插件场景，统一汇聚至服务端。
1. **读写分离**：仅 Web 和 Desktop 具备数据管理（读）能力；Mobile 和 Plugin 仅负责采集（写）。