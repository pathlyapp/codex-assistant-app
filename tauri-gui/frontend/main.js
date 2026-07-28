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
  download: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/></svg>',
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
  unplug: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m19 5 3-3"/><path d="m2 22 3-3"/><path d="M6.3 20.3a2.4 2.4 0 0 0 3.4 0L12 18l-6-6-2.3 2.3a2.4 2.4 0 0 0 0 3.4Z"/><path d="M7.5 13.5 10 11"/><path d="M10.5 16.5 13 14"/><path d="m12 6 6 6 2.3-2.3a2.4 2.4 0 0 0 0-3.4l-2.6-2.6a2.4 2.4 0 0 0-3.4 0Z"/></svg>',
  trash: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>',
};

const TASKS = [
  { id: "preflight", label: "检查本机环境", waiting: "检查配置目录和系统安装能力" },
  { id: "install_chatgpt", label: "准备 ChatGPT", waiting: "检测官方应用，缺失时通过 Microsoft Store 安装" },
  { id: "validate_router", label: "读取 Router 模型", waiting: "连接 /v1/models 并核对模型" },
  { id: "validate_router_response", label: "验证实际请求", waiting: "发送固定低成本请求并确认 /v1/responses 可用" },
  { id: "configure_codex", label: "写入 Codex 配置", waiting: "安全保存认证信息并更新 config.toml" },
  { id: "verify", label: "复核配置", waiting: "再次检查应用、配置和 Router" },
];

const GUIDED_STEPS = ["environment", "app", "service", "verify"];
const LIFECYCLE_ACTIONS = {
  uninstall_assistant: {
    title: "卸载 Codex 助手？",
    message: "只启动助手自己的系统卸载程序。ChatGPT、Codex 配置和助手数据都会保留。",
    confirmLabel: "卸载助手",
    confirmation: "UNINSTALL_ASSISTANT",
    danger: true,
  },
  restore_pre_assistant_config: {
    title: "恢复助手修改前的 Codex 配置？",
    message: "将移除助手管理的 Router 配置，同时保留 config.toml 中其他用户配置。操作使用可回滚事务。",
    confirmLabel: "恢复原配置",
    confirmation: "RESTORE_MANAGED_CONFIGURATION",
    danger: false,
  },
  delete_assistant_data: {
    title: "永久删除助手数据？",
    message: "将删除本地状态、事务备份、主题和保存的 Key。ChatGPT 和非助手管理的 Codex 配置不受影响。",
    confirmLabel: "删除助手数据",
    confirmation: "DELETE_ASSISTANT_DATA",
    danger: true,
  },
  open_official_app_management: {
    confirmation: "",
  },
};
const STAGE_GUIDED_STEP = {
  preflight: "environment",
  install_chatgpt: "app",
  validate_router: "service",
  validate_router_response: "service",
  configure_codex: "service",
  verify: "verify",
  rollback: "verify",
};

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
  installingApp: false,
  lifecycleRunning: "",
  lifecycleStatus: null,
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
  currentOperationId: "",
  finishedOperationId: "",
  selectedAppearance: "official",
  customThemeReady: false,
  appearanceRequestId: 0,
  presets: [],
  presetsLoading: false,
  gallery: [],
  galleryState: "idle",
  galleryError: "",
  confirmResolver: null,
  confirmReturnFocus: null,
  lastResultPayload: null,
  lastErrorEnvelope: null,
  repairPlan: null,
  repairRequestId: 0,
  repairing: false,
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
  $("#appInstallButton").addEventListener("click", installChatGPT);
  $("#setupInstallButton").addEventListener("click", () => installChatGPT({ continueToSetup: true }));
  $("#editConfigButton").addEventListener("click", () => navigate("setup"));
  $("#restoreConfigButton").addEventListener("click", restoreConfiguration);
  $("#disconnectRouterButton").addEventListener("click", disconnectRouter);
  $("#repairActionButton").addEventListener("click", runRecommendedRepair);
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
  $("#resultDiagnosticButton").addEventListener("click", copyDiagnostics);
  $("#resultLogButton").addEventListener("click", () => {
    $("#progressLog").open = true;
    showProgressPanel();
  });
  $("#copyLogButton").addEventListener("click", copyLog);
  $("#exportLogButton").addEventListener("click", exportLog);
  $("#diagnosticCopyButton").addEventListener("click", copyDiagnostics);
  $("#diagnosticExportButton").addEventListener("click", exportDiagnosticBundle);
  $("#uninstallAssistantButton").addEventListener("click", () => runLifecycleAction("uninstall_assistant"));
  $("#restoreManagedConfigButton").addEventListener("click", () =>
    runLifecycleAction("restore_pre_assistant_config"),
  );
  $("#deleteAssistantDataButton").addEventListener("click", () =>
    runLifecycleAction("delete_assistant_data"),
  );
  $("#openOfficialAppManagementButton").addEventListener("click", () =>
    runLifecycleAction("open_official_app_management"),
  );
  $$(".theme-card:not(:disabled)").forEach((button) =>
    button.addEventListener("click", () => selectAppearance(button.dataset.theme)),
  );
  $("#chooseBackgroundButton").addEventListener("click", () => $("#backgroundFileInput").click());
  $("#backgroundFileInput").addEventListener("change", importBackgroundImage);
  $("#applyAppearanceButton").addEventListener("click", applyAppearance);
  $("#galleryRefreshButton").addEventListener("click", () => loadGalleryThemes({ force: true }));
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
  if (view === "appearance") {
    refreshAppearanceStatus();
    loadPresetThemes();
    loadGalleryThemes();
  }
  if (view === "setup") renderSetupPrerequisites(state.status);
  if (view === "diagnostics" && state.status) {
    refreshRepairPlan();
    refreshLifecycleStatus();
  }
}

async function refreshStatus({ hydrateForm = false } = {}) {
  if (!tauri?.core || state.running || state.installingApp || state.lifecycleRunning) return;
  const button = $("#refreshButton");
  button.disabled = true;
  button.classList.add("spinning");
  if (!state.status) setReadiness("loading", "正在检查本机状态", "正在读取官方应用、Codex 配置和 Router。", "请稍候");
  try {
    const status = normalizeSystemStatus(await tauri.core.invoke("get_system_status"));
    state.status = status;
    renderSystemStatus(status);
    if (!state.formDirty) hydrateRouterForm(status);
    await refreshRepairPlan();
  } catch (error) {
    setReadiness("attention", "状态检查失败", friendlyError(error), "重新检查");
    $("#overviewAction").disabled = false;
    renderStatusError(error);
    renderRepairPlanError(error);
  } finally {
    button.disabled = false;
    button.classList.remove("spinning");
  }
}

