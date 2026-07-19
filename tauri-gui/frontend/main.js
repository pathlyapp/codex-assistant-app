const ICONS = {
  activity: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>',
  alert: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.73 18 13.73 4a2 2 0 0 0-3.46 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>',
  arrowLeft: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>',
  appWindow: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="16" x="2" y="4" rx="2"/><path d="M6 8h.01"/><path d="M10 8h.01"/><path d="M2 12h20"/></svg>',
  arrowRight: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>',
  check: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>',
  chevronDown: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>',
  circle: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/></svg>',
  copy: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>',
  cpu: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="16" height="16" x="4" y="4" rx="2"/><rect width="6" height="6" x="9" y="9" rx="1"/><path d="M9 1v3"/><path d="M15 1v3"/><path d="M9 20v3"/><path d="M15 20v3"/><path d="M20 9h3"/><path d="M20 14h3"/><path d="M1 9h3"/><path d="M1 14h3"/></svg>',
  externalLink: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/></svg>',
  eye: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2.06 12.35a1 1 0 0 1 0-.7C3.73 7.6 7.7 5 12 5c4.3 0 8.27 2.6 9.94 6.65a1 1 0 0 1 0 .7C20.27 16.4 16.3 19 12 19c-4.3 0-8.27-2.6-9.94-6.65"/><circle cx="12" cy="12" r="3"/></svg>',
  eyeOff: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m2 2 20 20"/><path d="M6.71 6.71C4.77 8 3.3 9.74 2.42 11.63a1 1 0 0 0 0 .74C4.12 16.2 7.78 19 12 19c1.48 0 2.9-.35 4.17-.97"/><path d="M10.73 5.08Q11.35 5 12 5c4.22 0 7.88 2.8 9.58 6.63a1 1 0 0 1 0 .74 12.7 12.7 0 0 1-1.18 2"/><path d="M14.12 14.12a3 3 0 0 1-4.24-4.24"/></svg>',
  fileCheck: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5Z"/><polyline points="14 2 14 8 20 8"/><path d="m9 15 2 2 4-4"/></svg>',
  home: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m3 11 9-8 9 8"/><path d="M5 10v10h14V10"/><path d="M9 20v-6h6v6"/></svg>',
  image: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-5-5L5 21"/></svg>',
  info: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>',
  key: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="7.5" cy="15.5" r="5.5"/><path d="m21 2-9.6 9.6"/><path d="m15 7 3 3"/><path d="m18 4 3 3"/></svg>',
  link: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>',
  loader: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 12a9 9 0 1 1-6.22-8.56"/></svg>',
  palette: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="13.5" cy="6.5" r=".5" fill="currentColor"/><circle cx="17.5" cy="10.5" r=".5" fill="currentColor"/><circle cx="8.5" cy="7.5" r=".5" fill="currentColor"/><circle cx="6.5" cy="12.5" r=".5" fill="currentColor"/><path d="M12 2a10 10 0 0 0 0 20c1.1 0 2-.9 2-2 0-.5-.2-1-.6-1.4-.4-.3-.6-.8-.6-1.3 0-1.1.9-2 2-2H17a5 5 0 0 0 5-5C22 5.7 17.5 2 12 2Z"/></svg>',
  plug: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22v-5"/><path d="M9 8V2"/><path d="M15 8V2"/><path d="M18 8v5a6 6 0 0 1-12 0V8Z"/></svg>',
  refresh: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 11a8.1 8.1 0 0 0-15.5-2M4 4v5h5"/><path d="M4 13a8.1 8.1 0 0 0 15.5 2M20 20v-5h-5"/></svg>',
  rotateCcw: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/></svg>',
  route: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="19" r="3"/><path d="M9 19h5.5a3.5 3.5 0 0 0 0-7h-5a3.5 3.5 0 0 1 0-7H15"/><circle cx="18" cy="5" r="3"/></svg>',
  shield: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 13c0 5-3.5 7.5-8 9-4.5-1.5-8-4-8-9V5l8-3 8 3Z"/></svg>',
  shieldCheck: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 13c0 5-3.5 7.5-8 9-4.5-1.5-8-4-8-9V5l8-3 8 3Z"/><path d="m9 12 2 2 4-4"/></svg>',
  terminal: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" x2="20" y1="19" y2="19"/></svg>',
};

const TASKS = [
  { id: "preflight", label: "检查本机环境", waiting: "检查配置目录和系统安装能力" },
  { id: "install_chatgpt", label: "准备 ChatGPT", waiting: "检测官方应用，缺失时通过 Microsoft Store 安装" },
  { id: "validate_router", label: "验证 Router", waiting: "连接 /v1/models 并核对模型" },
  { id: "configure_codex", label: "写入 Codex 配置", waiting: "安全保存认证信息并更新 config.toml" },
  { id: "verify", label: "复核配置", waiting: "再次检查应用、配置和 Router" },
];

