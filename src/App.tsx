import { Button } from "@heroui/react";
import { type FormEvent, useEffect, useState } from "react";
import { checkCodexLogin, runCodexTask } from "./api/codex";
import { checkWorkBuddyLogin, runWorkBuddyTask } from "./api/workbuddy";
import { ZH_CN } from "./i18n/zh-CN";
import type {
	AgentKind,
	AgentRunResult,
	AgentRuntimeStatus,
} from "./types/agent";
import { getErrorMessage } from "./utils/error";

type LoginState =
	| { status: "checking" }
	| { status: "resolved"; value: AgentRuntimeStatus }
	| { status: "failed" };

type LoginStates = Record<AgentKind, LoginState>;

/** Formats a measured latency without hiding sub-second precision. Example: `formatDuration(2450)` returns `2.45 s`. */
const formatDuration = (milliseconds: number) => {
	if (milliseconds < 1000) {
		return `${milliseconds} ms`;
	}
	return `${(milliseconds / 1000).toFixed(2)} s`;
};

/** Localizes a known agent reasoning level while retaining its wire value. Example: `formatReasoningEffort("high")` returns `高 (high)`. */
const formatReasoningEffort = (effort: string | null) => {
	if (!effort) {
		return ZH_CN.metricUnavailable;
	}

	const localized =
		ZH_CN.reasoningEffortLevels[
			effort as keyof typeof ZH_CN.reasoningEffortLevels
		];
	return localized ? `${localized} (${effort})` : effort;
};

