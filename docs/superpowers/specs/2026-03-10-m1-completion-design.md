# M1 Completion Design

## Overview
Complete remaining M1 tasks: Gemini LLM integration, 证件管理 pipeline enhancement, frontend fixes, and E2E tests.

**Status:** Implemented (2026-03-10)

## 1. Gemini API Integration

Replaced OpenAI-compatible LLM executor with Google AI Studio (Gemini) format.

**Models selected:**
- **Gemini 3.1 Pro** (`gemini-3.1-pro`) — text understanding and vision tasks (todo parsing, OCR extraction)
- **Gemini 3.1 Flash Image** (`gemini-3.1-flash-image`) — image generation/processing (document standardization)

**Three operating modes** (specified via `runtime_config.mode`):
- **`text`** (default) — text-to-text generation. Used by `todo_parse` capability.
- **`vision`** — image+text input → text output. Used by `ocr_extract` for reading document images.
- **`image_generation`** — image+text input → image+text output. Used by `image_process` for document standardization. Sets `responseModalities: ["TEXT", "IMAGE"]`.

**Config:**
- `LLM_API_KEY` — Gemini API key (env var, defaults to empty string for dev/test)
- `LLM_API_URL` — defaults to `https://generativelanguage.googleapis.com`
- Model specified per-capability in `runtime_config.model`

**Auth:** `?key=API_KEY` query parameter (Google AI Studio style).

**Request format (Gemini generateContent):**
```json
{
  "contents": [
    {"role": "user", "parts": [{"text": "..."}, {"inline_data": {"mime_type": "image/jpeg", "data": "base64..."}}]}
  ],
  "systemInstruction": {"parts": [{"text": "..."}]},
  "generationConfig": {
    "temperature": 0.1,
    "responseModalities": ["TEXT"]
  }
}
```

**Implementation files:**
- `server/src/tool/pipeline/gemini.rs` — Gemini API types, request builders, response extractors (6 unit tests)
- `server/src/tool/pipeline/executor.rs` — `execute_remote_llm()` rewritten for Gemini
- `server/src/common/config.rs` — LLM config defaults (no panic if unset)

## 2. 证件管理 Pipeline (5 steps)

Updated from 4 steps to 5 steps with document image standardization:

| Step | Capability | Model | Mode | Description |
|------|-----------|-------|------|-------------|
| 1 | `image_upload` | — (builtin) | — | Capture raw document image |
| 2 | `ocr_extract` | gemini-3.1-pro | vision | Extract cert_type, cert_number, full_name, expiry_date, issuing_country |
| 3 | `image_process` | gemini-3.1-flash-image | image_generation | Contrast enhancement + crop → standardized document image |
| 4 | `data_object_write` | — (builtin) | — | Persist structured data + processed image reference |
| 5 | `reminder_schedule` | — (builtin) | — | Create reminder based on expiry_date |

**Key design decisions:**
- `image_process` uses `on_failure: skip` — pipeline continues even if image processing fails
- `ocr_extract` receives raw image as `image_base64` + `mime_type` from step 1 output
- `data_object_write` receives OCR data as primary input, processed image as secondary

**Migration:** `server/migrations/20260310000018_gemini_seed_update.sql`

## 3. Todo List Pipeline (4 steps, updated)

| Step | Capability | Model | Mode |
|------|-----------|-------|------|
| 1 | `text_input` | — (builtin) | — |
| 2 | `todo_parse` | gemini-3.1-pro | text |
| 3 | `data_object_write` | — (builtin) | — |
| 4 | `reminder_schedule` | — (builtin) | — |

Input mappings updated for consistent key naming (`text`, `data`).

## 4. Frontend Fixes

**File upload endpoint fix:**
- `uploadFile()` changed from `/api/data-objects/:id/files` to `/api/files/upload`
- `getFileUrl()` changed from `/api/files/:id/download` to `/api/files/:id`
- Parameters updated: `(file, dataObjectId?, role?)` — all optional except file

**DocUploadPage enhancement:**
- After pipeline completion, fetches created data object and displays extracted fields:
  - 证件类型 (cert_type), 证件号码 (cert_number), 姓名 (full_name), 有效期 (expiry_date), 签发国家 (issuing_country)
- "Upload Another" button to reset form
- File input restricted to `accept="image/*"`
- Base64 data URL prefix stripped before sending to API

**Reminder API fix (discovered during E2E testing):**
- Frontend was sending `due_at` but backend expects `trigger_at`
- Fixed in `web/src/api/types.ts` and `web/src/pages/RemindersPage.tsx`

## 5. E2E Tests (Playwright)

**12 tests across 5 test files**, all passing:

| File | Tests | Description |
|------|-------|-------------|
| `auth.spec.ts` | 3 | Redirect, valid login, invalid login |
| `todo.spec.ts` | 3 | Navigate, open form, submit + pipeline status |
| `document.spec.ts` | 2 | Navigate, drop zone visible |
| `search.spec.ts` | 2 | Search input, no results for nonexistent query |
| `reminders.spec.ts` | 2 | Navigate, create reminder |

**Report output:**
- HTML report: `docs/reports/e2e-report/index.html`
- JSON results: `docs/reports/e2e-results.json`

**Configuration:** `web/playwright.config.ts`, runs against `http://localhost:9527` (Docker).

**Note:** E2E tests do not mock LLM calls. Pipeline tests rely on the app's `on_failure` behavior to handle missing LLM API keys gracefully.

## Test Summary

| Suite | Count | Status |
|-------|-------|--------|
| Backend integration (Rust) | 13 | All pass |
| Frontend unit (Vitest) | 11 | All pass |
| Gemini module unit (Rust) | 6 | All pass |
| E2E (Playwright) | 12 | All pass |
| **Total** | **42** | **All pass** |
