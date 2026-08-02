import { readFileSync } from "node:fs";
import { test } from "node:test";
import vm from "node:vm";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import path from "node:path";

const frontendMain = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "frontend",
  "main.js",
);

function createStubElement(selector) {
  const element = {
    selector,
    textContent: "",
    innerHTML: "",
    className: "",
    title: "",
    disabled: false,
    open: false,
    style: {},
    dataset: {},
    classList: {
      add: (...names) => {
        const current = new Set(element.className.split(/\s+/).filter(Boolean));
        names.forEach((name) => current.add(name));
        element.className = [...current].join(" ");
      },
      remove: (...names) => {
        const drop = new Set(names);
        element.className = element.className
          .split(/\s+/)
          .filter((name) => name && !drop.has(name))
          .join(" ");
      },
      toggle: (name, force) => {
        const has = element.className.split(/\s+/).includes(name);
        const next = force === undefined ? !has : Boolean(force);
        element.classList[next ? "add" : "remove"](name);
        return next;
      },
      contains: (name) => element.className.split(/\s+/).includes(name),
    },
    listeners: {},
    addEventListener: (type, handler) => {
      element.listeners[type] = handler;
    },
    removeEventListener: () => {},
    querySelector: (child) => createStubElement(`${selector} ${child}`),
    querySelectorAll: () => [],
    children: [],
    replaceChildren: (...nodes) => {
      element.children = nodes;
    },
    append: (...nodes) => {
      element.children.push(...nodes);
    },
    appendChild: (child) => {
      element.children.push(child);
      return child;
    },
    remove: () => {},
    setAttribute: () => {},
    getAttribute: () => null,
    scrollIntoView: () => {},
    click: () => element.listeners.click?.({ target: element }),
    focus: () => {},
  };
  return element;
}

function loadMain({ invoke } = {}) {
  const elements = new Map();
  const get = (selector) => {
    if (!elements.has(selector)) elements.set(selector, createStubElement(selector));
    return elements.get(selector);
  };
  const documentStub = {
    querySelector: (selector) => get(selector),
    querySelectorAll: () => [],
    addEventListener: () => {},
    createElement: () => createStubElement("dynamic"),
    body: createStubElement("body"),
  };
  const windowStub = {
    __TAURI__: invoke ? { core: { invoke }, event: { listen: async () => {} } } : undefined,
    addEventListener: () => {},
    location: { hash: "" },
  };
  const context = vm.createContext({
    document: documentStub,
    window: windowStub,
    console,
    setTimeout,
    clearTimeout,
    Date,
    JSON,
    Number,
    Promise,
  });
  vm.runInContext(readFileSync(frontendMain, "utf8"), context, { filename: "main.js" });
  return { context, get };
}

const chatgptStatus = {
  schemaVersion: 1,
  loginState: "chatgpt",
  authMode: "chatgpt",
  authPath: "/Users/test/.codex/auth.json",
  lastRefresh: "2026-08-01T10:00:00Z",
  profile: {
    email: "tester@example.com",
    name: "体验者",
    planType: "plus",
    accountId: "acct-1",
    tokenExpiresAt: "2030-03-25T08:46:40+00:00",
  },
  snapshot: {
    schemaVersion: 1,
    importedAt: "2026-08-02T03:00:00Z",
    email: "tester@example.com",
    name: "体验者",
    planType: "plus",
    accountId: "acct-1",
    usage: {
      fetchedAt: "2026-08-02T03:00:00Z",
      allowed: true,
      limitReached: false,
      primaryWindow: {
        usedPercent: 8,
        limitWindowSeconds: 604800,
        resetAfterSeconds: 500000,
        resetAt: "2026-08-08T03:00:00Z",
      },
      secondaryWindow: null,
      credits: { hasCredits: false, unlimited: false, balance: "0.00" },
    },
  },
  snapshotPath: "/Users/test/Library/Application Support/CodexAssistant/runtime/account-snapshot.json",
  localData: {
    sessionCount: 43,
    archivedSessionCount: 1,
    latestSessionAt: "2026-08-02T03:30:00Z",
    recentThreads: [
      { id: "t-1", name: "做一个产品介绍 PPT", updatedAt: "2026-08-02T03:00:00Z" },
      { id: "t-2", name: "", updatedAt: "2026-08-01T10:00:00Z" },
    ],
    totalBytes: 178_257_920,
    sessionsBytes: 2_621_440,
    logsBytes: 155_189_248,
    codexHome: "/Users/test/.codex",
  },
  message: "已把当前账号信息导入到本地",
};

