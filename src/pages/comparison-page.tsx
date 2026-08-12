import { Button } from "@heroui/react";
import type { TFunction } from "i18next";
import { type FormEvent, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { checkAgentProcesses } from "../api/agent";
import { checkClaudeLogin, runClaudeTask } from "../api/claude";
import { checkCodexLogin, runCodexTask } from "../api/codex";
import { checkWorkBuddyLogin, runWorkBuddyTask } from "../api/workbuddy";
import { AgentComparisonCard } from "../components/agent-comparison-card";
import { AgentSelectionCard } from "../components/agent-selection-card";
import type {
	AgentKind,
	AgentProcessStates,
	AgentRunResult,
	AgentRuntimeStatus,
} from "../types/agent";
import { getErrorMessage } from "../utils/error";

type LoginState =
	| { status: "checking" }
	| { status: "resolved"; value: AgentRuntimeStatus }
	| { status: "failed" };

type LoginStates = Record<AgentKind, LoginState>;

type ProcessState =
	| { status: "checking" }
	| { status: "resolved"; value: AgentProcessStates }
	| { status: "failed" };

type AgentRunState =
	| { status: "idle" }
	| { status: "running" }
	| {
			/** Completed metrics and response from this product. */
			status: "succeeded";
			/** Measured result returned by the local Agent runtime. */
			result: AgentRunResult;
	  }
	| {
			/** Failed run that does not interrupt the other selected products. */
			status: "failed";
			/** Localized error presented inside this product's comparison card. */
			errorMessage: string;
	  };

type AgentStatusDisplay = {
	/** User-visible installation, login, and process state. */
	message: string;
	/** Tailwind color class for the status indicator. */
	tone: string;
	/** Whether this product can participate in a comparison run. */
	isReady: boolean;
};

const AGENT_KINDS = ["codex", "claude", "workbuddy"] as const;

const AGENT_LOGIN_CHECKS: Record<AgentKind, () => Promise<AgentRuntimeStatus>> =
	{
		claude: checkClaudeLogin,
		codex: checkCodexLogin,
		workbuddy: checkWorkBuddyLogin,
	};

const AGENT_TASK_RUNNERS: Record<
	AgentKind,
	(query: string) => Promise<AgentRunResult>
> = {
	claude: runClaudeTask,
	codex: runCodexTask,
	workbuddy: runWorkBuddyTask,
};

/**
 * Resolves a product's selectable state from its live login and process probes.
 *
 * @example
 * resolveAgentStatus("codex", { status: "checking" }, processState, t);
 */
const resolveAgentStatus = (
	agent: AgentKind,
	loginState: LoginState,
	processState: ProcessState,
	t: TFunction,
): AgentStatusDisplay => {
	const agentName = t(`agentNames.${agent}`);
	if (loginState.status === "checking") {
		return {
			message: t("checkingLogin", { agent: agentName }),
			tone: "bg-amber-400",
			isReady: false,
		};
	}
	if (loginState.status === "failed") {
		return {
			message: t("loginCheckFailed", { agent: agentName }),
			tone: "bg-rose-400",
			isReady: false,
		};
	}
	if (!loginState.value.installed) {
		return {
			message: t("notInstalled", { agent: agentName }),
			tone: "bg-rose-400",
			isReady: false,
		};
	}
	if (!loginState.value.loggedIn) {
		return {
			message: t("notLoggedIn", { agent: agentName }),
			tone: "bg-rose-400",
			isReady: false,
		};
	}
	if (processState.status === "checking") {
		return {
			message: t("checkingProcess", { agent: agentName }),
			tone: "bg-amber-400",
			isReady: true,
		};
	}
	if (processState.status === "failed") {
		return {
			message: t("processCheckFailed", { agent: agentName }),
			tone: "bg-rose-400",
			isReady: true,
		};
	}
	if (processState.value[agent]) {
		return {
			message: t("agentRunning", { agent: agentName }),
			tone: "bg-emerald-400",
			isReady: true,
		};
	}
	return {
		message: t("agentReady", { agent: agentName }),
		tone: "bg-sky-400",
		isReady: true,
	};
};

/**
 * Renders multi-product selection, one shared task, and comparable run metrics.
 *
 * @example
 * <ComparisonPage />
 */
const ComparisonPage = () => {
	const { t, i18n } = useTranslation();
	const [selectedAgents, setSelectedAgents] = useState<AgentKind[]>([
		...AGENT_KINDS,
	]);
	const [loginStates, setLoginStates] = useState<LoginStates>({
		claude: { status: "checking" },
		codex: { status: "checking" },
		workbuddy: { status: "checking" },
	});
	const [query, setQuery] = useState("");
	const [processState, setProcessState] = useState<ProcessState>({
		status: "checking",
	});
	const [runStates, setRunStates] = useState<Record<AgentKind, AgentRunState>>({
		claude: { status: "idle" },
		codex: { status: "idle" },
		workbuddy: { status: "idle" },
	});
	const isRunning = Object.values(runStates).some(
		(state) => state.status === "running",
	);

	useEffect(() => {
		if (isRunning) {
			return;
		}

		let isActive = true;
		const pendingAgents = new Set<AgentKind>();

		/** Refreshes every login state independently without overlapping probes. */
		const refreshLoginStates = () => {
			for (const agent of AGENT_KINDS) {
				if (pendingAgents.has(agent)) {
					continue;
				}

				pendingAgents.add(agent);
				AGENT_LOGIN_CHECKS[agent]()
					.then((value) => {
						if (isActive) {
							setLoginStates((current) => ({
								...current,
								[agent]: { status: "resolved", value },
							}));
						}
					})
					.catch(() => {
						if (isActive) {
							setLoginStates((current) => ({
								...current,
								[agent]: { status: "failed" },
							}));
						}
					})
					.finally(() => pendingAgents.delete(agent));
			}
		};

		refreshLoginStates();
		const intervalId = window.setInterval(refreshLoginStates, 5000);
		window.addEventListener("focus", refreshLoginStates);

		return () => {
			isActive = false;
			window.clearInterval(intervalId);
			window.removeEventListener("focus", refreshLoginStates);
		};
	}, [isRunning]);

	useEffect(() => {
		let isActive = true;
		let isChecking = false;

		/** Refreshes the local process snapshot without overlapping requests. */
		const refreshAgentProcesses = async () => {
			if (isChecking) {
				return;
			}
			isChecking = true;
			try {
				const value = await checkAgentProcesses();
				if (isActive) {
					setProcessState({ status: "resolved", value });
				}
			} catch {
				if (isActive) {
					setProcessState((current) =>
						current.status === "checking" ? { status: "failed" } : current,
					);
				}
			} finally {
				isChecking = false;
			}
		};

		refreshAgentProcesses();
		const intervalId = window.setInterval(refreshAgentProcesses, 1000);

		return () => {
			isActive = false;
			window.clearInterval(intervalId);
		};
	}, []);

	const agentDisplays = AGENT_KINDS.reduce<
		Record<AgentKind, AgentStatusDisplay>
	>(
		(displays, agent) => {
			displays[agent] = resolveAgentStatus(
				agent,
				loginStates[agent],
				processState,
				t,
			);
			return displays;
		},
		{} as Record<AgentKind, AgentStatusDisplay>,
	);
	const runnableAgents = AGENT_KINDS.filter(
		(agent) => selectedAgents.includes(agent) && agentDisplays[agent].isReady,
	);
	const comparisonAgents = AGENT_KINDS.filter(
		(agent) => runStates[agent].status !== "idle",
	);
	const numberLocale = i18n.resolvedLanguage ?? "en-US";

	/** Changes and persists the active UI language through i18next. */
	const changeLanguage = async (language: "en-US" | "zh-CN") => {
		await i18n.changeLanguage(language);
	};

	/**
	 * Includes or excludes one ready product from the next comparison run.
	 *
	 * @example
	 * toggleAgent("workbuddy");
	 */
	const toggleAgent = (agent: AgentKind) => {
		if (isRunning || !agentDisplays[agent].isReady) {
			return;
		}
		setSelectedAgents((current) =>
			current.includes(agent)
				? current.filter((candidate) => candidate !== agent)
				: [...current, agent],
		);
		setRunStates({
			claude: { status: "idle" },
			codex: { status: "idle" },
			workbuddy: { status: "idle" },
		});
	};

	/**
	 * Sends one query to every selected product concurrently and records each result independently.
	 *
	 * @example
	 * onSubmit(event);
	 */
	const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
		event.preventDefault();
		const normalizedQuery = query.trim();
		if (
			isRunning ||
			normalizedQuery.length === 0 ||
			runnableAgents.length === 0
		) {
			return;
		}

		const activeAgents = [...runnableAgents];
		setRunStates({
			claude: activeAgents.includes("claude")
				? { status: "running" }
				: { status: "idle" },
			codex: activeAgents.includes("codex")
				? { status: "running" }
				: { status: "idle" },
			workbuddy: activeAgents.includes("workbuddy")
				? { status: "running" }
				: { status: "idle" },
		});

		await Promise.all(
			activeAgents.map(async (agent) => {
				try {
					const result = await AGENT_TASK_RUNNERS[agent](normalizedQuery);
					setRunStates((current) => ({
						...current,
						[agent]: { status: "succeeded", result },
					}));
				} catch (error) {
					setRunStates((current) => ({
						...current,
						[agent]: {
							status: "failed",
							errorMessage: getErrorMessage(error, t("requestFailed")),
						},
					}));
				}
			}),
		);
	};

	return (
		<main className="min-h-screen bg-slate-950 px-5 py-10 text-white sm:px-8">
			<section className="mx-auto max-w-7xl">
				<header className="mb-8">
					<div className="mb-8 flex items-center justify-between gap-4">
						<p className="font-semibold tracking-tight">{t("appName")}</p>
						<fieldset
							aria-label={t("languageSelection")}
							className="inline-flex gap-1 rounded-lg border border-white/10 bg-white/5 p-1"
						>
							{(["zh-CN", "en-US"] as const).map((language) => (
								<button
									aria-pressed={i18n.resolvedLanguage === language}
									className="rounded-md px-2 py-1 text-xs text-slate-400 transition hover:text-white aria-pressed:bg-white/10 aria-pressed:text-white"
									key={language}
									onClick={() => changeLanguage(language)}
									type="button"
								>
									{t(
										language === "zh-CN" ? "languages.zhCN" : "languages.enUS",
									)}
								</button>
							))}
						</fieldset>
					</div>
					<p className="mb-2 text-sm font-medium text-indigo-300">
						{t("tagline")}
					</p>
					<h1 className="max-w-4xl text-4xl font-semibold tracking-tight sm:text-5xl">
						{t("title")}
					</h1>
					<p className="mt-4 max-w-3xl leading-7 text-slate-400">
						{t("description")}
					</p>
				</header>

				<fieldset aria-label={t("agentSelection")} className="mb-6">
					<legend className="mb-3 text-sm font-medium text-slate-300">
						{t("agentSelection")}
					</legend>
					<div className="grid gap-3 md:grid-cols-3">
						{AGENT_KINDS.map((agent) => {
							const loginState = loginStates[agent];
							const runtimeStatus =
								loginState.status === "resolved" ? loginState.value : null;
							const isSelected =
								selectedAgents.includes(agent) && agentDisplays[agent].isReady;

							return (
								<AgentSelectionCard
									agent={agent}
									isDisabled={isRunning || !agentDisplays[agent].isReady}
									isSelected={isSelected}
									key={agent}
									onToggle={toggleAgent}
									runtimeStatus={runtimeStatus}
									statusMessage={agentDisplays[agent].message}
									statusTone={agentDisplays[agent].tone}
								/>
							);
						})}
					</div>
				</fieldset>

				<form
					className="rounded-2xl border border-white/10 bg-white/5 p-5"
					onSubmit={onSubmit}
				>
					<label
						className="mb-2 block text-sm font-medium"
						htmlFor="agent-query"
					>
						{t("queryLabel")}
					</label>
					<textarea
						className="min-h-32 w-full resize-y rounded-xl border border-white/10 bg-slate-950/80 p-4 text-sm leading-6 text-slate-100 outline-none transition focus:border-indigo-400"
						disabled={isRunning}
						id="agent-query"
						maxLength={16000}
						onChange={(event) => setQuery(event.target.value)}
						placeholder={t("queryPlaceholder")}
						value={query}
					/>
					<div className="mt-4 flex flex-wrap items-center justify-between gap-3">
						<p className="text-xs text-slate-400">
							{t("selectedAgents", { count: runnableAgents.length })}
						</p>
						<Button
							isDisabled={
								isRunning ||
								query.trim().length === 0 ||
								runnableAgents.length === 0
							}
							type="submit"
							variant="primary"
						>
							{isRunning
								? t("comparingAgents", { count: runnableAgents.length })
								: t("compareAgents", { count: runnableAgents.length })}
						</Button>
					</div>
				</form>

				{comparisonAgents.length > 0 ? (
					<section className="mt-8" aria-labelledby="comparison-title">
						<h2
							className="mb-4 text-sm font-medium text-slate-300"
							id="comparison-title"
						>
							{t("comparisonTitle")}
						</h2>
						<div className="grid items-start gap-4 lg:grid-cols-3">
							{comparisonAgents.map((agent) => {
								const runState = runStates[agent];

								return (
									<AgentComparisonCard
										agent={agent}
										errorMessage={
											runState.status === "failed"
												? runState.errorMessage
												: null
										}
										isRunning={runState.status === "running"}
										key={agent}
										numberLocale={numberLocale}
										result={
											runState.status === "succeeded" ? runState.result : null
										}
									/>
								);
							})}
						</div>
					</section>
				) : null}
			</section>
		</main>
	);
};

export { ComparisonPage };