const VIEW_COPY = {
  overview: ["首页", "确认 ChatGPT 与 Codex 服务状态"],
  setup: ["服务配置", "连接 Router 并设置 Codex 默认模型"],
  appearance: ["主题换肤", "切换 ChatGPT 工作界面与自定义背景"],
  diagnostics: ["帮助与诊断", "检查环境并导出脱敏诊断信息"],
};

const tauri = window.__TAURI__;
const state = {
  status: null,
  running: false,
  currentView: "overview",
  tasks: {},
  messages: {},
  models: [],
  testedGateway: "",
  testedWithKeyState: "",
  formDirty: false,
  keyVisible: false,
  lineCount: 0,
  finishedHandled: false,
  selectedAppearance: "official",
  customThemeReady: false,
  appearanceRequestId: 0,
  confirmResolver: null,
};

const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => [...document.querySelectorAll(selector)];

function init() {
  hydrateIcons();
  renderTasks();
  bindUi();
  wireBackendEvents();
  if (!tauri?.core || !tauri?.event) {
    $("#offlineBanner").classList.remove("hidden");
    $("#overviewAction").disabled = true;
    $("#applyButton").disabled = true;
    $("#testRouterButton").disabled = true;
    setReadiness("attention", "请从桌面程序打开", "当前页面只显示界面预览，无法读取系统状态。", "不可用");
    return;
  }
  refreshStatus({ hydrateForm: true });
  refreshAppearanceStatus();
}

function bindUi() {
  $$(".nav-item").forEach((button) => button.addEventListener("click", () => navigate(button.dataset.view)));
  $("#refreshButton").addEventListener("click", () => refreshStatus());
  $("#runDiagnosticsButton").addEventListener("click", () => refreshStatus());
  $("#overviewAction").addEventListener("click", onOverviewAction);
  $("#editConfigButton").addEventListener("click", () => navigate("setup"));
  $("#restoreConfigButton").addEventListener("click", restoreConfiguration);
  $("#diagnosticRestoreButton").addEventListener("click", restoreConfiguration);
  $("#routerForm").addEventListener("submit", applyConfiguration);
  $("#testRouterButton").addEventListener("click", () => testRouter({ announce: true }));
  $("#noAuthInput").addEventListener("change", updateAuthFields);
  $("#toggleKeyButton").addEventListener("click", toggleKeyVisibility);
  $("#gatewayInput").addEventListener("input", markFormChanged);
  $("#keyInput").addEventListener("input", markFormChanged);
  $("#modelInput").addEventListener("change", () => {
    state.formDirty = true;
    $("#applyButton").disabled = !$("#modelInput").value;
    setFormStep("apply");
  });
  $$("[data-preset]").forEach((button) => button.addEventListener("click", () => applyPreset(button.dataset.preset)));
  $("#launchButton").addEventListener("click", restartChatGPT);
  $("#resultBackButton").addEventListener("click", showSetupForm);
  $("#resultLogButton").addEventListener("click", () => {
    $("#progressLog").open = true;
    showProgressPanel();
  });
  $("#copyLogButton").addEventListener("click", copyLog);
  $("#exportLogButton").addEventListener("click", exportLog);
  $("#diagnosticCopyButton").addEventListener("click", copyDiagnostics);
  $$(".theme-card:not(:disabled)").forEach((button) =>
    button.addEventListener("click", () => selectAppearance(button.dataset.theme)),
  );
  $("#chooseBackgroundButton").addEventListener("click", () => $("#backgroundFileInput").click());
  $("#backgroundFileInput").addEventListener("change", importBackgroundImage);
  $("#applyAppearanceButton").addEventListener("click", applyAppearance);
  $("#confirmCancelButton").addEventListener("click", () => closeConfirmation(false));
  $("#confirmAcceptButton").addEventListener("click", () => closeConfirmation(true));
  $("#confirmOverlay").addEventListener("click", (event) => {
    if (event.target === $("#confirmOverlay")) closeConfirmation(false);
  });
  window.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !$("#confirmOverlay").classList.contains("hidden")) closeConfirmation(false);
  });
}

async function wireBackendEvents() {
  if (!tauri?.event) return;
  await tauri.event.listen("installer-stage", (event) => applyStage(event.payload));
  await tauri.event.listen("installer-log", (event) => appendLog(event.payload));
  await tauri.event.listen("installer-finished", (event) => finishRun(event.payload || {}));
}

function navigate(view) {
  if (state.running && view !== "setup") return;
  state.currentView = view;
  $$(".nav-item").forEach((item) => item.classList.toggle("active", item.dataset.view === view));
  $$('[data-view-panel]').forEach((panel) => panel.classList.toggle("active", panel.dataset.viewPanel === view));
  const [title, subtitle] = VIEW_COPY[view];
  $("#pageTitle").textContent = title;
  $("#pageSubtitle").textContent = subtitle;
  $("#refreshButton").classList.toggle("hidden", view === "appearance" || state.running);
  if (view === "appearance") refreshAppearanceStatus();
}