test("账号页渲染已登录账号与用量", async () => {
  const { context, get } = loadMain({
    invoke: async (command) => {
      assert.equal(command, "get_codex_account_status");
      return chatgptStatus;
    },
  });
  await context.refreshAccountStatus();
  await Promise.resolve();

  assert.equal(get("#accountLoginBadge").textContent, "已登录");
  assert.ok(get("#accountLoginBadge").className.includes("success"));
  assert.equal(get("#accountStatusTitle").textContent, "当前账号：tester@example.com");
  assert.equal(get("#accountAuthMode").textContent, "ChatGPT 账号");
  assert.equal(get("#accountEmail").textContent, "tester@example.com");
  assert.equal(get("#accountName").textContent, "体验者");
  assert.equal(get("#accountPlan").textContent, "ChatGPT Plus");
  assert.equal(get("#accountId").textContent, "acct-1");
  assert.equal(get("#primaryWindowPercent").textContent, "已用 8%");
  assert.equal(get("#primaryWindowBar").style.width, "8%");
  assert.ok(get("#primaryWindowReset").textContent.includes("窗口 7 天"));
  assert.ok(get("#secondaryWindowRow").className.includes("hidden"));
  assert.ok(get("#accountCreditsRow").className.includes("hidden"));
  assert.equal(get("#accountUsageBadge").textContent, "正常");
  assert.notEqual(get("#accountSnapshotTime").textContent, "尚未导入");
  assert.ok(get("#accountSnapshotPath").textContent.includes("account-snapshot.json"));
  assert.equal(get("#importAccountButton").disabled, false);
  assert.equal(get("#importAccountLabel").textContent, "重新导入");
});

test("账号页渲染本地数据概览", async () => {
  const { context, get } = loadMain({
    invoke: async (command) => {
      if (command === "get_codex_account_status") return chatgptStatus;
      throw new Error(`unexpected command ${command}`);
    },
  });
  await context.refreshAccountStatus();
  await Promise.resolve();

  assert.equal(get("#localDataBadge").textContent, "43 个会话");
  assert.equal(get("#localSessionCount").textContent, "43 个");
  assert.equal(get("#localArchivedCount").textContent, "1 个");
  assert.notEqual(get("#localLatestSession").textContent, "未知");
  assert.equal(get("#localStorageBytes").textContent, "170.0 MB");
  assert.ok(get("#localStorageBytes").title.includes("会话 2.5 MB"));
  assert.equal(get("#localDataHome").textContent, "/Users/test/.codex");

  const list = get("#recentThreadList");
  assert.equal(list.children.length, 2);
  assert.equal(list.children[0].children[0].textContent, "做一个产品介绍 PPT");
  assert.equal(list.children[1].children[0].textContent, "未命名会话");
  assert.ok(get("#recentThreadEmpty").className.includes("hidden"));
});

test("本地数据为空时显示零态", async () => {
  const empty = {
    ...chatgptStatus,
    localData: {
      sessionCount: 0,
      archivedSessionCount: 0,
      latestSessionAt: null,
      recentThreads: [],
      totalBytes: 0,
      sessionsBytes: 0,
      logsBytes: 0,
      codexHome: "/Users/test/.codex",
    },
  };
  const { context, get } = loadMain({
    invoke: async (command) => {
      if (command === "get_codex_account_status") return empty;
      throw new Error(`unexpected command ${command}`);
    },
  });
  await context.refreshAccountStatus();
  await Promise.resolve();
  assert.equal(get("#localDataBadge").textContent, "0 个会话");
  assert.equal(get("#localArchivedCount").textContent, "无");
  assert.equal(get("#localStorageBytes").textContent, "0 B");
  assert.equal(get("#recentThreadList").children.length, 0);
  assert.ok(!get("#recentThreadEmpty").className.includes("hidden"));
});

test("导入按钮触发 import_codex_account 并更新页面", async () => {
  const calls = [];
  const { context, get } = loadMain({
    invoke: async (command) => {
      calls.push(command);
      if (command === "import_codex_account" || command === "get_codex_account_status") {
        return chatgptStatus;
      }
      throw new Error(`unexpected command ${command}`);
    },
  });
  await context.importAccount();
  await Promise.resolve();
  assert.deepEqual(calls.filter((command) => command === "import_codex_account"), [
    "import_codex_account",
  ]);
  assert.equal(get("#accountStatusTitle").textContent, "当前账号：tester@example.com");
});

test("导入失败保留页面数据并显示错误", async () => {
  const { context, get } = loadMain({
    invoke: async (command) => {
      if (command === "get_codex_account_status") return chatgptStatus;
      if (command === "import_codex_account") {
        throw { title: "导入失败", message: "Codex 登录状态已过期，请运行 codex login" };
      }
      throw new Error(`unexpected command ${command}`);
    },
  });
  await context.refreshAccountStatus();
  await context.importAccount();
  await Promise.resolve();
  assert.equal(get("#accountStatusTitle").textContent, "导入未完成");
  assert.ok(get("#accountStatusDetail").textContent.includes("codex login"));
  assert.equal(get("#accountEmail").textContent, "tester@example.com");
});

test("未登录与 API Key 两种空态", async () => {
  const states = {
    not_logged_in: {
      ...chatgptStatus,
      loginState: "not_logged_in",
      authMode: null,
      profile: null,
      snapshot: null,
      lastRefresh: null,
      message: "未检测到 Codex 登录。请先在终端运行 codex login，再回到此页导入",
    },
    api_key: {
      ...chatgptStatus,
      loginState: "api_key",
      authMode: "apikey",
      profile: null,
      snapshot: null,
      lastRefresh: null,
      message: "当前是 API Key 登录方式，没有可导入的 ChatGPT 账号信息",
    },
  };
  for (const [stateName, status] of Object.entries(states)) {
    const { context, get } = loadMain({ invoke: async () => status });
    await context.refreshAccountStatus();
    await Promise.resolve();
    assert.equal(get("#importAccountButton").disabled, true, stateName);
    assert.equal(get("#accountUsageBadge").textContent, "未导入", stateName);
  }
});
