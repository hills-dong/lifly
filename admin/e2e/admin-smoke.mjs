// Standalone Playwright smoke test for the Lifly admin panel.
// Run from a dir where `@playwright/test` resolves (e.g. ../web):
//   node /abs/path/admin-smoke.mjs
// Requires the backend (serving admin/dist) reachable at BASE.
import { chromium } from "@playwright/test";

// Page base (where the SPA is served) vs API base (always at root /api/...).
// When the panel is hosted under /admin, set ADMIN_BASE=http://host/admin and
// API_BASE=http://host.
const BASE = process.env.ADMIN_BASE || "http://localhost:8090";
const API_BASE = process.env.API_BASE || BASE;
const SHOT_DIR = process.env.SHOT_DIR || "/home/hills/projects/lifly/docs/reports";

function assert(cond, msg) {
  if (!cond) throw new Error("ASSERT FAILED: " + msg);
}

// Fetch a real user id straight from the API (to satisfy the reminders FK).
async function fetchUserId() {
  const login = await fetch(`${API_BASE}/api/admin/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username: "admin", password: "admin123" }),
  }).then((r) => r.json());
  const token = login.data.token;
  const users = await fetch(`${API_BASE}/api/admin/data/users?per_page=1`, {
    headers: { Authorization: `Bearer ${token}` },
  }).then((r) => r.json());
  return users.data.items[0].id;
}

const run = async () => {
  const userId = await fetchUserId();
  const title = `e2e-ui-reminder-${Date.now()}`;

  const browser = await chromium.launch();
  const page = await browser.newPage();
  page.on("dialog", (d) => d.accept()); // auto-accept delete confirm()

  // 1) Login
  await page.goto(`${BASE}/login`);
  await page.fill('input[name="username"]', "admin");
  await page.fill('input[name="password"]', "admin123");
  await page.click('button[type="submit"]');

  // 2) Dashboard — 14 tables
  await page.waitForSelector("text=数据表");
  const cards = await page.locator('a[href*="/r/"]').count();
  assert(cards >= 14, `expected >=14 resource links on dashboard, got ${cards}`);
  await page.screenshot({ path: `${SHOT_DIR}/admin-dashboard.png`, fullPage: true });

  // 3) users list — has rows, hides password_hash
  await page.goto(`${BASE}/r/users`);
  await page.waitForSelector('[data-testid="resource-table"]');
  const headers = await page.locator('[data-testid="resource-table"] thead th').allInnerTexts();
  assert(headers.includes("username"), "users table should show username column");
  assert(!headers.includes("password_hash"), "users table must NOT show password_hash");
  const userRows = await page.locator('[data-testid="resource-row"]').count();
  assert(userRows >= 1, `expected >=1 user row, got ${userRows}`);
  await page.screenshot({ path: `${SHOT_DIR}/admin-users.png`, fullPage: true });

  // 4) CREATE a reminder via the UI
  await page.goto(`${BASE}/r/reminders/create`);
  await page.waitForSelector('input[name="title"]');
  await page.fill('input[name="user_id"]', userId);
  await page.fill('input[name="title"]', title);
  await page.fill('input[name="trigger_at"]', "2099-01-01T00:00:00Z");
  await page.fill('input[name="status"]', "pending");
  await page.click('button[type="submit"]');

  // back on the list — the new title should be visible
  await page.waitForSelector('[data-testid="resource-table"]');
  await page.waitForSelector(`text=${title}`);
  assert(await page.getByText(title).first().isVisible(), "created reminder should appear in list");
  await page.screenshot({ path: `${SHOT_DIR}/admin-reminder-created.png`, fullPage: true });

  // 5) EDIT that reminder's title
  const updated = `${title}-edited`;
  const row = page.locator('[data-testid="resource-row"]', { hasText: title });
  await row.first().getByText("编辑").click();
  await page.waitForSelector('input[name="title"]');
  await page.fill('input[name="title"]', updated);
  await page.click('button[type="submit"]');
  await page.waitForSelector(`text=${updated}`);
  assert(await page.getByText(updated).first().isVisible(), "edited title should appear in list");

  // 6) DELETE it
  const row2 = page.locator('[data-testid="resource-row"]', { hasText: updated });
  await row2.first().getByText("删除").click();
  await page.waitForSelector(`text=${updated}`, { state: "detached" });
  const stillThere = await page.getByText(updated).count();
  assert(stillThere === 0, "deleted reminder should be gone from list");

  await browser.close();
  console.log("ADMIN E2E PASSED ✓ (login, dashboard×14, users hides password_hash, reminder create/edit/delete)");
};

run().catch((e) => {
  console.error(e);
  process.exit(1);
});
