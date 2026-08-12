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
		<Card
			aria-labelledby={titleId}
			className="overflow-hidden border border-zinc-200 bg-white shadow-[0_12px_38px_rgba(0,0,0,0.045)]"
			role="article"
		>
			<Card.Header className="flex items-center justify-between gap-3 border-b border-zinc-100 px-5 py-4">
				<h3 className="text-sm font-bold" id={titleId}>
					{t("comparisonResult", { agent: t(`agentNames.${agent}`) })}
				</h3>
				{isRunning ? (
					<Chip size="sm" variant="soft">
						<Spinner size="sm" />
						{t("agentRunRunning")}
					</Chip>
				) : result ? (
					<CircleCheck aria-hidden="true" className="size-5 text-zinc-500" />
				) : null}
			</Card.Header>

			{errorMessage ? (
				<Card.Content
					className="flex gap-3 p-5 text-sm text-zinc-600"
					role="alert"
				>
					<TriangleExclamation
						aria-hidden="true"
						className="mt-0.5 size-4 shrink-0"
					/>
					<p>{errorMessage}</p>
				</Card.Content>
			) : null}

			{result ? (
				<Card.Content className="p-5">
					<div className="grid grid-cols-3 divide-x divide-zinc-200 rounded-xl border border-zinc-200 bg-zinc-50">
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
							<div className="min-w-0 p-3" key={label}>
								<p className="truncate text-[10px] font-semibold uppercase tracking-[0.08em] text-zinc-400">
									{label}
								</p>
								<p className="mt-2 text-sm font-bold tabular-nums">{value}</p>
							</div>
						))}
					</div>
					{result.tokenUsage ? (
						<p className="mt-3 text-[11px] leading-5 text-zinc-400">
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
					<div className="mt-5 border-t border-zinc-100 pt-4">
						<p className="mb-3 flex items-center gap-2 text-[10px] font-bold uppercase tracking-[0.12em] text-zinc-400">
							<Clock aria-hidden="true" className="size-3.5" />
							{t("responseTitle")}
						</p>
						<pre className="m-0 max-h-80 overflow-auto whitespace-pre-wrap rounded-xl bg-zinc-950 p-4 font-sans text-sm leading-6 text-zinc-200">
							{result.response}
						</pre>
					</div>
				</Card.Content>
			) : null}
		</Card>
	);
};

export { AgentComparisonCard };

import { CircleCheck, Clock, TriangleExclamation } from "@gravity-ui/icons";
import { Card, Chip, Spinner } from "@heroui/react";
