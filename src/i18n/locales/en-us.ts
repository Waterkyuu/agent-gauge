const enUS = {
	translation: {
		appName: "AgentGauge",
		appEdition: "Local agent lab",
		mainNavigation: "Main navigation",
		collapseSidebar: "Collapse sidebar",
		expandSidebar: "Expand sidebar",
		loadingPage: "Loading page",
		common: {
			search: "Search",
			clearSearch: "Clear search",
		},
		navigation: {
			compare: "Compare",
			comparisonHistory: "History",
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
			"Select local agent products, run the same task in parallel, and compare response speed, thinking time, tool use, token usage, and responses.",
		queryLabel: "Task query",
		queryPlaceholder:
			"For example: Summarize the main features of this repository without modifying files.",
		selectedAgents: "{{count}} available agents selected",
		compareAgents: "Compare {{count}} agents",
		comparingAgents: "Comparing {{count}} agents…",
		checkingLogin: "Checking the local {{agent}} login status…",
		checkingProcess: "Checking whether local {{agent}} has started…",
		processCheckFailed: "Unable to check whether local {{agent}} has started.",
		agentRunning: "Started",
		agentReady: "Not started",
		notInstalled: "Not installed",
		notLoggedIn: "Not signed in",
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
		agentRunFinished: "{{task}} task has finished in {{agent}}",
		viewResult: "View the result",
		firstToken: "First token",
		totalDuration: "Total duration",
		totalTokens: "Total tokens",
		inputTokens: "Input",
		outputTokens: "Output",
		reasoningTokens: "Reasoning",
		thinkingDuration: "Thinking time",
		toolCallCount: "Tool calls",
		toolCallsTitle: "Tool details",
		noToolCalls: "No tools called in this run",
		responseTitle: "Response",
		metricUnavailable: "Unavailable",
		requestFailed: "The task failed. Please try again.",
		comparisonHistory: {
			tagline: "Local performance records",
			title: "Comparison history",
			description:
				"Review every comparison task, runtime configuration, metric, tool call, and final response.",
			listTitle: "Comparison records",
			loading: "Loading comparison history",
			loadingDetail: "Loading comparison detail",
			loadFailed: "Comparison history could not be loaded.",
			detailFailed: "This comparison detail could not be loaded.",
			emptyTitle: "No comparison history yet",
			emptyDescription:
				"Complete a performance comparison and its results will appear here automatically.",
			selectPrompt: "Select a history record to view its complete result.",
			loadMore: "Load more",
			saveFailed:
				"The results finished, but comparison history could not be saved.",
		},
		runBoard: {
			title: "Agent run board",
			description:
				"Track local agent tasks that are running, awaiting input, completed, or interrupted.",
			searchPlaceholder: "Search agent products",
			noSearchResults: "No matching agent tasks.",
			layoutSelection: "Switch run board layout",
			verticalLayout: "Vertical panel",
			horizontalLayout: "Horizontal panel",
			empty: "No tasks in this status.",
			status: {
				running: "Running",
				waiting: "Awaiting user",
				finish: "Finish",
				error: "Error",
			},
			statusDescription: {
				running: "In progress",
				waiting: "Needs an answer to continue",
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
						"The target database version is required before migration steps can continue.",
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
