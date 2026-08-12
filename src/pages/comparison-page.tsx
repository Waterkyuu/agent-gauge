import { MagicWand, Play, Sparkles } from "@gravity-ui/icons";
import { Button, Card, Chip, TextArea } from "@heroui/react";
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
			tone: "bg-black",
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
		<main className="relative overflow-hidden">
			<div className="pointer-events-none absolute inset-x-0 top-0 -z-10 h-[520px] bg-[radial-gradient(circle_at_72%_12%,rgba(0,0,0,0.055),transparent_34%),linear-gradient(to_bottom,#fafafa,white)]" />
			<section className="mx-auto max-w-[1440px] px-5 py-10 sm:px-8 lg:px-10 lg:py-16">
				<header className="mb-12 grid items-end gap-8 lg:grid-cols-[minmax(0,1fr)_280px]">
					<div>
						<Chip className="mb-5" size="sm" variant="soft">
							<Sparkles aria-hidden="true" className="size-3.5" />
							{t("tagline")}
						</Chip>
						<h1 className="max-w-4xl text-5xl font-bold leading-[0.98] tracking-[-0.055em] sm:text-6xl lg:text-7xl">
							{t("title")}
						</h1>
						<p className="mt-6 max-w-2xl text-base leading-7 text-zinc-500 sm:text-lg">
							{t("description")}
						</p>
					</div>
					<Card className="border border-zinc-200 bg-black text-white shadow-[0_18px_50px_rgba(0,0,0,0.14)]">
						<Card.Content className="p-6">
							<div className="flex items-center justify-between">
								<span className="grid size-10 place-items-center rounded-xl bg-white text-black">
									<MagicWand aria-hidden="true" className="size-5" />
								</span>
								<span className="font-mono text-xs text-zinc-500">01 / 03</span>
							</div>
							<p className="mt-8 text-sm font-semibold">
								{t("agentSelection")}
							</p>
							<p className="mt-2 text-xs leading-5 text-zinc-400">
								{t("selectedAgents", { count: runnableAgents.length })}
							</p>
						</Card.Content>
					</Card>
				</header>

				<fieldset aria-label={t("agentSelection")} className="mb-8">
					<div className="mb-4 flex items-center justify-between gap-4">
						<legend className="text-xs font-bold uppercase tracking-[0.16em] text-zinc-500">
							{t("agentSelection")}
						</legend>
						<span className="text-xs font-medium text-zinc-400">
							{t("selectedAgents", { count: runnableAgents.length })}
						</span>
					</div>
					<div className="grid gap-4 md:grid-cols-3">
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

				<Card className="overflow-hidden border border-zinc-200 bg-white shadow-[0_16px_50px_rgba(0,0,0,0.055)]">
					<form onSubmit={onSubmit}>
						<Card.Header className="flex items-center gap-3 border-b border-zinc-100 px-5 py-4 sm:px-6">
							<span className="grid size-9 place-items-center rounded-xl bg-zinc-100 text-black">
								<MagicWand aria-hidden="true" className="size-[18px]" />
							</span>
							<div>
								<p className="text-sm font-bold">{t("queryLabel")}</p>
								<p className="mt-0.5 text-xs text-zinc-400">02 / 03</p>
							</div>
						</Card.Header>
						<Card.Content className="p-4 sm:p-6">
							<label className="sr-only" htmlFor="agent-query">
								{t("queryLabel")}
							</label>
							<TextArea
								className="min-h-36 w-full resize-y rounded-2xl border border-zinc-200 bg-zinc-50 p-4 text-sm leading-6 text-zinc-900 outline-none transition focus:border-black focus:bg-white focus:ring-4 focus:ring-zinc-100"
								disabled={isRunning}
								id="agent-query"
								maxLength={16000}
								onChange={(event) => setQuery(event.target.value)}
								placeholder={t("queryPlaceholder")}
								value={query}
								variant="secondary"
							/>
						</Card.Content>
						<Card.Footer className="flex flex-wrap items-center justify-between gap-3 border-t border-zinc-100 bg-zinc-50/60 px-5 py-4 sm:px-6">
							<p className="text-xs font-medium text-zinc-400">
								{t("selectedAgents", { count: runnableAgents.length })}
							</p>
							<Button
								className="rounded-xl bg-black px-5 text-white shadow-lg shadow-black/10"
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

				{comparisonAgents.length > 0 ? (
					<section className="mt-12" aria-labelledby="comparison-title">
						<h2
							className="mb-5 text-xs font-bold uppercase tracking-[0.16em] text-zinc-500"
							id="comparison-title"
						>
							{t("comparisonTitle")}
						</h2>
						<div className="grid items-start gap-4 xl:grid-cols-3">
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