async function refreshStatus({ hydrateForm = false } = {}) {
  if (!tauri?.core || state.running) return;
  const button = $("#refreshButton");
  button.disabled = true;
  button.classList.add("spinning");
  if (!state.status) setReadiness("loading", "正在检查本机状态", "正在读取官方应用、Codex 配置和 Router。", "请稍候");
  try {
    const status = await tauri.core.invoke("get_system_status");
    state.status = status;
    renderSystemStatus(status);
    if (!state.formDirty) hydrateRouterForm(status);
  } catch (error) {
    setReadiness("attention", "状态检查失败", friendlyError(error), "重新检查");
    $("#overviewAction").disabled = false;
    renderStatusError(error);
  } finally {
    button.disabled = false;
    button.classList.remove("spinning");
  }
}

function renderSystemStatus(status) {
  const ready = Boolean(status.ready);
  if (ready) {
    setReadiness("ready", "Codex 已准备就绪", `${status.configuredModel} 已连接，可以从 ChatGPT 进入 Codex。`, "打开 ChatGPT");
  } else {
    const missing = [];
    if (!status.appInstalled) missing.push("ChatGPT");
    if (!status.configPresent) missing.push("Codex 配置");
    if (!status.routerReachable) missing.push("Router 连接");
    setReadiness("attention", "还需要完成配置", `待处理：${missing.join("、") || "重新检查状态"}`, "开始配置");
  }
  $("#overviewAction").disabled = false;
  const overall = $("#overallStatusBadge");
  overall.textContent = ready ? "全部正常" : "需要处理";
  overall.className = `status-badge ${ready ? "success" : "warning"}`;

  setStatusCard("app", status.appInstalled, status.appInstalled ? "ChatGPT 已安装" : "未检测到 ChatGPT", status.appDetail);
  setStatusCard(
    "router",
    status.routerReachable,
    status.routerReachable ? "Router 可用" : status.configuredGateway ? "Router 不可用" : "尚未配置",
    status.routerDetail,
  );
  setStatusCard(
    "config",
    status.configPresent,
    status.configPresent ? "配置有效" : "配置未完成",
    status.configPresent ? status.configPath : "需要写入 Codex Router 配置",
  );

  $("#currentGateway").textContent = status.configuredGateway || "未配置";
  $("#currentModel").textContent = status.configuredModel || "未配置";
  $("#currentKeyState").textContent = status.keyConfigured ? "已安全保存" : status.configPresent ? "无需 Key" : "未配置";
  $("#restoreConfigButton").classList.toggle("hidden", !status.backupAvailable);
  $("#diagnosticRestoreButton").disabled = !status.backupAvailable;
  $("#diagPlatform").textContent = `${status.platform} · ${formatArchitecture(status.architecture)}`;
  $("#diagApp").textContent = `${status.appInstalled ? "正常" : "未安装"} · ${status.appDetail}`;
  $("#diagConfig").textContent = status.configPresent ? `有效 · ${status.configuredModel}` : "未配置";
  $("#diagRouter").textContent = `${status.routerReachable ? "正常" : "异常"} · ${status.routerDetail}`;
  $("#diagConfigPath").textContent = status.configPath;
}

function setReadiness(kind, title, detail, action) {
  const panel = $("#readinessPanel");
  panel.className = `readiness-panel ${kind === "ready" ? "" : kind}`.trim();
  $("#readinessTitle").textContent = title;
  $("#readinessText").textContent = detail;
  $("#overviewAction").textContent = action;
  const icon = kind === "ready" ? ICONS.check : kind === "attention" ? ICONS.alert : ICONS.loader;
  $("#readinessPanel .readiness-icon .icon").innerHTML = icon;
  $("#readinessPanel .readiness-icon .icon").classList.toggle("spin", kind === "loading");
}

function setStatusCard(prefix, ok, title, detail) {
  $(`#${prefix}StatusTitle`).textContent = title;
  $(`#${prefix}StatusDetail`).textContent = detail;
  const badge = $(`#${prefix}StatusBadge`);
  badge.textContent = ok ? "正常" : "待处理";
  badge.className = `status-badge ${ok ? "success" : "warning"}`;
}

function renderStatusError(error) {
  ["app", "router", "config"].forEach((prefix) => setStatusCard(prefix, false, "检查失败", friendlyError(error)));
  $("#overallStatusBadge").textContent = "检查失败";
  $("#overallStatusBadge").className = "status-badge error";
}