function renderSystemStatus(status) {
  const ready = status.overall === "ready";
  const appNeedsRepair = status.appState === "needs_repair";
  const recommendedAction = status.recommendedAction || {
    id: ready ? "open_chatgpt" : "configure_router",
    label: ready ? "打开 ChatGPT" : "开始配置",
  };
  if (ready) {
    setReadiness(
      "ready",
      "Codex 已准备就绪",
      `${status.configuredModel} 已连接，可以从 ChatGPT 进入 Codex。`,
      recommendedAction.label,
    );
  } else {
    const missing = [];
    if (appNeedsRepair) missing.push("ChatGPT 安装需要修复");
    else if (!status.appInstalled) missing.push("ChatGPT");
    if (!status.configPresent) missing.push("Codex 配置");
    if (!status.routerReachable) missing.push("Router 连接");
    setReadiness(
      "attention",
      appNeedsRepair ? "ChatGPT 安装需要修复" : status.overall === "blocked" ? "Router 当前不可用" : "还需要完成配置",
      `待处理：${missing.join("、") || "重新检查状态"}`,
      recommendedAction.label,
    );
  }
  $("#overviewAction").disabled = false;
  const overall = $("#overallStatusBadge");
  overall.textContent = ready ? "全部正常" : "需要处理";
  overall.className = `status-badge ${ready ? "success" : "warning"}`;

  setStatusCard(
    "app",
    status.appInstalled,
    status.appInstalled ? "ChatGPT 已安装" : appNeedsRepair ? "ChatGPT 安装异常" : "未检测到 ChatGPT",
    status.appDetail,
    appNeedsRepair ? "error" : "warning",
  );
  $("#appInstallButton").classList.toggle(
    "hidden",
    status.appInstalled || status.platform !== "Windows" || appNeedsRepair,
  );
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
  $("#disconnectRouterButton").classList.toggle("hidden", !status.configPresent);
  $("#diagPlatform").textContent = `${status.platform} · ${formatArchitecture(status.architecture)}`;
  $("#diagApp").textContent = `${status.appInstalled ? "正常" : appNeedsRepair ? "需要修复" : "未安装"} · ${status.appDetail}`;
  $("#diagConfig").textContent = status.configPresent ? `有效 · ${status.configuredModel}` : "未配置";
  $("#diagRouter").textContent = `${status.routerReachable ? "正常" : "异常"} · ${status.routerDetail}`;
  $("#diagConfigPath").textContent = status.configPath;
  renderSetupPrerequisites(status);
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

function setStatusCard(prefix, ok, title, detail, failureKind = "warning") {
  $(`#${prefix}StatusTitle`).textContent = title;
  $(`#${prefix}StatusDetail`).textContent = detail;
  const badge = $(`#${prefix}StatusBadge`);
  badge.textContent = ok ? "正常" : "待处理";
  badge.className = `status-badge ${ok ? "success" : failureKind}`;
}

function renderStatusError(error) {
  ["app", "router", "config"].forEach((prefix) => setStatusCard(prefix, false, "检查失败", friendlyError(error)));
  $("#appInstallButton").classList.add("hidden");
  $("#overallStatusBadge").textContent = "检查失败";
  $("#overallStatusBadge").className = "status-badge error";
  $("#setupEnvironmentDetail").textContent = friendlyError(error);
  setBadge($("#setupEnvironmentState"), "检查失败", "error");
  $("#setupAppDetail").textContent = "等待环境检测完成";
  setBadge($("#setupAppState"), "等待", "neutral");
  $("#setupInstallButton").classList.add("hidden");
  setGuidedStep("environment", { failed: true });
}

function renderSetupPrerequisites(status) {
  if (!status) {
    $("#setupEnvironmentDetail").textContent = "正在读取系统与架构信息";
    setBadge($("#setupEnvironmentState"), "检查中", "pending");
    $("#setupAppDetail").textContent = "正在检查安装状态";
    setBadge($("#setupAppState"), "检查中", "pending");
    $("#setupInstallButton").classList.add("hidden");
    setGuidedStep("environment");
    return;
  }

  $("#setupEnvironmentDetail").textContent = `${status.platform} · ${formatArchitecture(status.architecture)}`;
  setBadge($("#setupEnvironmentState"), "可用", "success");
  const appNeedsRepair = status.appState === "needs_repair";
  $("#setupAppDetail").textContent = status.appInstalled
    ? status.appDetail
    : appNeedsRepair
      ? status.appDetail
      : "未检测到 ChatGPT 官方应用";
  setBadge(
    $("#setupAppState"),
    status.appInstalled ? "已安装" : appNeedsRepair ? "需修复" : "待安装",
    status.appInstalled ? "success" : appNeedsRepair ? "error" : "warning",
  );
  $("#setupInstallButton").classList.toggle(
    "hidden",
    status.appInstalled || status.platform !== "Windows" || appNeedsRepair,
  );

  if (!status.appInstalled) {
    setGuidedStep("app");
  } else if (!state.running && $("#resultPanel").classList.contains("hidden")) {
    setGuidedStep("service");
  }
}

function setBadge(element, label, kind) {
  element.textContent = label;
  element.className = `status-badge ${kind}`;
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
  const action = state.status?.recommendedAction?.id;
  switch (action) {
    case "open_chatgpt":
      openChatGPT();
      break;
    case "install_chatgpt":
      installChatGPT({ continueToSetup: true });
      break;
    case "open_install_guide":
    case "open_diagnostics":
      navigate("diagnostics");
      break;
    default:
      navigate("setup");
  }
}

async function refreshRepairPlan() {
  if (!tauri?.core || !state.status) return;
  const requestId = ++state.repairRequestId;
  const errorCode = state.lastErrorEnvelope?.code || state.lastResultPayload?.error?.code || "";
  renderRepairPlanLoading();
  try {
    const plan = await tauri.core.invoke("get_repair_plan", {
      request: { errorCode },
    });
    if (requestId !== state.repairRequestId) return;
    state.repairPlan = plan;
    renderRepairPlan(plan);
  } catch (error) {
    if (requestId !== state.repairRequestId) return;
    state.repairPlan = null;
    renderRepairPlanError(error);
  }
}

function renderRepairPlanLoading() {
  const panel = $("#repairPanel");
  panel.className = "repair-panel";
  setBadge($("#repairState"), "检查中", "pending");
  $("#repairTitle").textContent = "正在生成修复方案";
  $("#repairDetail").textContent = "助手会根据当前系统状态选择一个安全的处理动作。";
  $("#repairErrorCode").classList.add("hidden");
  $("#repairActionButton").classList.add("hidden");
}

function renderRepairPlan(plan) {
  const panel = $("#repairPanel");
  panel.className = `repair-panel ${String(plan.state || "").replaceAll("_", "-")}`;
  const labels = {
    action_available: ["可自动修复", "warning"],
    manual_required: ["需人工处理", "warning"],
    not_needed: ["无需修复", "success"],
  };
  const [label, kind] = labels[plan.state] || ["已检查", "neutral"];
  setBadge($("#repairState"), label, kind);
  $("#repairTitle").textContent = plan.title || "修复方案";
  $("#repairDetail").textContent = plan.detail || "暂无可执行动作。";
  const code = $("#repairErrorCode");
  code.textContent = plan.errorCode || "";
  code.classList.toggle("hidden", !plan.errorCode);
  const button = $("#repairActionButton");
  button.classList.toggle("hidden", !plan.action);
  button.disabled = state.repairing || !plan.action;
  if (plan.action) {
    button.querySelector("span:last-child").textContent = plan.action.label;
    button.dataset.actionId = plan.action.id;
  } else {
    button.dataset.actionId = "";
  }
}

function renderRepairPlanError(error) {
  const panel = $("#repairPanel");
  panel.className = "repair-panel manual-required";
  setBadge($("#repairState"), "检查失败", "error");
  $("#repairTitle").textContent = "暂时无法生成修复方案";
  $("#repairDetail").textContent = friendlyError(error);
  $("#repairErrorCode").classList.add("hidden");
  $("#repairActionButton").classList.add("hidden");
}

async function runRecommendedRepair() {
  const action = state.repairPlan?.action;
  if (!tauri?.core || !action || state.repairing) return;
  if (action.requiresConfirmation) {
    const confirmed = await requestConfirmation({
      title: `${action.label}？`,
      message: action.description,
      confirmLabel: action.label,
    });
    if (!confirmed) return;
  }

  const button = $("#repairActionButton");
  const resultNode = $("#repairResult");
  state.repairing = true;
  button.disabled = true;
  button.querySelector(".icon").innerHTML = ICONS.loader;
  button.querySelector(".icon").classList.add("spin");
  resultNode.className = "repair-result hidden";
  resultNode.dataset.actionId = "";
  resultNode.dataset.changed = "";
  resultNode.dataset.beforeRouterState = "";
  resultNode.dataset.afterRouterState = "";
  try {
    const result = await tauri.core.invoke("run_repair", {
      request: {
        actionId: action.id,
        errorCode: state.repairPlan.errorCode || "",
      },
    });
    state.formDirty = false;
    await refreshStatus({ hydrateForm: true });
    resultNode.textContent = result.changed
      ? `修复完成，状态已更新：${result.summary}`
      : `检查完成，系统状态未变化：${result.summary}`;
    resultNode.dataset.actionId = result.actionId || "";
    resultNode.dataset.changed = String(Boolean(result.changed));
    resultNode.dataset.beforeRouterState = result.before?.routerState || "";
    resultNode.dataset.afterRouterState = result.after?.routerState || "";
    resultNode.className = "repair-result";
    showToast(result.changed ? "修复已完成" : "检查已完成");
  } catch (error) {
    resultNode.textContent = friendlyError(error);
    resultNode.className = "repair-result error";
    showToast(friendlyError(error), true);
    await refreshRepairPlan();
  } finally {
    state.repairing = false;
    button.querySelector(".icon").classList.remove("spin");
    button.querySelector(".icon").innerHTML = ICONS.refresh;
    if (state.repairPlan) renderRepairPlan(state.repairPlan);
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
  setGuidedStep("service");
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
    const code = errorCode(error);
    if (code === "ROUTER_AUTH_FAILED") {
      $("#keyInput").focus();
    } else if (["ROUTER_DNS_FAILED", "ROUTER_CONNECTION_REFUSED", "ROUTER_TIMEOUT", "ROUTER_TLS_FAILED"].includes(code)) {
      $("#gatewayInput").focus();
    }
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
  setGuidedStep("service");
}

function setFormStep(step) {
  setGuidedStep(["connect", "model", "apply"].includes(step) ? "service" : step);
}

function setGuidedStep(step, { failed = false, completed = false } = {}) {
  const currentIndex = Math.max(0, GUIDED_STEPS.indexOf(step));
  $$("[data-guided-step]").forEach((item, index) => {
    const isCurrent = index === currentIndex;
    const isComplete = completed ? index <= currentIndex : index < currentIndex && guidedStepCompleted(item.dataset.guidedStep);
    item.classList.toggle("active", isCurrent && !completed);
    item.classList.toggle("complete", isComplete);
    item.classList.toggle("failed", isCurrent && failed);
    if (isCurrent && !completed) {
      item.setAttribute("aria-current", "step");
    } else {
      item.removeAttribute("aria-current");
    }
  });
}

function guidedStepCompleted(step) {
  const stageDone = (stage) => ["complete", "skipped", "restored"].includes(state.tasks[stage]);
  if (step === "environment") return Boolean(state.status) || stageDone("preflight");
  if (step === "app") return Boolean(state.status?.appInstalled) || stageDone("install_chatgpt");
  if (step === "service") {
    return stageDone("validate_router")
      && stageDone("validate_router_response")
      && stageDone("configure_codex");
  }
  return false;
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
  state.currentOperationId = "";
  state.finishedOperationId = "";
  state.tasks = {};
  state.messages = {};
  state.lineCount = 0;
  $("#logOutput").textContent = "";
  $("#logCount").textContent = "0 行";
  renderTasks();
  setGuidedStep("environment");
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
    if (!state.finishedHandled) await finishRun(result);
  } catch (error) {
    await finishRun({ success: false, summary: friendlyError(error), stages: [] });
  }
}

function applyStage(stage) {
  if (!stage?.stage) return;
  if (stage.operationId) {
    if (state.currentOperationId && state.currentOperationId !== stage.operationId) return;
    state.currentOperationId = stage.operationId;
  }
  state.tasks[stage.stage] = normalizedStatus(stage.status);
  state.messages[stage.stage] = stage.message || "";
  renderTasks();
  const percent = stage.total > 0 ? Math.round((stage.current / stage.total) * 100) : 0;
  $("#progressBar").style.width = `${percent}%`;
  $("#progressPercent").textContent = `${percent}%`;
  $("#progressTitle").textContent = stage.status === "failed" ? `${stage.label}未完成` : stage.label;
  setGuidedStep(STAGE_GUIDED_STEP[stage.stage] || "service", { failed: stage.status === "failed" });
}

function renderTasks() {
  const list = $("#taskList");
  list.replaceChildren();
  TASKS.forEach((task) => {
    const status = state.tasks[task.id] || "waiting";
    const item = document.createElement("div");
    item.className = `task-item ${status}`;
    item.dataset.taskId = task.id;
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
  if (status === "complete" || status === "skipped" || status === "restored") return `<span class="icon">${ICONS.check}</span>`;
  if (status === "failed") return `<span class="icon">${ICONS.alert}</span>`;
  return `<span class="icon">${ICONS.circle}</span>`;
}

function taskStatusLabel(status) {
  return { running: "进行中", complete: "已完成", skipped: "已确认", failed: "失败", restored: "已恢复", waiting: "等待" }[status] || "等待";
}

function normalizedStatus(status) {
  if (["complete", "skipped", "failed", "restored", "running"].includes(status)) return status;
  return "waiting";
}

async function finishRun(payload) {
  if (payload.operationId) {
    if (state.currentOperationId && state.currentOperationId !== payload.operationId) return;
    if (state.finishedOperationId === payload.operationId) return;
    state.currentOperationId = payload.operationId;
    state.finishedOperationId = payload.operationId;
  }
  if (state.finishedHandled) return;
  state.finishedHandled = true;
  state.running = false;
  state.lastResultPayload = payload;
  state.lastErrorEnvelope = payload.error || null;
  if (Array.isArray(payload.stages)) {
    payload.stages.forEach((stage) => {
      state.tasks[stage.stage] = normalizedStatus(stage.status);
      state.messages[stage.stage] = stage.message || "";
    });
  }
  renderTasks();
  setUiRunning(false);
  await refreshStatus({ hydrateForm: false });
  showResult(payload);
}

function showResult(payload) {
  state.lastResultPayload = payload;
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
  $("#resultBackButton").className = success ? "secondary-button" : "primary-button";
  $("#progressLog").open = !success;
  const failedStage = (payload.stages || []).find((stage) => stage.stage === "rollback" && stage.status === "failed")
    || (payload.stages || []).find((stage) => stage.status === "failed");
  setGuidedStep(success ? "verify" : STAGE_GUIDED_STEP[failedStage?.stage] || "environment", {
    failed: !success,
    completed: success,
  });
  renderRecoveryStatus(payload);
  renderResultSummary(success, payload);
  window.setTimeout(() => $("#resultPanel").focus(), 0);
}

function renderResultSummary(success, payload) {
  const summary = $("#resultSummary");
  summary.replaceChildren();
  if (success) {
    addSummaryRow(
      summary,
      "ChatGPT",
      state.status?.appInstalled ? "官方应用已确认" : "安装状态待复核",
      "app",
    );
    addSummaryRow(summary, "Router", safeGateway($("#gatewayInput").value), "router");
    addSummaryRow(summary, "模型", $("#modelInput").value, "model");
    addSummaryRow(
      summary,
      "最近验证",
      formatTimestamp(state.status?.router?.lastVerifiedAt) || "本次验证通过",
      "last-verified",
    );
    addSummaryRow(
      summary,
      "恢复能力",
      state.status?.backupAvailable ? "可恢复到上次配置" : "已记录本次配置事务",
      "recovery",
    );
  } else {
    const stages = payload.stages || [];
    const failed = stages.find((stage) => stage.stage === "rollback" && stage.status === "failed")
      || stages.find((stage) => stage.status === "failed");
    addSummaryRow(summary, "失败步骤", failed?.label || payload.summary || "启动配置", "failed-stage");
    addSummaryRow(summary, "原因", failed?.message || payload.summary || "未知错误", "failure-reason");
    if (payload.failure?.code) addSummaryRow(summary, "错误代码", payload.failure.code, "error-code");
    addSummaryRow(summary, "建议操作", "修正连接信息后重新验证", "recommended-action");
  }
}

function renderRecoveryStatus(payload) {
  const recovery = $("#resultRecovery");
  const recoveryText = $("#resultRecoveryText");
  const rollback = (payload.stages || []).find((stage) => stage.stage === "rollback");
  recovery.className = "result-recovery hidden";
  recovery.dataset.recoveryState = "none";
  recoveryText.textContent = "";
  if (rollback?.status === "restored") {
    recovery.className = "result-recovery restored";
    recovery.dataset.recoveryState = "restored";
    recoveryText.textContent = "本次修改未生效，助手已恢复到操作前状态。";
  } else if (rollback?.status === "failed") {
    recovery.className = "result-recovery failed";
    recovery.dataset.recoveryState = "failed";
    recoveryText.textContent = "自动恢复失败。请先复制诊断信息，不要继续修改配置。";
  }
}

function addSummaryRow(container, label, value, key = "") {
  const row = document.createElement("div");
  if (key) row.dataset.summaryKey = key;
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
  renderSetupPrerequisites(state.status);
  navigate("setup");
}

function setUiRunning(running) {
  $$(".nav-item").forEach((button) => {
    if (button.dataset.view !== "setup") button.disabled = running;
  });
  $("#refreshButton").classList.toggle("hidden", running);
}

async function installChatGPT({ continueToSetup = false } = {}) {
  if (!tauri?.core || state.running || state.installingApp) return;
  let installationFailed = false;
  const confirmed = await requestConfirmation({
    title: "下载并安装 ChatGPT？",
    message: continueToSetup
      ? "助手将先通过 Microsoft Store 官方渠道下载并安装 ChatGPT（可能需要几分钟，期间可能出现系统确认窗口），完成后继续服务配置。"
      : "助手将调用 Microsoft Store 官方安装渠道下载并安装 ChatGPT，过程可能需要几分钟，期间可能出现系统确认窗口。",
    confirmLabel: continueToSetup ? "安装并配置" : "开始安装",
  });
  if (!confirmed) return;
  state.installingApp = true;
  $("#overviewAction").disabled = true;
  if (continueToSetup) {
    setReadiness("loading", "正在下载并安装 ChatGPT", "正在调用 Microsoft Store 官方渠道，请留意系统确认窗口。", "请稍候");
  }
  renderAppInstallState();
  try {
    const status = normalizeSystemStatus(await tauri.core.invoke("install_chatgpt_app"));
    state.status = status;
    renderSystemStatus(status);
    showToast("ChatGPT 已通过官方渠道安装");
    if (continueToSetup) navigate("setup");
  } catch (error) {
    installationFailed = true;
    setStatusCard("app", false, "ChatGPT 安装未完成", friendlyError(error));
    $("#setupAppDetail").textContent = friendlyError(error);
    setBadge($("#setupAppState"), "未完成", "error");
    setGuidedStep("app", { failed: true });
    if (continueToSetup) {
      setReadiness("attention", "还需要完成配置", "ChatGPT 安装未完成，可点击重试安装。", "安装并配置");
    }
    showToast(friendlyError(error), true);
  } finally {
    state.installingApp = false;
    renderAppInstallState();
    if (!installationFailed) renderSetupPrerequisites(state.status);
    $("#overviewAction").disabled = false;
  }
}

function renderAppInstallState() {
  const installing = state.installingApp;
  const buttons = [$("#appInstallButton"), $("#setupInstallButton")];
  buttons.forEach((button) => {
    button.disabled = installing;
    button.querySelector(".icon").innerHTML = installing ? ICONS.loader : ICONS.download;
    button.querySelector(".icon").classList.toggle("spin", installing);
  });
  $("#appInstallButton").querySelector("span:last-child").textContent = installing ? "正在安装…" : "下载安装";
  $("#setupInstallButton").querySelector("span:last-child").textContent = installing ? "安装中…" : "安装";
  if (installing) {
    $("#appStatusTitle").textContent = "正在下载并安装 ChatGPT";
    $("#appStatusDetail").textContent = "正在调用 Microsoft Store 官方渠道，请留意系统确认窗口。";
    const badge = $("#appStatusBadge");
    badge.textContent = "安装中";
    badge.className = "status-badge pending";
    $("#setupAppDetail").textContent = "正在调用 Microsoft Store 官方渠道";
    setBadge($("#setupAppState"), "安装中", "pending");
    setGuidedStep("app");
  }
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
  const buttons = [$("#restoreConfigButton")];
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

async function disconnectRouter() {
  if (!tauri?.core || !state.status?.configPresent || state.running || state.lifecycleRunning) return;
  const confirmed = await requestConfirmation({
    title: "断开本地 Router？",
    message: "将从 Codex 配置中移除助手写入的 Router 内容并重启 ChatGPT，恢复官方默认行为。已保存的 Router 地址、Key 和配置备份都会保留，可随时重新应用或恢复。",
    confirmLabel: "断开并重启",
  });
  if (!confirmed) return;
  const button = $("#disconnectRouterButton");
  button.disabled = true;
  try {
    const result = await tauri.core.invoke("disconnect_router");
    if (result.changed) {
      try {
        await tauri.core.invoke("restart_chatgpt");
        showToast("已断开 Router，ChatGPT 已使用官方配置重新打开");
      } catch (restartError) {
        showToast(`${result.message}；${friendlyError(restartError)}`, true);
      }
    } else {
      showToast(result.message);
    }
    state.formDirty = false;
    await refreshStatus({ hydrateForm: true });
  } catch (error) {
    showToast(friendlyError(error), true);
  } finally {
    button.disabled = false;
  }
}

async function refreshLifecycleStatus() {
  if (!tauri?.core || state.lifecycleRunning) return;
  state.lifecycleStatus = null;
  renderLifecycleStatusLoading();
  try {
    const status = await tauri.core.invoke("get_lifecycle_status");
    state.lifecycleStatus = status;
    renderLifecycleStatus(status);
  } catch (error) {
    state.lifecycleStatus = null;
    renderLifecycleStatusError(error);
  }
}

function renderLifecycleStatusLoading() {
  setBadge($("#lifecycleState"), "检查中", "pending");
  $("#assistantUninstallDetail").textContent = "正在检查系统卸载入口。";
  $("#managedConfigDetail").textContent = "正在检查助手管理的配置。";
  $("#assistantDataDetail").textContent = "正在检查本地状态、备份、主题和 Key。";
  $("#officialAppManagementDetail").textContent = "正在检查 ChatGPT 官方应用。";
  [
    "#uninstallAssistantButton",
    "#restoreManagedConfigButton",
    "#deleteAssistantDataButton",
    "#openOfficialAppManagementButton",
  ].forEach((selector) => {
    $(selector).disabled = true;
  });
}

function renderLifecycleStatus(status) {
  setBadge($("#lifecycleState"), "边界已分离", "success");
  const busy = Boolean(state.lifecycleRunning);

  const uninstallButton = $("#uninstallAssistantButton");
  uninstallButton.disabled = busy || !status.assistantUninstallAvailable;
  $("#assistantUninstallDetail").textContent =
    status.assistantUninstallMode === "nsis"
      ? "系统卸载只移除助手程序，默认保留 ChatGPT、Codex 配置和助手数据。"
      : status.assistantUninstallMode === "finder"
        ? "助手会在 Finder 中定位应用；移到废纸篓不会删除 ChatGPT 或配置。"
        : "当前不是完整安装版，请通过系统应用管理或重新安装后卸载。";

  const configButton = $("#restoreManagedConfigButton");
  configButton.disabled = busy || !status.managedConfigPresent;
  $("#managedConfigDetail").textContent = status.managedConfigPresent
    ? "检测到助手管理的 Router 配置；恢复时保留其他用户设置。"
    : "没有助手管理的 Codex 配置，无需恢复。";

  const dataButton = $("#deleteAssistantDataButton");
  dataButton.disabled = busy || !status.assistantDataPresent || status.dataRemovalBlocked;
  $("#assistantDataDetail").textContent = !status.assistantDataPresent
    ? "没有助手运行数据。"
    : status.dataRemovalBlocked
      ? "当前仍被 Codex 配置使用，请先恢复原配置。"
      : "可单独删除本地状态、备份、主题和保存的 Key。";

  const officialButton = $("#openOfficialAppManagementButton");
  officialButton.disabled = busy || !status.officialAppInstalled;
  $("#officialAppManagementDetail").textContent = status.officialAppInstalled
    ? `${status.officialAppTrusted ? "已检测到可信官方应用" : "已检测到应用但可信状态异常"}；卸载由操作系统再次确认。`
    : "当前未检测到 ChatGPT；助手卸载不会改变此状态。";
}

function renderLifecycleStatusError(error) {
  setBadge($("#lifecycleState"), "检查失败", "error");
  $("#assistantUninstallDetail").textContent = friendlyError(error);
  $("#managedConfigDetail").textContent = "未执行任何配置修改。";
  $("#assistantDataDetail").textContent = "未执行任何数据删除。";
  $("#officialAppManagementDetail").textContent = "未执行任何官方应用操作。";
  [
    "#uninstallAssistantButton",
    "#restoreManagedConfigButton",
    "#deleteAssistantDataButton",
    "#openOfficialAppManagementButton",
  ].forEach((selector) => {
    $(selector).disabled = true;
  });
}

async function runLifecycleAction(actionId) {
  if (!tauri?.core || state.running || state.installingApp || state.lifecycleRunning) return;
  const action = LIFECYCLE_ACTIONS[actionId];
  if (!action) return;
  if (action.title) {
    const confirmed = await requestConfirmation({
      title: action.title,
      message: action.message,
      confirmLabel: action.confirmLabel,
      danger: action.danger,
    });
    if (!confirmed) return;
  }

  const buttons = {
    uninstall_assistant: $("#uninstallAssistantButton"),
    restore_pre_assistant_config: $("#restoreManagedConfigButton"),
    delete_assistant_data: $("#deleteAssistantDataButton"),
    open_official_app_management: $("#openOfficialAppManagementButton"),
  };
  const button = buttons[actionId];
  const icon = button.querySelector(".icon");
  const originalIcon = icon.innerHTML;
  const resultNode = $("#lifecycleResult");
  state.lifecycleRunning = actionId;
  if (state.lifecycleStatus) renderLifecycleStatus(state.lifecycleStatus);
  icon.innerHTML = ICONS.loader;
  icon.classList.add("spin");
  resultNode.className = "lifecycle-result hidden";
  try {
    const result = await tauri.core.invoke("run_lifecycle_action", {
      request: {
        actionId,
        confirmation: action.confirmation,
      },
    });
    resultNode.textContent = result.summary;
    resultNode.dataset.actionId = result.actionId || "";
    resultNode.dataset.status = result.status || "";
    resultNode.dataset.changed = String(Boolean(result.changed));
    resultNode.dataset.beforeManagedConfig = String(Boolean(result.before?.managedConfigPresent));
    resultNode.dataset.afterManagedConfig = String(Boolean(result.after?.managedConfigPresent));
    resultNode.dataset.beforeAssistantData = String(Boolean(result.before?.assistantDataPresent));
    resultNode.dataset.afterAssistantData = String(Boolean(result.after?.assistantDataPresent));
    resultNode.className = "lifecycle-result";
    showToast(result.summary);
    if (result.appExitRequested) {
      await tauri.core.invoke("complete_assistant_uninstall_handoff");
    } else {
      state.lifecycleRunning = "";
      state.formDirty = false;
      await refreshStatus({ hydrateForm: true });
      await refreshLifecycleStatus();
    }
  } catch (error) {
    state.lastErrorEnvelope = errorEnvelope(error);
    resultNode.textContent = friendlyError(error);
    resultNode.className = "lifecycle-result error";
    showToast(friendlyError(error), true);
    state.lifecycleRunning = "";
    await refreshLifecycleStatus();
  } finally {
    state.lifecycleRunning = "";
    icon.classList.remove("spin");
    icon.innerHTML = originalIcon;
    if (state.lifecycleStatus) renderLifecycleStatus(state.lifecycleStatus);
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

function escapeHtml(value) {
  return String(value ?? "").replace(/[&<>"']/g, (character) => `&#${character.charCodeAt(0)};`);
}

function themeDisplayName(themeId) {
  if (themeId === "official") return "官方外观";
  if (themeId === "focus") return "专注深色";
  if (themeId === "custom") return "自定义背景";
  const preset = state.presets.find((item) => item.id === themeId);
  if (preset) return preset.name;
  const gallery = state.gallery.find((item) => `gallery:${item.versionId}` === themeId);
  if (gallery) return gallery.name;
  return themeId;
}

async function loadPresetThemes() {
  if (!tauri?.core || state.presetsLoading || state.presets.length) return;
  state.presetsLoading = true;
  try {
    state.presets = await tauri.core.invoke("list_preset_themes");
    renderPresetThemes();
  } catch (error) {
    $("#presetThemeGrid").innerHTML = `<p class="art-grid-note">${escapeHtml(friendlyError(error))}</p>`;
  } finally {
    state.presetsLoading = false;
  }
}

function safeCssColor(value, fallback) {
  const text = String(value || "").trim();
  const valid = /^#[0-9a-fA-F]{3,8}$/.test(text) || /^(rgb|hsl)a?\([0-9.,%\s]+\)$/.test(text);
  return valid ? text : fallback;
}

function renderPresetThemes() {
  const grid = $("#presetThemeGrid");
  grid.innerHTML = "";
  state.presets.forEach((preset) => {
    const card = document.createElement("button");
    card.type = "button";
    card.className = "theme-card art-card";
    card.dataset.theme = preset.id;
    card.innerHTML = `
      <span class="art-preview"></span>
      <span><strong>${escapeHtml(preset.name)}</strong><small>${escapeHtml(preset.author)} · ${escapeHtml(preset.license)}</small></span>
      <span class="theme-check icon" data-icon="check"></span>`;
    card.querySelector(".art-preview").style.backgroundImage = `url(${JSON.stringify(preset.previewDataUrl)})`;
    card.addEventListener("click", () => selectAppearance(preset.id));
    grid.appendChild(card);
  });
  hydrateIcons();
  renderAppearanceSelection();
}

async function loadGalleryThemes({ force = false } = {}) {
  if (!tauri?.core || state.galleryState === "loading") return;
  if (!force && state.galleryState === "ready") return;
  state.galleryState = "loading";
  state.galleryError = "";
  renderGalleryThemes();
  try {
    state.gallery = await tauri.core.invoke("list_gallery_themes");
    state.galleryState = "ready";
  } catch (error) {
    state.galleryState = "error";
    state.galleryError = friendlyError(error);
  }
  renderGalleryThemes();
}

function renderGalleryThemes() {
  const grid = $("#galleryThemeGrid");
  const refreshButton = $("#galleryRefreshButton");
  refreshButton.disabled = state.galleryState === "loading";
  if (state.galleryState === "loading") {
    grid.innerHTML = `<p class="art-grid-note">正在从 dreamskin.cc 加载热门主题…</p>`;
    return;
  }
  if (state.galleryState === "error") {
    grid.innerHTML = `<p class="art-grid-note">${escapeHtml(state.galleryError || "加载失败")}，可点击右上角“刷新”重试。</p>`;
    return;
  }
  if (!state.gallery.length) {
    grid.innerHTML = `<p class="art-grid-note">进入本页后自动加载热门主题。</p>`;
    return;
  }
  grid.innerHTML = "";
  state.gallery.forEach((theme) => {
    const colors = theme.colors || {};
    const from = safeCssColor(colors.accent, "#8095a5");
    const to = safeCssColor(colors.background, theme.appearance === "light" ? "#f2f2f0" : "#0b1118");
    const size = theme.packageBytes ? ` · ${(theme.packageBytes / 1024 / 1024).toFixed(1)} MB` : "";
    const downloaded = theme.downloaded ? `<em class="art-badge">已下载</em>` : "";
    const card = document.createElement("button");
    card.type = "button";
    card.className = "theme-card art-card";
    card.dataset.theme = `gallery:${theme.versionId}`;
    card.innerHTML = `
      <span class="art-preview"></span>
      <span><strong>${escapeHtml(theme.name)}${downloaded}</strong><small>${escapeHtml(theme.author)} · ${escapeHtml(theme.license)} · ${theme.downloads} 次下载${size}</small></span>
      <span class="theme-check icon" data-icon="check"></span>`;
    card.querySelector(".art-preview").style.backgroundImage = `linear-gradient(135deg, ${from}, ${to})`;
    card.addEventListener("click", () => selectAppearance(card.dataset.theme));
    grid.appendChild(card);
  });
  hydrateIcons();
  renderAppearanceSelection();
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
  const themeName = themeDisplayName(state.selectedAppearance);
  const galleryTheme = state.selectedAppearance.startsWith("gallery:")
    ? state.gallery.find((item) => `gallery:${item.versionId}` === state.selectedAppearance)
    : null;
  const needsDownload = galleryTheme && !galleryTheme.downloaded;
  const confirmed = await requestConfirmation({
    title: state.selectedAppearance === "official" ? "恢复官方外观？" : `应用主题「${themeName}」？`,
    message:
      state.selectedAppearance === "official"
        ? "恢复官方外观需要重新启动 ChatGPT。请先保存尚未发送的内容。"
        : needsDownload
          ? `将先从 dreamskin.cc 下载该主题（约 ${(galleryTheme.packageBytes / 1024 / 1024).toFixed(1)} MB，许可：${galleryTheme.license}），然后重启 ChatGPT 应用。请先保存尚未发送的内容。`
          : "应用外观需要重新启动 ChatGPT。请先保存尚未发送的内容。",
    confirmLabel: "应用并重启",
  });
  if (!confirmed) return;
  state.appearanceRequestId += 1;
  const button = $("#applyAppearanceButton");
  button.disabled = true;
  button.textContent = needsDownload ? "正在下载并应用…" : "正在重启并应用…";
  const badge = $("#appearanceStatus");
  badge.textContent = "应用中";
  badge.className = "status-badge warning";
  try {
    const appearance = await tauri.core.invoke("apply_appearance", { theme: state.selectedAppearance });
    if (galleryTheme) {
      galleryTheme.downloaded = true;
      renderGalleryThemes();
    }
    badge.textContent = appearance.active ? "已生效" : "官方外观";
    badge.className = "status-badge success";
    $("#appearanceMessage").textContent = appearance.message;
    showToast("ChatGPT 外观已更新");
  } catch (error) {
    state.lastErrorEnvelope = errorEnvelope(error);
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
  const failedStage = (state.lastResultPayload?.stages || []).find(
    (stage) => stage.stage === "rollback" && stage.status === "failed",
  ) || (state.lastResultPayload?.stages || []).find((stage) => stage.status === "failed");
  const text = [
    `Codex Assistant: 0.8.8`,
    `Status schema: ${status.schemaVersion || "legacy"}`,
    `Platform: ${status.platform} ${formatArchitecture(status.architecture)}`,
    `Overall: ${status.overall || (status.ready ? "ready" : "action_required")}`,
    `ChatGPT: state=${status.appState || (status.appInstalled ? "installed" : "not_installed")} trusted=${Boolean(status.appTrusted)} source=${status.appSource || "unknown"} (${redactDiagnosticText(status.appDetail)})`,
    `Codex config: ${status.config?.state || (status.configPresent ? "valid" : "missing")}`,
    `Config source: ${status.config?.effectiveSource || "unknown"}`,
    `Router: ${status.router?.state || (status.routerReachable ? "reachable" : "unreachable")} (${redactDiagnosticText(status.routerDetail)})`,
    `Gateway: ${safeGateway(status.configuredGateway) || "not configured"}`,
    `Model: ${redactDiagnosticText(status.configuredModel || "not configured")}`,
    `Last verified: ${status.router?.lastVerifiedAt || "not verified"}`,
    `Config path: ${redactDiagnosticText(status.configPath)}`,
    `Config backup: ${status.backupAvailable ? "available" : "missing"}`,
    `Last transaction: ${status.config?.lastTransactionId || "none"}`,
    `Recommended action: ${status.recommendedAction?.id || "none"}`,
    `Appearance: light`,
    ...(failedStage
      ? [
          `Last failed stage: ${failedStage.stage}`,
          `Last failure: ${redactDiagnosticText(failedStage.message || state.lastResultPayload?.summary || "unknown")}`,
        ]
      : []),
  ].join("\n");
  try {
    await navigator.clipboard.writeText(text);
    showToast("诊断信息已复制");
  } catch {
    showToast("复制失败", true);
  }
}

async function exportDiagnosticBundle() {
  const button = $("#diagnosticExportButton");
  if (!tauri?.core) return showToast("当前环境无法导出诊断包", true);
  const lastError = state.lastResultPayload?.error || {};
  button.disabled = true;
  button.querySelector(".icon").innerHTML = ICONS.loader;
  button.querySelector(".icon").classList.add("spin");
  try {
    const bundle = await tauri.core.invoke("export_diagnostics", {
      request: {
        supportId: lastError.supportId || "",
        errorCode: lastError.code || "",
        errorStage: lastError.stage || "",
        suggestedAction: lastError.suggestedAction || "",
      },
    });
    if (!bundle.savedPath) throw new Error("诊断包未写入下载目录");
    showToast(`诊断包已保存到 Downloads · ${bundle.supportId}`);
  } catch (error) {
    showToast(friendlyError(error, "诊断包导出失败"), true);
  } finally {
    button.disabled = false;
    button.querySelector(".icon").classList.remove("spin");
    button.querySelector(".icon").innerHTML = ICONS.download;
  }
}

function safeGateway(gateway) {
  if (!gateway) return "";
  try {
    const parsed = new URL(gateway);
    parsed.username = "";
    parsed.password = "";
    ["key", "token", "api_key", "apikey", "access_token"].forEach((name) => {
      if (parsed.searchParams.has(name)) parsed.searchParams.set(name, "REDACTED");
    });
    return parsed.toString().replace(/\/$/, gateway.endsWith("/") ? "/" : "");
  } catch {
    return redactDiagnosticText(gateway);
  }
}

function redactDiagnosticText(value) {
  return String(value || "")
    .replace(/[A-Za-z]:\\Users\\[^\\\s)]+/gi, "%USERPROFILE%")
    .replace(/\/Users\/[^/\s)]+/g, "~")
    .replace(/\/home\/[^/\s)]+/g, "~")
    .replace(/((?:api[_-]?key|access[_-]?token|token|key)=)[^&\s]+/gi, "$1<redacted>")
    .replace(/(Bearer\s+)\S+/gi, "$1<redacted>");
}

function formatTimestamp(value) {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(date);
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

function requestConfirmation({ title, message, confirmLabel = "确认", danger = false }) {
  if (state.confirmResolver) closeConfirmation(false);
  state.confirmReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  $("#confirmTitle").textContent = title;
  $("#confirmMessage").textContent = message;
  $("#confirmAcceptButton").textContent = confirmLabel;
  $("#confirmIcon").classList.toggle("danger", danger);
  $("#confirmAcceptButton").classList.toggle("danger", danger);
  $("#confirmOverlay").classList.remove("hidden");
  window.setTimeout(() => $("#confirmAcceptButton").focus(), 0);
  return new Promise((resolve) => {
    state.confirmResolver = resolve;
  });
}

function closeConfirmation(accepted) {
  const resolver = state.confirmResolver;
  const returnFocus = state.confirmReturnFocus;
  state.confirmResolver = null;
  state.confirmReturnFocus = null;
  $("#confirmOverlay").classList.add("hidden");
  if (resolver) resolver(accepted);
  if (returnFocus?.isConnected) window.setTimeout(() => returnFocus.focus(), 0);
}

function friendlyError(error) {
  const envelope = errorEnvelope(error);
  if (envelope) {
    if (envelope.title && envelope.message && envelope.title !== envelope.message) {
      return `${envelope.title}：${envelope.message}`;
    }
    return envelope.message || envelope.title || `操作未完成（${envelope.code}）`;
  }
  const text = String(error?.message || error || "未知错误");
  return text
    .replace(/^Error:\s*/i, "")
    .replace(/Transport\([^)]*\)/g, "连接失败")
    .replace(/Bearer\s+[^\s]+/gi, "Bearer [redacted]");
}

function errorEnvelope(error) {
  if (error && typeof error === "object" && error.code && error.schemaVersion) return error;
  const candidate = error?.message;
  if (candidate && typeof candidate === "object" && candidate.code && candidate.schemaVersion) return candidate;
  if (typeof candidate === "string" || typeof error === "string") {
    const text = typeof candidate === "string" ? candidate : error;
    try {
      const parsed = JSON.parse(text);
      if (parsed?.code && parsed?.schemaVersion) return parsed;
    } catch {
      return null;
    }
  }
  return null;
}

function errorCode(error) {
  return errorEnvelope(error)?.code || "LEGACY_UNCLASSIFIED";
}

function normalizeSystemStatus(status = {}) {
  if (status.schemaVersion === 1 && status.app && status.router && status.config) {
    return {
      ...status,
      appInstalled: Boolean(status.app.installed),
      appState: status.app.state || (status.app.installed ? "installed" : "not_installed"),
      appTrusted: Boolean(status.app.trusted),
      appSource: status.app.source || "unknown",
      appName: status.app.name || "ChatGPT",
      appVersion: status.app.version || null,
      appDetail: status.app.detail || "",
      configPresent: Boolean(status.config.present),
      configPath: status.config.path || "",
      routerReachable: Boolean(status.router.reachable),
      routerDetail: status.router.detail || "",
      configuredGateway: status.router.gateway || null,
      configuredModel: status.router.model || null,
      keyConfigured: Boolean(status.router.keyConfigured),
      backupAvailable: Boolean(status.config.backupAvailable),
      ready: status.overall === "ready",
    };
  }

  const ready = Boolean(status.ready);
  const action = ready
    ? { id: "open_chatgpt", label: "打开 ChatGPT" }
    : !status.appInstalled && status.platform === "Windows"
      ? { id: "install_chatgpt", label: "安装并配置" }
      : { id: "configure_router", label: "开始配置" };
  return {
    ...status,
    schemaVersion: status.schemaVersion || 0,
    overall: ready ? "ready" : "action_required",
    appState: status.appInstalled ? "installed" : "not_installed",
    appTrusted: Boolean(status.appInstalled),
    appSource: status.appInstalled ? "legacy-detection" : "not-detected",
    recommendedAction: status.recommendedAction || action,
  };
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
