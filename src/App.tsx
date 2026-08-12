import { Button } from "@heroui/react";
import type { TFunction } from "i18next";
import { type FormEvent, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { checkClaudeLogin, runClaudeTask } from "./api/claude";
import { checkCodexLogin, runCodexTask } from "./api/codex";
import { checkWorkBuddyLogin, runWorkBuddyTask } from "./api/workbuddy";
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

/**
 * Formats a measured latency without hiding sub-second precision.
 *
 * @example
 * formatDuration(2450); // "2.45 s"
 */
const formatDuration = (milliseconds: number) => {
	if (milliseconds < 1000) {
		return `${milliseconds} ms`;
	}
	return `${(milliseconds / 1000).toFixed(2)} s`;
};

/**
 * Localizes a known agent reasoning level while retaining its wire value.
 *
 * @example
 * formatReasoningEffort("high", t); // "高 (high)"
 */
const formatReasoningEffort = (effort: string | null, t: TFunction) => {
	if (!effort) {
		return t("metricUnavailable");
	}

	const localized = t(`reasoningEffortLevels.${effort}`, {
		defaultValue: effort,
	});
	return localized ? `${localized} (${effort})` : effort;
};

/**
 * Renders the local agent selector, query composer, and completed task metrics.
 *
 * @example
 * <App />
 */
const App = () => {
	const { t, i18n } = useTranslation();
	const [selectedAgent, setSelectedAgent] = useState<AgentKind>("codex");
	const [loginStates, setLoginStates] = useState<LoginStates>({
		claude: { status: "checking" },
		codex: { status: "checking" },
		workbuddy: { status: "checking" },
	});
	const [query, setQuery] = useState("");
	const [isRunning, setIsRunning] = useState(false);
	const [result, setResult] = useState<AgentRunResult | null>(null);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	useEffect(() => {
		checkClaudeLogin()
			.then((value) =>
				setLoginStates((current) => ({
					...current,
					claude: { status: "resolved", value },
				})),
			)
			.catch(() =>
				setLoginStates((current) => ({
					...current,
					claude: { status: "failed" },
				})),
			);
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
	const agentName = t(`agentNames.${selectedAgent}`);
	const runtimeStatus =
		loginState.status === "resolved" ? loginState.value : null;
	const isReady = runtimeStatus?.installed === true && runtimeStatus.loggedIn;
	const numberLocale = i18n.resolvedLanguage ?? "en-US";

	/**
	 * Changes and persists the active UI language through i18next.
	 *
	 * @example
	 * changeLanguage("zh-CN");
	 */
	const changeLanguage = async (language: "en-US" | "zh-CN") => {
		await i18n.changeLanguage(language);
	};

	/**
	 * Switches the active local agent and clears metrics from the prior product.
	 *
	 * @example
	 * selectAgent("workbuddy");
	 */
	const selectAgent = (agent: AgentKind) => {
		if (isRunning) {
			return;
		}
		setSelectedAgent(agent);
		setResult(null);
		setErrorMessage(null);
	};

	/**
	 * Submits the current query once and replaces the prior result.
	 *
	 * @example
	 * onSubmit(event);
	 */
	const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
		event.preventDefault();
		if (!isReady || isRunning || query.trim().length === 0) {
			return;
		}

		setIsRunning(true);
		setErrorMessage(null);
		setResult(null);
		try {
			if (selectedAgent === "claude") {
				setResult(await runClaudeTask(query.trim()));
			} else if (selectedAgent === "codex") {
				setResult(await runCodexTask(query.trim()));
			} else {
				setResult(await runWorkBuddyTask(query.trim()));
			}
		} catch (error) {
			setErrorMessage(getErrorMessage(error, t("requestFailed")));
		} finally {
			setIsRunning(false);
		}
	};

	let loginMessage: string = t("checkingLogin", { agent: agentName });
	let loginTone: string = "bg-amber-400";
	if (loginState.status === "failed") {
		loginMessage = t("loginCheckFailed", { agent: agentName });
		loginTone = "bg-rose-400";
	} else if (loginState.status === "resolved") {
		if (!loginState.value.installed) {
			loginMessage = t("notInstalled", { agent: agentName });
			loginTone = "bg-rose-400";
		} else if (!loginState.value.loggedIn) {
			loginMessage = t("notLoggedIn", { agent: agentName });
			loginTone = "bg-rose-400";
		} else {
			const authenticationMethod =
				selectedAgent === "workbuddy"
					? null
					: loginState.value.authenticationMethod;
			loginMessage = authenticationMethod
				? t("loggedInWithMethod", {
						agent: agentName,
						method: authenticationMethod,
					})
				: t("loggedIn", { agent: agentName });
			loginTone = "bg-emerald-400";
		}
	}

	return (
		<main className="min-h-screen bg-slate-950 px-5 py-10 text-white sm:px-8">
			<section className="mx-auto max-w-4xl">
				<header className="mb-8">
					<div className="mb-5 flex items-center justify-between gap-4">
						<p className="font-semibold tracking-tight">{t("appName")}</p>
						<div className="flex items-center gap-3">
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
											language === "zh-CN"
												? "languages.zhCN"
												: "languages.enUS",
										)}
									</button>
								))}
							</fieldset>
							<div className="flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-2 text-xs text-slate-300">
								<span className={`size-2 rounded-full ${loginTone}`} />
								{loginMessage}
							</div>
						</div>
					</div>
					<fieldset
						aria-label={t("agentSelection")}
						className="mb-6 inline-flex gap-1 rounded-xl border border-white/10 bg-white/5 p-1"
					>
						{(["codex", "claude", "workbuddy"] as const).map((agent) => (
							<button
								aria-pressed={selectedAgent === agent}
								className="rounded-lg px-4 py-2 text-sm text-slate-400 transition hover:text-white aria-pressed:bg-indigo-500 aria-pressed:text-white"
								disabled={isRunning}
								key={agent}
								onClick={() => selectAgent(agent)}
								type="button"
							>
								{t(`agentNames.${agent}`)}
							</button>
						))}
					</fieldset>
					<p className="mb-2 text-sm font-medium text-indigo-300">
						{t("tagline")}
					</p>
					<h1 className="max-w-3xl text-4xl font-semibold tracking-tight sm:text-5xl">
						{t("title")}
					</h1>
					<p className="mt-4 max-w-2xl leading-7 text-slate-400">
						{t("description")}
					</p>
					{isReady ? (
						<dl
							aria-label={t("runtimeConfiguration", { agent: agentName })}
							className="mt-5 grid gap-3 sm:grid-cols-2"
						>
							<div className="rounded-xl border border-white/10 bg-white/5 px-4 py-3">
								<dt className="text-xs text-slate-500">{t("currentModel")}</dt>
								<dd className="mt-1 font-medium text-slate-100">
									{runtimeStatus?.model ?? t("metricUnavailable")}
								</dd>
							</div>
							<div className="rounded-xl border border-white/10 bg-white/5 px-4 py-3">
								<dt className="text-xs text-slate-500">
									{t("reasoningEffort")}
								</dt>
								<dd className="mt-1 font-medium text-slate-100">
									{formatReasoningEffort(
										runtimeStatus?.reasoningEffort ?? null,
										t,
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
						{t("queryLabel")}
					</label>
					<textarea
						className="min-h-32 w-full resize-y rounded-xl border border-white/10 bg-slate-950/80 p-4 text-sm leading-6 text-slate-100 outline-none transition focus:border-indigo-400"
						disabled={!isReady || isRunning}
						id="agent-query"
						maxLength={16000}
						onChange={(event) => setQuery(event.target.value)}
						placeholder={t("queryPlaceholder")}
						value={query}
					/>
					<div className="mt-4 flex items-center justify-end">
						<Button
							isDisabled={!isReady || isRunning || query.trim().length === 0}
							type="submit"
							variant="primary"
						>
							{isRunning
								? t("running", { agent: agentName })
								: t("send", { agent: agentName })}
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
							{t("metricsTitle")}
						</h2>
						<div className="grid gap-3 sm:grid-cols-3">
							{[
								[
									t("firstToken"),
									result.timeToFirstTokenMs === null
										? t("metricUnavailable")
										: formatDuration(result.timeToFirstTokenMs),
								],
								[t("totalDuration"), formatDuration(result.totalDurationMs)],
								[
									t("totalTokens"),
									result.tokenUsage?.totalTokens.toLocaleString(numberLocale) ??
										t("metricUnavailable"),
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
								{t("inputTokens")}{" "}
								{result.tokenUsage.inputTokens.toLocaleString(numberLocale)} ·{" "}
								{t("outputTokens")}{" "}
								{result.tokenUsage.outputTokens.toLocaleString(numberLocale)} ·{" "}
								{t("reasoningTokens")}{" "}
								{result.tokenUsage.reasoningOutputTokens?.toLocaleString(
									numberLocale,
								) ?? t("metricUnavailable")}
							</p>
						) : null}
						<div className="mt-5 rounded-2xl border border-white/10 bg-white/5 p-5">
							<h2 className="mb-3 text-sm font-medium text-slate-300">
								{t("responseTitle", { agent: agentName })}
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
