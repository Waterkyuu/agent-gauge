const ZH_CN = {
	appName: "AgentGauge",
	tagline: "本地 Codex 性能测量",
	title: "发送一个任务，得到真实运行指标。",
	description:
		"直接连接本机 Codex，监听任务从接收到首 token，再到完整结束的全过程。",
	queryLabel: "任务内容",
	queryPlaceholder: "例如：概括这个仓库的主要功能，不要修改文件。",
	send: "发送任务",
	running: "Codex 正在执行…",
	checkingLogin: "正在检查本地 Codex 登录状态…",
	notInstalled: "未找到本地 Codex，请先安装 Codex CLI。",
	notLoggedIn: "本地 Codex 尚未登录，请先运行 codex login。",
	loginCheckFailed: "无法检查本地 Codex 登录状态。",
	/** Formats the safe local authentication mode. Example: `loggedIn("ChatGPT")`. */
	loggedIn: (method: string) => `已通过 ${method} 登录`,
	metricsTitle: "本次运行",
	firstToken: "首 token",
	totalDuration: "总耗时",
	totalTokens: "总 token",
	inputTokens: "输入",
	outputTokens: "输出",
	reasoningTokens: "推理",
	responseTitle: "Codex 返回",
	metricUnavailable: "不可用",
	requestFailed: "任务执行失败，请重试。",
} as const;

export { ZH_CN };