function hydrateRouterForm(status) {
  $("#gatewayInput").value = status.configuredGateway || "http://127.0.0.1:11434/v1";
  const noAuth = status.configPresent ? !status.keyConfigured : true;
  $("#noAuthInput").checked = noAuth;
  $("#keyInput").value = "";
  updateAuthFields();
  populateModels(status.configuredModel ? [status.configuredModel] : [], status.configuredModel || "");
  state.testedGateway = status.routerReachable ? status.configuredGateway || "" : "";
  state.testedWithKeyState = status.routerReachable ? authFingerprint() : "";
  if (status.routerReachable) {
    setConnectionResult("success", status.routerDetail);
    setModelReady(true, status.configuredModel ? 1 : 0);
  } else {
    setConnectionResult("neutral", status.configuredGateway ? status.routerDetail : "先测试连接，助手会读取 Router 的真实模型列表。");
    setModelReady(false, 0);
  }
  state.formDirty = false;
}

function onOverviewAction() {
  if (state.status?.ready) {
    openChatGPT();
  } else {
    navigate("setup");
  }
}

function applyPreset(preset) {
  if (preset === "ollama") {
    $("#gatewayInput").value = "http://127.0.0.1:11434/v1";
    $("#noAuthInput").checked = true;
    const windowsArm = state.status?.platform === "Windows" && ["aarch64", "arm64"].includes(state.status?.architecture);
    $("#gatewayHelp").textContent = windowsArm
      ? "当前是 Windows ARM64。127.0.0.1 只指向此 VM；宿主机 Ollama 请填写宿主机可访问地址。"
      : "仅填写 Ollama 默认地址，不会自动安装或启动 Ollama。";
  }
  updateAuthFields();
  markFormChanged();
  if (preset === "ollama") {
    const windowsArm = state.status?.platform === "Windows" && ["aarch64", "arm64"].includes(state.status?.architecture);
    setConnectionResult(
      "neutral",
      windowsArm
        ? "请确认 Windows 内已有 Ollama，或改填 macOS 宿主机地址后再测试。"
        : "请确认 Ollama 已安装、正在运行且至少下载了一个模型。",
    );
  }
}

function updateAuthFields() {
  const noAuth = $("#noAuthInput").checked;
  $("#keyInput").disabled = noAuth;
  $("#toggleKeyButton").disabled = noAuth;
  $("#keyShell").classList.toggle("disabled", noAuth);
  $("#keyInput").placeholder = noAuth
    ? "Ollama 默认无需填写"
    : state.status?.keyConfigured
      ? "已安全保存；留空保持不变"
      : "输入 Router Access Key";
  state.formDirty = true;
  invalidateConnectionTest();
}

function toggleKeyVisibility() {
  state.keyVisible = !state.keyVisible;
  $("#keyInput").type = state.keyVisible ? "text" : "password";
  $("#toggleKeyButton").title = state.keyVisible ? "隐藏 Access Key" : "显示 Access Key";
  $("#toggleKeyButton").setAttribute("aria-label", $("#toggleKeyButton").title);
  $("#toggleKeyButton .icon").innerHTML = state.keyVisible ? ICONS.eyeOff : ICONS.eye;
}

function markFormChanged() {
  state.formDirty = true;
  invalidateConnectionTest();
}

function invalidateConnectionTest() {
  if (state.testedGateway && (state.testedGateway !== $("#gatewayInput").value.trim() || state.testedWithKeyState !== authFingerprint())) {
    state.testedGateway = "";
    state.testedWithKeyState = "";
    setModelReady(false, 0);
    setConnectionResult("neutral", "配置已修改，请重新测试连接。");
  }
}

function authFingerprint() {
  if ($("#noAuthInput").checked) return "no-auth";
  if ($("#keyInput").value.trim()) return "new-key";
  return state.status?.keyConfigured ? "saved-key" : "missing-key";
}

async function testRouter({ announce = false } = {}) {
  if (!tauri?.core || state.running) return false;
  const gateway = $("#gatewayInput").value.trim();
  const noAuth = $("#noAuthInput").checked;
  const key = noAuth ? "" : $("#keyInput").value.trim();
  const useSavedKey = !noAuth && !key && Boolean(state.status?.keyConfigured) && gateway === state.status?.configuredGateway;
  if (!gateway) {
    setConnectionResult("error", "请填写 Router URL。");
    $("#gatewayInput").focus();
    return false;
  }
  if (!noAuth && !key && !useSavedKey) {
    setConnectionResult("error", "请填写 Access Key，或选择“无需 Key”。");
    $("#keyInput").focus();
    return false;
  }
  const button = $("#testRouterButton");
  button.disabled = true;
  button.querySelector(".icon").innerHTML = ICONS.loader;
  button.querySelector(".icon").classList.add("spin");
  setConnectionResult("testing", "正在连接 Router 并读取模型列表…");
  setFormStep("connect");
  try {
    const response = await tauri.core.invoke("discover_models", {
      request: { gateway, key, useSavedKey },
    });
    state.models = response.models || [];
    state.testedGateway = response.gateway;
    state.testedWithKeyState = authFingerprint();
    $("#gatewayInput").value = response.gateway;
    const selected = state.status?.configuredGateway === response.gateway ? state.status.configuredModel : "";
    populateModels(state.models, selected || state.models[0]);
    setModelReady(true, state.models.length);
    setConnectionResult("success", response.message);
    if (announce) showToast("Router 连接正常");
    return true;
  } catch (error) {
    state.models = [];
    state.testedGateway = "";
    state.testedWithKeyState = "";
    populateModels([], "");
    setModelReady(false, 0);
    setConnectionResult("error", friendlyError(error));
    return false;
  } finally {
    button.disabled = false;
    button.querySelector(".icon").innerHTML = ICONS.plug;
    button.querySelector(".icon").classList.remove("spin");
  }
}

