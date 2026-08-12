const ZH_CN = {
	appName: "AgentGauge",
	agentNames: {
		codex: "Codex",
		workbuddy: "WorkBuddy",
	},
	agentSelection: "选择 Agent 产品",
	tagline: "本地 Agent 性能测量",
	title: "发送一个任务，得到真实运行指标。",
	description:
		"直接连接本机 Agent，监听任务从接收到首 token，再到完整结束的全过程。",
	queryLabel: "任务内容",
	queryPlaceholder: "例如：概括这个仓库的主要功能，不要修改文件。",
	/** Formats the selected task target. Example: `send("WorkBuddy")`. */
	send: (agent: string) => `发送给 ${agent}`,
	/** Formats the active execution state. Example: `running("Codex")`. */
	running: (agent: string) => `${agent} 正在执行…`,
	/** Formats the login probe state. Example: `checkingLogin("Codex")`. */
	checkingLogin: (agent: string) => `正在检查本地 ${agent} 登录状态…`,
	/** Formats the missing product state. Example: `notInstalled("WorkBuddy")`. */
	notInstalled: (agent: string) => `未找到本地 ${agent}。`,
	/** Formats the signed-out product state. Example: `notLoggedIn("WorkBuddy")`. */
	notLoggedIn: (agent: string) => `本地 ${agent} 尚未登录。`,
	/** Formats a failed product probe. Example: `loginCheckFailed("Codex")`. */
	loginCheckFailed: (agent: string) => `无法检查本地 ${agent} 登录状态。`,
	/** Formats the safe local authentication mode. Example: `loggedIn("Codex", "ChatGPT")`. */
	loggedIn: (agent: string, method: string | null) =>
		method ? `${agent}：已通过 ${method} 登录` : `${agent}：已登录`,
	/** Formats the runtime configuration label. Example: `runtimeConfiguration("Codex")`. */
	runtimeConfiguration: (agent: string) => `当前 ${agent} 配置`,
	currentModel: "当前模型",
	reasoningEffort: "思考强度",
	reasoningEffortLevels: {
		enabled: "默认",
		none: "无",
		minimal: "最低",
		low: "低",
		medium: "中",
		high: "高",
		xhigh: "超高",
		max: "最高",
	},
	metricsTitle: "本次运行",
	firstToken: "首 token",
	totalDuration: "总耗时",
	totalTokens: "总 token",
	inputTokens: "输入",
	outputTokens: "输出",
	reasoningTokens: "推理",
	/** Formats the response heading. Example: `responseTitle("WorkBuddy")`. */
	responseTitle: (agent: string) => `${agent} 返回`,
	metricUnavailable: "不可用",
	requestFailed: "任务执行失败，请重试。",
} as const;

export { ZH_CN };
