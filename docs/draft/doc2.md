# Litly Frame — Data Flow

## Overview

数据流分为四个顺序阶段：采集 → 处理 → 存储 → 使用。

```
Clients/Plugins
     │
     ▼
1. Collect Data
     │  pic / video / record / text / log / ...
     │  watch / phone / laptop / software / ...
     ▼
2. Process Data  (Server)
     │  LLM + Human Review
     │  输出格式：Folder / File / Markdown / JSON / SQL DB
     ▼
3. Store Data  (Server)
     │  Folder / File / DB / Markdown / JSON / ...
     ▼
4. Use Data
        View / Remind / Share / Suggestion / ...
```

-----

## Stage 1 · Collect Data（客户端 / 插件）

**触发方**：Clients + Plugins（所有端）

采集的数据类型：

- `pic` — 图片
- `video` — 视频
- `record` — 录音
- `text` — 文字
- `log` — 系统/行为日志
- `...` — 其他扩展类型

采集来源设备：

- `watch` — 智能手表
- `phone` — 手机
- `laptop` — 笔记本电脑
- `software` — 软件插件（Chrome Extension 等）
- `...` — 其他设备

-----

## Stage 2 · Process Data（Server）

**执行方**：Self-Host Server

处理方式：

- `LLM` — 大语言模型自动处理
- `Human` — 人工介入审核/标注（LLM + Human 协作）

处理输出格式（结构化）：

- `Folder / File` — 文件系统组织
- `Markdown` — 结构化文档
- `JSON` — 结构化数据
- `SQL DB` — 关系型数据库

> 注：此阶段原稿有删除线，说明存储格式在 Process 阶段仅作中间态转换，最终持久化由 Stage 3 完成。

-----

## Stage 3 · Store Data（Server）

**执行方**：Self-Host Server

持久化存储格式：

- `Folder / File` — 文件系统
- `DB` — 数据库（SQL 或其他）
- `Markdown` — 文档形式存储
- `JSON` — 结构化数据存储
- `...` — 其他格式扩展

-----

## Stage 4 · Use Data

**消费方**：用户（Person）

使用方式：

- `View` — 浏览查看历史数据
- `Remind` — 提醒/回顾
- `Share` — 分享给他人
- `Suggestion` — 由 LLM 基于数据给出建议
- `...` — 其他扩展用途

-----

## Key Observations

1. **LLM 在 Stage 2 介入**：原始数据经 LLM 处理后才入库，入库的是结构化/语义化数据，而非原始 raw data。
1. **Human-in-the-loop**：Stage 2 明确标注 LLM + Human，说明系统设计上保留人工审核节点。
1. **存储格式多元**：同时支持文件系统、数据库、Markdown、JSON，适配不同查询和展示场景。
1. **Use Data 面向用户**：最终消费层强调 View / Remind / Share / Suggestion，是典型的个人知识管理（PKM）产品形态。