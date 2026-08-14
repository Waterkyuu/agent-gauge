import { MagicWand, Play } from "@gravity-ui/icons";
import { Button, Card, TextArea, Toast } from "@heroui/react";
import type { TFunction } from "i18next";
import { type FormEvent, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { checkAgentProcesses } from "@/api/agent";
import {
	checkClaudeLogin,
	onClaudeConfigChanged,
	runClaudeTask,
} from "@/api/claude";
import {
	checkCodexLogin,
	onCodexConfigChanged,
	runCodexTask,
} from "@/api/codex";
import { checkWorkBuddyLogin, runWorkBuddyTask } from "@/api/workbuddy";
import type {
	AgentKind,
	AgentProcessStates,
	AgentRunResult,
	AgentRuntimeStatus,
} from "@/types/agent";
import { getErrorMessage } from "@/utils/error";
import { AgentComparisonCard } from "./components/agent-comparison-card";
import { AgentSelectionCard } from "./components/agent-selection-card";

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
 * Compares the user-visible authentication and runtime configuration fields.
 *
 * @example
 * areRuntimeStatusesEqual(previousStatus, nextStatus);
 */
const areRuntimeStatusesEqual = (
	left: AgentRuntimeStatus,
	right: AgentRuntimeStatus,
) =>
	left.installed === right.installed &&
	left.loggedIn === right.loggedIn &&
	left.authenticationMethod === right.authenticationMethod &&
	left.model === right.model &&
	left.reasoningEffort === right.reasoningEffort;

/**
 * Compares one complete running-process snapshot.
 *
 * @example
 * areProcessStatesEqual(previousProcesses, nextProcesses);
 */
const areProcessStatesEqual = (
	left: AgentProcessStates,
	right: AgentProcessStates,
) =>
	left.claude === right.claude &&
	left.codex === right.codex &&
	left.workbuddy === right.workbuddy;

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
			tone: "bg-mute",
			isReady: false,
		};
	}
	if (loginState.status === "failed") {
		return {
			message: t("loginCheckFailed", { agent: agentName }),
			tone: "bg-hairline-strong",
			isReady: false,
		};
	}
	if (!loginState.value.installed) {
		return {
			message: t("notInstalled", { agent: agentName }),
			tone: "bg-hairline-strong",
			isReady: false,
		};
	}
	if (!loginState.value.loggedIn) {
		return {
			message: t("notLoggedIn", { agent: agentName }),
			tone: "bg-hairline-strong",
			isReady: false,
		};
	}
	if (processState.status === "checking") {
		return {
			message: t("checkingProcess", { agent: agentName }),
			tone: "bg-mute",
			isReady: true,
		};
	}
	if (processState.status === "failed") {
		return {
			message: t("processCheckFailed", { agent: agentName }),
			tone: "bg-hairline-strong",
			isReady: true,
		};
	}
	if (processState.value[agent]) {
		return {
			message: t("agentRunning", { agent: agentName }),
			tone: "bg-primary",
			isReady: true,
		};
	}
	return {
		message: t("agentReady", { agent: agentName }),
		tone: "bg-charcoal",
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
	const loginStatesRef = useRef(loginStates);
	const processStateRef = useRef(processState);
	const activeLoginProbeCountRef = useRef(0);
	const refreshProcessesAfterLoginRef = useRef<() => void>(() => {});
	const isRunning = Object.values(runStates).some(
		(state) => state.status === "running",
	);

	useEffect(() => {
		if (isRunning) {
			return;
		}

		let isActive = true;
		const pendingAgents = new Set<AgentKind>();
		const queuedAgents = new Set<AgentKind>();

		/** Refreshes one login state without overlapping another probe for that Agent. */
		const refreshLoginState = (agent: AgentKind) => {
			if (pendingAgents.has(agent)) {
				return;
			}

			pendingAgents.add(agent);
			activeLoginProbeCountRef.current += 1;
			AGENT_LOGIN_CHECKS[agent]()
				.then((value) => {
					if (isActive) {
						const previous = loginStatesRef.current[agent];
						if (
							previous.status === "resolved" &&
							areRuntimeStatusesEqual(previous.value, value)
						) {
							return;
						}
						const next: LoginStates = {
							...loginStatesRef.current,
							[agent]: { status: "resolved", value },
						};
						loginStatesRef.current = next;
						setLoginStates(next);
					}
				})
				.catch(() => {
					if (isActive && loginStatesRef.current[agent].status !== "failed") {
						const next: LoginStates = {
							...loginStatesRef.current,
							[agent]: { status: "failed" },
						};
						loginStatesRef.current = next;
						setLoginStates(next);
					}
				})
				.finally(() => {
					pendingAgents.delete(agent);
					activeLoginProbeCountRef.current -= 1;
					if (isActive && queuedAgents.delete(agent)) {
						refreshLoginState(agent);
						return;
					}
					if (activeLoginProbeCountRef.current === 0) {
						refreshProcessesAfterLoginRef.current();
					}
				});
		};

		/** Preserves a native change notification that arrives during an active probe. */
		const refreshLoginStateAfterChange = (agent: AgentKind) => {
			if (pendingAgents.has(agent)) {
				queuedAgents.add(agent);
				return;
			}
			refreshLoginState(agent);
		};

		/** Refreshes every login state independently. */
		const refreshLoginStates = () => {
			for (const agent of AGENT_KINDS) {
				refreshLoginState(agent);
			}
		};

		refreshLoginStates();
		const intervalId = window.setInterval(refreshLoginStates, 5000);
		window.addEventListener("focus", refreshLoginStates);
		const stopConfigListeners = Promise.all([
			onCodexConfigChanged(() => {
				refreshLoginStateAfterChange("codex");
			}),
			onClaudeConfigChanged(() => {
				refreshLoginStateAfterChange("claude");
			}),
		]);

		return () => {
			isActive = false;
			window.clearInterval(intervalId);
			window.removeEventListener("focus", refreshLoginStates);
			stopConfigListeners.then((stopListeners) => {
				for (const stopListening of stopListeners) {
					stopListening();
				}
			});
		};
	}, [isRunning]);

	useEffect(() => {
		let isActive = true;
		let isChecking = false;

		/** Refreshes the local process snapshot outside authentication probe windows. */
		const refreshAgentProcesses = async () => {
			if (isChecking || activeLoginProbeCountRef.current > 0) {
				return;
			}
			isChecking = true;
			try {
				const value = await checkAgentProcesses();
				if (isActive) {
					const previous = processStateRef.current;
					if (
						previous.status === "resolved" &&
						areProcessStatesEqual(previous.value, value)
					) {
						return;
					}
					const next: ProcessState = { status: "resolved", value };
					processStateRef.current = next;
					setProcessState(next);
				}
			} catch {
				if (isActive && processStateRef.current.status === "checking") {
					const next: ProcessState = { status: "failed" };
					processStateRef.current = next;
					setProcessState(next);
				}
			} finally {
				isChecking = false;
			}
		};

		refreshProcessesAfterLoginRef.current = refreshAgentProcesses;
		refreshAgentProcesses();
		const intervalId = window.setInterval(refreshAgentProcesses, 1000);

		return () => {
			isActive = false;
			refreshProcessesAfterLoginRef.current = () => {};
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
		const taskLabel =
			normalizedQuery.length > 40
				? `${normalizedQuery.slice(0, 40)}…`
				: normalizedQuery;
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
					Toast.toast.success(
						t("agentRunFinished", {
							task: taskLabel,
							agent: t(`agentNames.${agent}`),
						}),
						{ description: t("viewResult") },
					);
				} catch (error) {
					setRunStates((current) => ({
						...current,
						[agent]: {
							status: "failed",
							errorMessage: getErrorMessage(error, t("requestFailed")),
						},
					}));
					Toast.toast.danger(
						t("agentRunFinished", {
							task: taskLabel,
							agent: t(`agentNames.${agent}`),
						}),
						{ description: t("viewResult") },
					);
				}
			}),
		);
	};

	return (
		<main className="mx-auto max-w-[1320px] px-4 py-8 sm:px-6 lg:px-8 lg:py-10">
			<header className="mb-8 border-b border-hairline pb-7">
				<p className="mb-sm text-body-sm font-medium text-body">
					{t("tagline")}
				</p>
				<h1 className="max-w-3xl font-primary text-display-lg font-medium leading-display-lg sm:text-display-xl sm:leading-display-xl">
					{t("title")}
				</h1>
				<p className="mt-md max-w-[65ch] text-body-sm leading-body-md text-body sm:text-body-md">
					{t("description")}
				</p>
			</header>

			<div className="grid items-start gap-5 xl:grid-cols-[minmax(360px,0.9fr)_minmax(0,1.35fr)]">
				<section className="overflow-hidden rounded-xl border border-hairline bg-surface-card">
					<header className="flex items-center justify-between gap-lg border-b border-hairline px-lg py-md sm:px-xl">
						<h2 className="text-body-sm-strong font-medium">
							{t("agentSelection")}
						</h2>
						<span className="rounded-full bg-surface-soft px-md py-xs font-mono text-caption-sm text-body tabular-nums">
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

				<Card className="overflow-hidden rounded-xl border border-hairline bg-surface-card shadow-none">
					<form onSubmit={onSubmit}>
						<Card.Header className="!flex-row !justify-start gap-md border-b border-hairline px-lg py-md sm:px-xl">
							<span className="grid size-8 place-items-center rounded-lg bg-surface-soft text-body">
								<MagicWand aria-hidden="true" className="size-4" />
							</span>
							<p className="text-body-sm-strong font-medium">
								{t("queryLabel")}
							</p>
						</Card.Header>
						<Card.Content className="p-lg sm:p-xl">
							<label
								className="mb-sm block text-body-sm font-medium text-charcoal"
								htmlFor="agent-query"
							>
								{t("queryLabel")}
							</label>
							<TextArea
								className="min-h-56 w-full resize-y rounded-lg border border-hairline bg-canvas p-lg text-body-sm leading-body-md text-ink outline-none transition-colors placeholder:text-mute focus:border-hairline-strong focus:ring-2 focus:ring-focus-ring"
								disabled={isRunning}
								id="agent-query"
								maxLength={16000}
								onChange={(event) => setQuery(event.target.value)}
								placeholder={t("queryPlaceholder")}
								value={query}
								variant="secondary"
							/>
							<p className="mt-sm text-right font-mono text-caption-sm text-body tabular-nums">
								{query.length.toLocaleString(numberLocale)} / 16,000
							</p>
						</Card.Content>
						<Card.Footer className="flex flex-wrap items-center justify-between gap-md border-t border-hairline bg-surface-soft px-lg py-md sm:px-xl">
							<p className="text-caption-sm text-body">
								{t("selectedAgents", { count: runnableAgents.length })}
							</p>
							<Button
								className="h-9 rounded-full bg-primary px-5 text-button-md font-medium text-on-primary shadow-none transition-transform active:scale-[0.98]"
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
					<h2
						className="mb-md text-body-sm-strong font-semibold"
						id="comparison-title"
					>
						{t("comparisonTitle")}
					</h2>
					<div className="grid overflow-hidden rounded-xl border border-hairline bg-surface-card lg:grid-cols-3">
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

export default ComparisonPage;