function setConnectionResult(kind, message) {
  const result = $("#connectionResult");
  result.className = `connection-result ${kind}`;
  result.querySelector(".icon").innerHTML = kind === "success" ? ICONS.check : kind === "error" ? ICONS.alert : kind === "testing" ? ICONS.loader : ICONS.info;
  result.querySelector("span:last-child").textContent = message;
}

function populateModels(models, selected) {
  const select = $("#modelInput");
  select.replaceChildren();
  if (!models.length) {
    select.add(new Option("测试连接后选择", ""));
    select.disabled = true;
    return;
  }
  models.forEach((model) => select.add(new Option(model, model)));
  select.value = models.includes(selected) ? selected : models[0];
  select.disabled = false;
}

function setModelReady(ready, count = 0) {
  $("#modelSection").classList.toggle("locked", !ready);
  $("#modelInput").disabled = !ready;
  $("#applyButton").disabled = !ready || !$("#modelInput").value;
  const badge = $("#modelCountBadge");
  badge.textContent = ready ? `${count} 个模型` : "尚未连接";
  badge.className = `status-badge ${ready ? "success" : "neutral"}`;
  $("#modelSectionHelp").textContent = ready
    ? "连接已验证，请选择 Codex 默认模型。"
    : "连接成功后可选择 Router 返回的真实模型。";
  setFormStep(ready ? "model" : "connect");
}

function setFormStep(step) {
  $$('[data-form-step]').forEach((item) => item.classList.toggle("active", item.dataset.formStep === step));
}

async function applyConfiguration(event) {
  event.preventDefault();
  if (state.running || !tauri?.core) return;
  const currentGateway = $("#gatewayInput").value.trim();
  if (state.testedGateway !== currentGateway || state.testedWithKeyState !== authFingerprint() || !$("#modelInput").value) {
    const connected = await testRouter();
    if (!connected) return;
  }
  state.running = true;
  state.finishedHandled = false;
  state.tasks = {};
  state.messages = {};
  state.lineCount = 0;
  $("#logOutput").textContent = "";
  $("#logCount").textContent = "0 行";
  renderTasks();
  showProgressPanel();
  setUiRunning(true);
  appendLog("Codex Assistant setup started\n");
  const options = {
    gateway: $("#gatewayInput").value.trim(),
    model: $("#modelInput").value,
    key: $("#noAuthInput").checked ? "" : $("#keyInput").value.trim(),
    noAuth: $("#noAuthInput").checked,
    installChatgpt: true,
  };
  try {
    const result = await tauri.core.invoke("start_setup", { options });
    if (!state.finishedHandled) finishRun(result);
  } catch (error) {
    finishRun({ success: false, summary: friendlyError(error), stages: [] });
  }
}

function applyStage(stage) {
  if (!stage?.stage) return;
  state.tasks[stage.stage] = normalizedStatus(stage.status);
  state.messages[stage.stage] = stage.message || "";
  renderTasks();
  const percent = stage.total > 0 ? Math.round((stage.current / stage.total) * 100) : 0;
  $("#progressBar").style.width = `${percent}%`;
  $("#progressPercent").textContent = `${percent}%`;
  $("#progressTitle").textContent = stage.status === "failed" ? `${stage.label}未完成` : stage.label;
}

function renderTasks() {
  const list = $("#taskList");
  list.replaceChildren();
  TASKS.forEach((task) => {
    const status = state.tasks[task.id] || "waiting";
    const item = document.createElement("div");
    item.className = `task-item ${status}`;
    const icon = document.createElement("span");
    icon.className = "task-icon";
    icon.innerHTML = taskIcon(status);
    const copy = document.createElement("span");
    copy.className = "task-copy";
    const title = document.createElement("strong");
    title.textContent = task.label;
    const detail = document.createElement("small");
    detail.textContent = state.messages[task.id] || task.waiting;
    copy.append(title, detail);
    const label = document.createElement("span");
    label.className = "task-state";
    label.textContent = taskStatusLabel(status);
    item.append(icon, copy, label);
    list.append(item);
  });
}

function taskIcon(status) {
  if (status === "running") return `<span class="icon spin">${ICONS.loader}</span>`;
  if (status === "complete" || status === "skipped") return `<span class="icon">${ICONS.check}</span>`;
  if (status === "failed") return `<span class="icon">${ICONS.alert}</span>`;
  return `<span class="icon">${ICONS.circle}</span>`;
}

