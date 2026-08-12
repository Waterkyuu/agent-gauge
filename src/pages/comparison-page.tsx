import { MagicWand, Play } from "@gravity-ui/icons";
import { Button, Card, TextArea } from "@heroui/react";
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
			tone: "bg-zinc-400",
			isReady: false,
		};
	}
	if (loginState.status === "failed") {
		return {
			message: t("loginCheckFailed", { agent: agentName }),
			tone: "bg-zinc-300",
			isReady: false,
		};
	}
	if (!loginState.value.installed) {
		return {
			message: t("notInstalled", { agent: agentName }),
			tone: "bg-zinc-300",
			isReady: false,
		};
	}
	if (!loginState.value.loggedIn) {
		return {
			message: t("notLoggedIn", { agent: agentName }),
			tone: "bg-zinc-300",
			isReady: false,
		};
	}
	if (processState.status === "checking") {
		return {
			message: t("checkingProcess", { agent: agentName }),
			tone: "bg-zinc-400",
			isReady: true,
		};
	}
	if (processState.status === "failed") {
		return {
			message: t("processCheckFailed", { agent: agentName }),
			tone: "bg-zinc-300",
			isReady: true,
		};
	}
	if (processState.value[agent]) {
		return {
			message: t("agentRunning", { agent: agentName }),
			tone: "bg-zinc-800",
			isReady: true,
		};
	}
	return {
		message: t("agentReady", { agent: agentName }),
		tone: "bg-zinc-500",
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
		<main className="mx-auto max-w-[1320px] px-4 py-8 sm:px-6 lg:px-8 lg:py-10">
			<header className="mb-8 border-b border-[var(--app-line)] pb-7">
				<p className="mb-2 text-sm font-medium text-[var(--app-muted)]">
					{t("tagline")}
				</p>
				<h1 className="max-w-3xl text-3xl font-semibold leading-[1.08] tracking-[-0.04em] sm:text-4xl">
					{t("title")}
				</h1>
				<p className="mt-3 max-w-[65ch] text-sm leading-6 text-[var(--app-muted)] sm:text-base">
					{t("description")}
				</p>
			</header>

			<div className="grid items-start gap-5 xl:grid-cols-[minmax(360px,0.9fr)_minmax(0,1.35fr)]">
				<section className="overflow-hidden rounded-xl border border-[var(--app-line)] bg-[var(--app-surface)]">
					<header className="flex items-center justify-between gap-4 border-b border-[var(--app-line)] px-4 py-3.5">
						<h2 className="text-sm font-semibold">{t("agentSelection")}</h2>
						<span className="font-mono text-xs text-[var(--app-muted)] tabular-nums">
							{runnableAgents.length} / {AGENT_KINDS.length}
						</span>
					</header>
					<fieldset aria-label={t("agentSelection")}>
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
					</fieldset>
				</section>

				<Card className="overflow-hidden rounded-xl border border-[var(--app-line)] bg-[var(--app-raised)] shadow-none">
					<form onSubmit={onSubmit}>
						<Card.Header className="!flex-row !justify-start gap-3 border-b border-[var(--app-line)] px-4 py-3.5 sm:px-5">
							<span className="grid size-8 place-items-center rounded-lg bg-[var(--app-hover)] text-[var(--app-muted)]">
								<MagicWand aria-hidden="true" className="size-4" />
							</span>
							<p className="text-sm font-semibold">{t("queryLabel")}</p>
						</Card.Header>
						<Card.Content className="p-4 sm:p-5">
							<label
								className="mb-2 block text-xs font-medium text-[var(--app-muted)]"
								htmlFor="agent-query"
							>
								{t("queryLabel")}
							</label>
							<TextArea
								className="min-h-56 w-full resize-y rounded-lg border border-[var(--app-line)] bg-[var(--app-canvas)] p-4 text-sm leading-6 text-[var(--app-ink)] outline-none transition-colors placeholder:text-[var(--app-faint)] focus:border-zinc-700 focus:bg-[var(--app-raised)] focus:ring-2 focus:ring-zinc-200"
								disabled={isRunning}
								id="agent-query"
								maxLength={16000}
								onChange={(event) => setQuery(event.target.value)}
								placeholder={t("queryPlaceholder")}
								value={query}
								variant="secondary"
							/>
							<p className="mt-2 text-right font-mono text-[11px] text-[var(--app-faint)] tabular-nums">
								{query.length.toLocaleString(numberLocale)} / 16,000
							</p>
						</Card.Content>
						<Card.Footer className="flex flex-wrap items-center justify-between gap-3 border-t border-[var(--app-line)] bg-[var(--app-surface)] px-4 py-3.5 sm:px-5">
							<p className="text-xs text-[var(--app-muted)]">
								{t("selectedAgents", { count: runnableAgents.length })}
							</p>
							<Button
								className="rounded-lg bg-zinc-900 px-5 text-zinc-50 transition-transform active:scale-[0.98]"
								isDisabled={
									isRunning ||
									query.trim().length === 0 ||
									runnableAgents.length === 0
								}
								type="submit"
								variant="primary"
							>
								<Play aria-hidden="true" className="size-4" />
								{isRunning
									? t("comparingAgents", { count: runnableAgents.length })
									: t("compareAgents", { count: runnableAgents.length })}
							</Button>
						</Card.Footer>
					</form>
				</Card>
			</div>

			{comparisonAgents.length > 0 ? (
				<section className="mt-8" aria-labelledby="comparison-title">
					<h2 className="mb-3 text-sm font-semibold" id="comparison-title">
						{t("comparisonTitle")}
					</h2>
					<div className="grid overflow-hidden rounded-xl border border-[var(--app-line)] bg-[var(--app-surface)] lg:grid-cols-3">
						{comparisonAgents.map((agent) => {
							const runState = runStates[agent];

							return (
								<AgentComparisonCard
									agent={agent}
									errorMessage={
										runState.status === "failed" ? runState.errorMessage : null
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
		</main>
	);
};

export { ComparisonPage };
