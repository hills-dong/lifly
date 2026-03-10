import { type Page, type APIRequestContext, expect } from '@playwright/test';

export async function login(page: Page) {
  await page.goto('/login');
  await page.fill('#username', 'admin');
  await page.fill('#password', 'admin123');
  await page.click('button[type="submit"]');
  await expect(page).not.toHaveURL(/login/, { timeout: 10_000 });
}

/** Login via API and return JWT token. */
export async function apiLogin(request: APIRequestContext, baseURL: string): Promise<string> {
  const resp = await request.post(`${baseURL}/api/auth/login`, {
    data: { username: 'admin', password: 'admin123' },
  });
  const body = await resp.json();
  return body.data.token;
}

/** Submit a todo via API and wait for its pipeline to complete. Returns the pipeline_id. */
export async function createTodoViaApi(
  request: APIRequestContext,
  baseURL: string,
  token: string,
  content: string,
): Promise<string> {
  const toolId = '00000000-0000-0000-0000-000000000201';
  const resp = await request.post(`${baseURL}/api/raw-inputs`, {
    headers: { Authorization: `Bearer ${token}` },
    data: { tool_id: toolId, input_type: 'text', raw_content: content },
  });
  const body = await resp.json();
  const pipelineId = body.data.pipeline_id;

  // Poll until pipeline completes (max 15s)
  for (let i = 0; i < 15; i++) {
    await new Promise((r) => setTimeout(r, 1000));
    const pr = await request.get(`${baseURL}/api/pipelines/${pipelineId}`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    const pb = await pr.json();
    const status = pb.data?.status;
    if (status === 'completed' || status === 'failed') break;
  }

  return pipelineId;
}
