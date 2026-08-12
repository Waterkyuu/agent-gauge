import { useTranslation } from "react-i18next";
import type { AgentKind, AgentRunResult } from "../types/agent";

type AgentComparisonCardProps = {
	/** Product represented by this comparison result. */
	agent: AgentKind;
	/** Completed metrics and response, or null while running or failed. */
	result: AgentRunResult | null;
	/** Safe failure detail scoped to this product. */
	errorMessage: string | null;
	/** Whether this product is still executing its task. */
	isRunning: boolean;
	/** Locale used for token number formatting. */
	numberLocale: string;
};

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
 * Renders the metrics and response for one product in a comparison run.
 *
 * @example
 * <AgentComparisonCard agent="codex" result={result} errorMessage={null} isRunning={false} numberLocale="en-US" />
 */
const AgentComparisonCard = ({
	agent,
	result,
	errorMessage,
	isRunning,
	numberLocale,
}: AgentComparisonCardProps) => {
	const { t } = useTranslation();
	const titleId = `comparison-${agent}-title`;

	return (
		<article
			aria-labelledby={titleId}
			className="rounded-2xl border border-white/10 bg-white/5 p-5"
		>
			<div className="flex items-center justify-between gap-3">
				<h3 className="font-semibold" id={titleId}>
					{t("comparisonResult", { agent: t(`agentNames.${agent}`) })}
				</h3>
				{isRunning ? (
					<span className="text-xs text-indigo-300">
						{t("agentRunRunning")}
					</span>
				) : null}
			</div>

			{errorMessage ? (
				<p className="mt-5 text-sm text-rose-300" role="alert">
					{errorMessage}
				</p>
			) : null}

			{result ? (
				<>
					<div className="mt-5 grid grid-cols-3 gap-2">
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
							<div className="rounded-xl bg-slate-950/60 p-3" key={label}>
								<p className="text-[11px] text-slate-500">{label}</p>
								<p className="mt-2 font-semibold tabular-nums">{value}</p>
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
					<div className="mt-5 border-t border-white/10 pt-4">
						<p className="mb-2 text-xs font-medium text-slate-400">
							{t("responseTitle")}
						</p>
						<pre className="m-0 max-h-80 overflow-auto whitespace-pre-wrap font-sans text-sm leading-6 text-slate-200">
							{result.response}
						</pre>
					</div>
				</>
			) : null}
		</article>
	);
};

export { AgentComparisonCard };