/** Renders the local agent selector, query composer, and completed task metrics. Example: `<App />`. */
const App = () => {
	const [selectedAgent, setSelectedAgent] = useState<AgentKind>("codex");
	const [loginStates, setLoginStates] = useState<LoginStates>({
		codex: { status: "checking" },
		workbuddy: { status: "checking" },
	});
	const [query, setQuery] = useState("");
	const [isRunning, setIsRunning] = useState(false);
	const [result, setResult] = useState<AgentRunResult | null>(null);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	useEffect(() => {
		checkCodexLogin()
			.then((value) =>
				setLoginStates((current) => ({
					...current,
					codex: { status: "resolved", value },
				})),
			)
			.catch(() =>
				setLoginStates((current) => ({
					...current,
					codex: { status: "failed" },
				})),
			);
		checkWorkBuddyLogin()
			.then((value) =>
				setLoginStates((current) => ({
					...current,
					workbuddy: { status: "resolved", value },
				})),
			)
			.catch(() =>
				setLoginStates((current) => ({
					...current,
					workbuddy: { status: "failed" },
				})),
			);
	}, []);

	const loginState = loginStates[selectedAgent];
	const agentName = ZH_CN.agentNames[selectedAgent];
	const runtimeStatus =
		loginState.status === "resolved" ? loginState.value : null;
	const isReady = runtimeStatus?.installed === true && runtimeStatus.loggedIn;

	/** Switches the active local agent and clears metrics from the prior product. Example: `selectAgent("workbuddy")`. */
	const selectAgent = (agent: AgentKind) => {
		if (isRunning) {
			return;
		}
		setSelectedAgent(agent);
		setResult(null);
		setErrorMessage(null);
	};

	/** Submits the current query once and replaces the prior result. Example: `onSubmit(event)`. */
	const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
		event.preventDefault();
		if (!isReady || isRunning || query.trim().length === 0) {
			return;
		}

		setIsRunning(true);
		setErrorMessage(null);
		setResult(null);
		try {
			setResult(
				await (selectedAgent === "codex"
					? runCodexTask(query.trim())
					: runWorkBuddyTask(query.trim())),
			);
		} catch (error) {
			setErrorMessage(getErrorMessage(error, ZH_CN.requestFailed));
		} finally {
			setIsRunning(false);
		}
	};

	let loginMessage: string = ZH_CN.checkingLogin(agentName);
	let loginTone: string = "bg-amber-400";
	if (loginState.status === "failed") {
		loginMessage = ZH_CN.loginCheckFailed(agentName);
		loginTone = "bg-rose-400";
	} else if (loginState.status === "resolved") {
		if (!loginState.value.installed) {
			loginMessage = ZH_CN.notInstalled(agentName);
			loginTone = "bg-rose-400";
		} else if (!loginState.value.loggedIn) {
			loginMessage = ZH_CN.notLoggedIn(agentName);
			loginTone = "bg-rose-400";
		} else {
			loginMessage = ZH_CN.loggedIn(
				agentName,
				selectedAgent === "workbuddy"
					? null
					: loginState.value.authenticationMethod,
			);
			loginTone = "bg-emerald-400";
		}
	}

	return (
		<main className="min-h-screen bg-slate-950 px-5 py-10 text-white sm:px-8">
			<section className="mx-auto max-w-4xl">
				<header className="mb-8">
					<div className="mb-5 flex items-center justify-between gap-4">
						<p className="font-semibold tracking-tight">{ZH_CN.appName}</p>
						<div className="flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-2 text-xs text-slate-300">
							<span className={`size-2 rounded-full ${loginTone}`} />
							{loginMessage}
						</div>
					</div>
					<fieldset
						aria-label={ZH_CN.agentSelection}
						className="mb-6 inline-flex gap-1 rounded-xl border border-white/10 bg-white/5 p-1"
					>
						{(["codex", "workbuddy"] as const).map((agent) => (
							<button
								aria-pressed={selectedAgent === agent}
								className="rounded-lg px-4 py-2 text-sm text-slate-400 transition hover:text-white aria-pressed:bg-indigo-500 aria-pressed:text-white"
								disabled={isRunning}
								key={agent}
								onClick={() => selectAgent(agent)}
								type="button"
							>
								{ZH_CN.agentNames[agent]}
							</button>
						))}
					</fieldset>
					<p className="mb-2 text-sm font-medium text-indigo-300">
						{ZH_CN.tagline}
					</p>
					<h1 className="max-w-3xl text-4xl font-semibold tracking-tight sm:text-5xl">
						{ZH_CN.title}
					</h1>
					<p className="mt-4 max-w-2xl leading-7 text-slate-400">
						{ZH_CN.description}
					</p>
					{isReady ? (
						<dl
							aria-label={ZH_CN.runtimeConfiguration(agentName)}
							className="mt-5 grid gap-3 sm:grid-cols-2"
						>
							<div className="rounded-xl border border-white/10 bg-white/5 px-4 py-3">
								<dt className="text-xs text-slate-500">{ZH_CN.currentModel}</dt>
								<dd className="mt-1 font-medium text-slate-100">
									{runtimeStatus?.model ?? ZH_CN.metricUnavailable}
								</dd>
							</div>
							<div className="rounded-xl border border-white/10 bg-white/5 px-4 py-3">
								<dt className="text-xs text-slate-500">
									{ZH_CN.reasoningEffort}
								</dt>
								<dd className="mt-1 font-medium text-slate-100">
									{formatReasoningEffort(
										runtimeStatus?.reasoningEffort ?? null,
									)}
								</dd>
							</div>
						</dl>
					) : null}
				</header>

				<form
					className="rounded-2xl border border-white/10 bg-white/5 p-5"
					onSubmit={onSubmit}
				>
					<label
						className="mb-2 block text-sm font-medium"
						htmlFor="agent-query"
					>
						{ZH_CN.queryLabel}
					</label>
					<textarea
						className="min-h-32 w-full resize-y rounded-xl border border-white/10 bg-slate-950/80 p-4 text-sm leading-6 text-slate-100 outline-none transition focus:border-indigo-400"
						disabled={!isReady || isRunning}
						id="agent-query"
						maxLength={16000}
						onChange={(event) => setQuery(event.target.value)}
						placeholder={ZH_CN.queryPlaceholder}
						value={query}
					/>
					<div className="mt-4 flex items-center justify-end">
						<Button
							isDisabled={!isReady || isRunning || query.trim().length === 0}
							type="submit"
							variant="primary"
						>
							{isRunning ? ZH_CN.running(agentName) : ZH_CN.send(agentName)}
						</Button>
					</div>
					{errorMessage ? (
						<p className="mt-4 text-sm text-rose-300" role="alert">
							{errorMessage}
						</p>
					) : null}
				</form>

				{result ? (
					<section className="mt-6" aria-labelledby="metrics-title">
						<h2
							className="mb-3 text-sm font-medium text-slate-300"
							id="metrics-title"
						>
							{ZH_CN.metricsTitle}
						</h2>
						<div className="grid gap-3 sm:grid-cols-3">
							{[
								[
									ZH_CN.firstToken,
									result.timeToFirstTokenMs === null
										? ZH_CN.metricUnavailable
										: formatDuration(result.timeToFirstTokenMs),
								],
								[ZH_CN.totalDuration, formatDuration(result.totalDurationMs)],
								[
									ZH_CN.totalTokens,
									result.tokenUsage?.totalTokens.toLocaleString("zh-CN") ??
										ZH_CN.metricUnavailable,
								],
							].map(([label, value]) => (
								<div
									className="rounded-xl border border-white/10 bg-white/5 p-4"
									key={label}
								>
									<p className="text-xs text-slate-500">{label}</p>
									<p className="mt-2 text-2xl font-semibold tabular-nums">
										{value}
									</p>
								</div>
							))}
						</div>
						{result.tokenUsage ? (
							<p className="mt-3 text-xs text-slate-500">
								{ZH_CN.inputTokens}{" "}
								{result.tokenUsage.inputTokens.toLocaleString("zh-CN")} ·{" "}
								{ZH_CN.outputTokens}{" "}
								{result.tokenUsage.outputTokens.toLocaleString("zh-CN")} ·{" "}
								{ZH_CN.reasoningTokens}{" "}
								{result.tokenUsage.reasoningOutputTokens?.toLocaleString(
									"zh-CN",
								) ?? ZH_CN.metricUnavailable}
							</p>
						) : null}
						<div className="mt-5 rounded-2xl border border-white/10 bg-white/5 p-5">
							<h2 className="mb-3 text-sm font-medium text-slate-300">
								{ZH_CN.responseTitle(agentName)}
							</h2>
							<pre className="m-0 whitespace-pre-wrap font-sans text-sm leading-7 text-slate-200">
								{result.response}
							</pre>
						</div>
					</section>
				) : null}
			</section>
		</main>
	);
};

export default App;