function taskStatusLabel(status) {
  return { running: "进行中", complete: "已完成", skipped: "已确认", failed: "失败", waiting: "等待" }[status] || "等待";
}

function normalizedStatus(status) {
  if (["complete", "skipped", "failed", "running"].includes(status)) return status;
  return "waiting";
}

function finishRun(payload) {
  if (state.finishedHandled) return;
  state.finishedHandled = true;
  state.running = false;
  if (Array.isArray(payload.stages)) {
    payload.stages.forEach((stage) => {
      state.tasks[stage.stage] = normalizedStatus(stage.status);
      state.messages[stage.stage] = stage.message || "";
    });
  }
  renderTasks();
  setUiRunning(false);
  showResult(payload);
  if (payload.success) refreshStatus({ hydrateForm: false });
}

function showResult(payload) {
  $("#setupFormPanel").classList.add("hidden");
  $("#progressPanel").classList.add("hidden");
  $("#resultPanel").classList.remove("hidden");
  const success = Boolean(payload.success);
  const mark = $("#resultMark");
  mark.className = `result-mark ${success ? "success" : "error"}`;
  mark.querySelector(".icon").innerHTML = success ? ICONS.check : ICONS.alert;
  $("#resultTitle").textContent = success ? "配置完成" : "配置未完成";
  $("#resultText").textContent = success
    ? "配置已写入并通过验证。重启 ChatGPT 后即可使用新的 Codex 模型服务。"
    : `${payload.summary || "请根据失败步骤修正后重试。"}，日志中没有保存 Access Key 明文。`;
  $("#launchButton").classList.toggle("hidden", !success);
  $("#resultBackButton").textContent = success ? "返回修改" : "修正并重试";
  renderResultSummary(success, payload);
}

function renderResultSummary(success, payload) {
  const summary = $("#resultSummary");
  summary.replaceChildren();
  if (success) {
    addSummaryRow(summary, "ChatGPT", "官方应用已确认");
    addSummaryRow(summary, "Router", $("#gatewayInput").value);
    addSummaryRow(summary, "模型", $("#modelInput").value);
    addSummaryRow(summary, "Access Key", $("#noAuthInput").checked ? "无需 Key" : "已安全保存");
  } else {
    const failed = (payload.stages || []).find((stage) => stage.status === "failed");
    addSummaryRow(summary, "失败步骤", failed?.label || payload.summary || "启动配置");
    addSummaryRow(summary, "原因", failed?.message || payload.summary || "未知错误");
  }
}

function addSummaryRow(container, label, value) {
  const row = document.createElement("div");
  const name = document.createElement("span");
  const content = document.createElement("strong");
  name.textContent = label;
  content.textContent = value;
  content.title = value;
  row.append(name, content);
  container.append(row);
}

function showProgressPanel() {
  navigate("setup");
  $("#setupFormPanel").classList.add("hidden");
  $("#resultPanel").classList.add("hidden");
  $("#progressPanel").classList.remove("hidden");
}

function showSetupForm() {
  $("#progressPanel").classList.add("hidden");
  $("#resultPanel").classList.add("hidden");
  $("#setupFormPanel").classList.remove("hidden");
  setFormStep(state.testedGateway ? "model" : "connect");
  navigate("setup");
}

function setUiRunning(running) {
  $$(".nav-item").forEach((button) => {
    if (button.dataset.view !== "setup") button.disabled = running;
  });
  $("#refreshButton").classList.toggle("hidden", running);
}

async function openChatGPT() {
  if (!tauri?.core) return;
  const button = $("#overviewAction");
  button.disabled = true;
  try {
    await tauri.core.invoke("launch_chatgpt");
    showToast("正在打开 ChatGPT");
  } catch (error) {
    showToast(friendlyError(error), true);
  } finally {
    button.disabled = false;
  }
}

async function restartChatGPT() {
  if (!tauri?.core) return;
  const confirmed = await requestConfirmation({
    title: "重启 ChatGPT？",
    message: "ChatGPT 将关闭后重新打开，以加载新的 Codex 配置。请先保存尚未发送的内容。",
    confirmLabel: "重启并打开",
  });
  if (!confirmed) return;
  const button = $("#launchButton");
  button.disabled = true;
  button.querySelector("span:first-child").textContent = "正在重启…";
  try {
    await tauri.core.invoke("restart_chatgpt");
    showToast("ChatGPT 已重新打开");
  } catch (error) {
    showToast(friendlyError(error), true);
  } finally {
    button.disabled = false;
    button.querySelector("span:first-child").textContent = "重启并打开 ChatGPT";
  }
}

