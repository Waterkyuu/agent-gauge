const enUS = {
	translation: {
		appName: "AgentGauge",
		appEdition: "Local agent lab",
		mainNavigation: "Main navigation",
		collapseSidebar: "Collapse sidebar",
		expandSidebar: "Expand sidebar",
		loadingPage: "Loading page",
		navigation: {
			compare: "Compare",
			runs: "Run board",
		},
		agentNames: {
			claude: "Claude Code",
			codex: "Codex",
			workbuddy: "WorkBuddy",
		},
		agentSelection: "Select agents to compare",
		languageSelection: "Switch language",
		languages: {
			zhCN: "中文",
			enUS: "English",
		},
		tagline: "Local agent performance measurement",
		title: "Run one task across agents. Compare what matters.",
		description:
			"Select local agent products, run the same task in parallel, and compare first-token latency, total duration, token usage, and responses.",
		queryLabel: "Task query",
		queryPlaceholder:
			"For example: Summarize the main features of this repository without modifying files.",
		selectedAgents: "{{count}} available agents selected",
		compareAgents: "Compare {{count}} agents",
		comparingAgents: "Comparing {{count}} agents…",
		checkingLogin: "Checking the local {{agent}} login status…",
		checkingProcess: "Checking whether local {{agent}} is running…",
		processCheckFailed: "Unable to check whether local {{agent}} is running.",
		agentRunning: "{{agent}}: running",
		agentReady: "{{agent}}: ready, not running",
		notInstalled: "Local {{agent}} was not found.",
		notLoggedIn: "Local {{agent}} is not signed in.",
		loginCheckFailed: "Unable to check the local {{agent}} login status.",
		loggedIn: "{{agent}}: signed in",
		loggedInWithMethod: "{{agent}}: signed in with {{method}}",
		runtimeConfiguration: "Current {{agent}} configuration",
		currentModel: "Current model",
		reasoningEffort: "Reasoning effort",
		reasoningEffortLevels: {
			enabled: "Default",
			none: "None",
			minimal: "Minimal",
			low: "Low",
			medium: "Medium",
			high: "High",
			xhigh: "X-High",
			max: "Max",
		},
		comparisonTitle: "Comparison results",
		comparisonResult: "{{agent}} comparison result",
		agentRunRunning: "Running…",
		firstToken: "First token",
		totalDuration: "Total duration",
		totalTokens: "Total tokens",
		inputTokens: "Input",
		outputTokens: "Output",
		reasoningTokens: "Reasoning",
		responseTitle: "Response",
		metricUnavailable: "Unavailable",
		requestFailed: "The task failed. Please try again.",
		runBoard: {
			title: "Agent run board",
			description:
				"Track local agent tasks by status, from work in progress to completed and interrupted runs.",
			empty: "No tasks in this status.",
			status: {
				running: "Running",
				finish: "Finish",
				error: "Error",
			},
			statusDescription: {
				running: "In progress",
				finish: "Completed successfully",
				error: "Needs attention",
			},
			items: {
				repositoryAudit: {
					title: "Repository architecture audit",
					description:
						"Map module boundaries and flag highly coupled code paths.",
				},
				releaseNotes: {
					title: "Generate release notes",
					description:
						"Summarize this week's commits into a user-facing update.",
				},
				apiComparison: {
					title: "API performance comparison",
					description:
						"Compare response latency and token use across three agents.",
				},
				testCoverage: {
					title: "Test coverage analysis",
					description:
						"Find critical business branches without regression coverage.",
				},
				migrationPlan: {
					title: "Database migration plan",
					description:
						"The connection stopped while migration steps were generated.",
				},
				dependencyScan: {
					title: "Dependency security scan",
					description:
						"The dependency manifest failed to parse; the lockfile needs an update.",
				},
			},
		},
	},
} as const;

export { enUS };
