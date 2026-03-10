# M1 Full Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the complete M1 milestone — skeleton, core pipeline, scenario validation (Todo + ID docs), and React frontend.

**Architecture:** Rust/Axum backend with SQLx + PostgreSQL (pgvector), React frontend served as static files, docker-compose deployment. 14 database entities across 4 domains + cross-cutting identity.

**Tech Stack:** Rust, Axum, SQLx, PostgreSQL + pgvector, React, TypeScript, pnpm, Docker, GitHub Actions

---

## Phase 1: Basic Skeleton (P1)

### Task 1: Cargo workspace + Axum skeleton
### Task 2: SQLx migrations (14 tables)
### Task 3: Common modules (error, response, config, middleware)
### Task 4: docker-compose setup
### Task 5: React project initialization
### Task 6: GitHub Actions CI

## Phase 2: Core Pipeline (P2)

### Task 7: Identity module (User auth + Device)
### Task 8: Capability module (AtomicCapability + CapabilityParam)
### Task 9: Tool definition module (Tool + ToolVersion + ToolStep)
### Task 10: Data module (DataObject + FileStorage + Category)
### Task 11: Pipeline execution engine
### Task 12: RawInput + Pipeline trigger
### Task 13: Reminder module
### Task 14: WebSocket push
### Task 15: File upload/download

## Phase 3: Scenario Validation (P3)

### Task 16: Remote LLM runtime
### Task 17: Todo tool seed + end-to-end
### Task 18: ID document tool seed + end-to-end
### Task 19: Vector search
### Task 20: Integration tests

## Phase 4: Frontend + Integration (P4)

### Task 21: Layout + routing + API client
### Task 22: Login page
### Task 23: Tool list / home
### Task 24: Todo frontend (input, list, detail, edit)
### Task 25: ID document frontend (upload, list, detail, filter)
### Task 26: Pipeline status (WebSocket)
### Task 27: Reminder list + notifications
### Task 28: Search page
### Task 29: E2E tests