async function restoreConfiguration() {
  if (!tauri?.core || !state.status?.backupAvailable) return;
  const confirmed = await requestConfirmation({
    title: "恢复上次配置？",
    message: "助手会先保留当前配置，再恢复最近一次备份并重启 ChatGPT。请先保存尚未发送的内容。",
    confirmLabel: "恢复并重启",
  });
  if (!confirmed) return;
  const buttons = [$("#restoreConfigButton"), $("#diagnosticRestoreButton")];
  buttons.forEach((button) => {
    button.disabled = true;
  });
  try {
    const result = await tauri.core.invoke("restore_codex_config");
    try {
      await tauri.core.invoke("restart_chatgpt");
      showToast("上次配置已恢复，ChatGPT 已重新打开");
    } catch (restartError) {
      showToast(`${result.message}；${friendlyError(restartError)}`, true);
    }
    state.formDirty = false;
    await refreshStatus({ hydrateForm: true });
  } catch (error) {
    showToast(friendlyError(error), true);
  } finally {
    buttons.forEach((button) => {
      button.disabled = false;
    });
  }
}

async function refreshAppearanceStatus() {
  if (!tauri?.core) {
    $("#applyAppearanceButton").disabled = true;
    return;
  }
  const requestId = ++state.appearanceRequestId;
  const badge = $("#appearanceStatus");
  badge.textContent = "检查中";
  badge.className = "status-badge neutral";
  try {
    const appearance = await tauri.core.invoke("get_appearance_status");
    if (requestId !== state.appearanceRequestId) return;
    state.selectedAppearance = appearance.selectedTheme || "official";
    state.customThemeReady = Boolean(appearance.customThemeReady);
    $("#customThemeLabel").textContent = appearance.customThemeName || "选择一张本地图片";
    renderAppearanceSelection();
    badge.textContent = appearance.active ? "已生效" : state.selectedAppearance === "official" ? "官方外观" : "等待启动";
    badge.className = `status-badge ${appearance.active || state.selectedAppearance === "official" ? "success" : "warning"}`;
    $("#appearanceMessage").textContent = appearance.message;
    $("#applyAppearanceButton").disabled = !appearance.supported || (state.selectedAppearance === "custom" && !state.customThemeReady);
  } catch (error) {
    if (requestId !== state.appearanceRequestId) return;
    badge.textContent = "不可用";
    badge.className = "status-badge error";
    $("#appearanceMessage").textContent = friendlyError(error);
    $("#applyAppearanceButton").disabled = true;
  }
}

function selectAppearance(theme) {
  if (theme === "custom" && !state.customThemeReady) {
    $("#backgroundFileInput").click();
    return;
  }
  state.selectedAppearance = theme;
  renderAppearanceSelection();
}

function renderAppearanceSelection() {
  $$(".theme-card").forEach((card) => card.classList.toggle("selected", card.dataset.theme === state.selectedAppearance));
  $("#applyAppearanceButton").disabled = state.selectedAppearance === "custom" && !state.customThemeReady;
}

async function importBackgroundImage(event) {
  const input = event.currentTarget;
  const file = input.files?.[0];
  if (!file || !tauri?.core) return;
  const button = $("#chooseBackgroundButton");
  button.disabled = true;
  const original = button.innerHTML;
  button.textContent = "正在导入…";
  state.appearanceRequestId += 1;
  try {
    if (file.size > 8 * 1024 * 1024) throw new Error("图片不能超过 8 MB");
    const dataUrl = await readFileAsDataUrl(file);
    const appearance = await tauri.core.invoke("import_theme_image", {
      request: { fileName: file.name, mimeType: file.type, dataUrl },
    });
    state.customThemeReady = Boolean(appearance.customThemeReady);
    state.selectedAppearance = "custom";
    $("#customThemeLabel").textContent = appearance.customThemeName || file.name;
    const preview = $("#customThemePreview");
    preview.style.backgroundImage = `url(${JSON.stringify(dataUrl)})`;
    preview.classList.add("has-image");
    renderAppearanceSelection();
    $("#appearanceMessage").textContent = "背景已安全保存。点击“应用并重启 ChatGPT”即可生效。";
    showToast("背景图片已导入");
  } catch (error) {
    $("#appearanceMessage").textContent = friendlyError(error);
    showToast(friendlyError(error), true);
  } finally {
    input.value = "";
    button.disabled = false;
    button.innerHTML = original;
    hydrateIcons();
  }
}

function readFileAsDataUrl(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.addEventListener("load", () => resolve(reader.result));
    reader.addEventListener("error", () => reject(new Error("读取背景图片失败")));
    reader.readAsDataURL(file);
  });
}

async function applyAppearance() {
  if (!tauri?.core || !state.selectedAppearance) return;
  const confirmed = await requestConfirmation({
    title: state.selectedAppearance === "official" ? "恢复官方外观？" : state.selectedAppearance === "custom" ? "应用自定义背景？" : "应用专注外观？",
    message:
      state.selectedAppearance === "official"
        ? "恢复官方外观需要重新启动 ChatGPT。请先保存尚未发送的内容。"
        : "应用外观需要重新启动 ChatGPT。请先保存尚未发送的内容。",
    confirmLabel: "应用并重启",
  });
  if (!confirmed) return;
  state.appearanceRequestId += 1;
  const button = $("#applyAppearanceButton");
  button.disabled = true;
  button.textContent = "正在重启并应用…";
  const badge = $("#appearanceStatus");
  badge.textContent = "应用中";
  badge.className = "status-badge warning";
  try {
    const appearance = await tauri.core.invoke("apply_appearance", { theme: state.selectedAppearance });
    badge.textContent = appearance.active ? "已生效" : "官方外观";
    badge.className = "status-badge success";
    $("#appearanceMessage").textContent = appearance.message;
    showToast("ChatGPT 外观已更新");
  } catch (error) {
    badge.textContent = "应用失败";
    badge.className = "status-badge error";
    $("#appearanceMessage").textContent = friendlyError(error);
    showToast(friendlyError(error), true);
  } finally {
    button.disabled = false;
    button.textContent = "应用并重启 ChatGPT";
  }
}

function appendLog(line) {
  const clean = String(line)
    .replace(/(Bearer\s+)[^\s]+/gi, "$1[redacted]")
    .replace(/([?&](?:token|key)=)[^&\s]+/gi, "$1[redacted]");
  $("#logOutput").textContent += clean.endsWith("\n") ? clean : `${clean}\n`;
  state.lineCount += 1;
  $("#logCount").textContent = `${state.lineCount} 行`;
  $("#logOutput").scrollTop = $("#logOutput").scrollHeight;
}

async function copyLog() {
  const text = $("#logOutput").textContent;
  if (!text.trim()) return showToast("暂无日志");
  try {
    await navigator.clipboard.writeText(text);
    showToast("日志已复制");
  } catch {
    showToast("复制失败", true);
  }
}

function exportLog() {
  const text = $("#logOutput").textContent;
  if (!text.trim()) return showToast("暂无日志");
  const url = URL.createObjectURL(new Blob([text], { type: "text/plain;charset=utf-8" }));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `codex-assistant-${new Date().toISOString().replace(/[:.]/g, "-")}.log`;
  anchor.click();
  URL.revokeObjectURL(url);
  showToast("日志已导出");
}

async function copyDiagnostics() {
  if (!state.status) return showToast("请先运行诊断", true);
  const status = state.status;
  const text = [
    `Codex Assistant: 0.8.4`,
    `Platform: ${status.platform}`,
    `ChatGPT: ${status.appInstalled ? "installed" : "missing"} (${status.appDetail})`,
    `Codex config: ${status.configPresent ? "valid" : "missing"}`,
    `Router: ${status.routerReachable ? "reachable" : "unreachable"} (${status.routerDetail})`,
    `Gateway: ${status.configuredGateway || "not configured"}`,
    `Model: ${status.configuredModel || "not configured"}`,
    `Config path: ${status.configPath}`,
    `Config backup: ${status.backupAvailable ? "available" : "missing"}`,
  ].join("\n");
  try {
    await navigator.clipboard.writeText(text);
    showToast("诊断信息已复制");
  } catch {
    showToast("复制失败", true);
  }
}

function showToast(message, error = false) {
  const toast = $("#actionToast");
  toast.textContent = message;
  toast.className = `action-toast${error ? " error" : ""}`;
  window.clearTimeout(showToast.timer);
  showToast.timer = window.setTimeout(() => {
    toast.textContent = "";
  }, 3000);
}

function requestConfirmation({ title, message, confirmLabel = "确认" }) {
  if (state.confirmResolver) closeConfirmation(false);
  $("#confirmTitle").textContent = title;
  $("#confirmMessage").textContent = message;
  $("#confirmAcceptButton").textContent = confirmLabel;
  $("#confirmOverlay").classList.remove("hidden");
  window.setTimeout(() => $("#confirmAcceptButton").focus(), 0);
  return new Promise((resolve) => {
    state.confirmResolver = resolve;
  });
}

function closeConfirmation(accepted) {
  const resolver = state.confirmResolver;
  state.confirmResolver = null;
  $("#confirmOverlay").classList.add("hidden");
  if (resolver) resolver(accepted);
}

function friendlyError(error) {
  const text = String(error?.message || error || "未知错误");
  return text
    .replace(/^Error:\s*/i, "")
    .replace(/Transport\([^)]*\)/g, "连接失败")
    .replace(/Bearer\s+[^\s]+/gi, "Bearer [redacted]");
}

function formatArchitecture(architecture) {
  return { aarch64: "ARM64", arm64: "ARM64", x86_64: "x64", x64: "x64" }[architecture] || architecture || "未知架构";
}

function hydrateIcons() {
  $$('[data-icon]').forEach((element) => {
    element.innerHTML = ICONS[element.dataset.icon] || "";
  });
}

init();
