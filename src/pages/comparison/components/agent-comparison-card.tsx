import { CircleCheck, Clock, TriangleExclamation } from "@gravity-ui/icons";
import { Skeleton } from "@heroui/react";
import { cn } from "cnfast";
import { useTranslation } from "react-i18next";
import { AgentLogo } from "@/components/agent-logo";
import type { AgentKind, AgentRunResult } from "@/types/agent";

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
	/** Model configuration captured for a historical result. */
	model?: string | null;
	/** Reasoning configuration captured for a historical result. */
	reasoningEffort?: string | null;
};

const TOOL_DURATION_WARNING_MS = 20_000;
const TOOL_DURATION_CRITICAL_MS = 60_000;

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
 * Highlights long tool calls using duration-only severity tones.
 *
 * @example
 * getToolDurationTone(21_000); // "bg-terminal-yellow/20 text-ink"
 */
const getToolDurationTone = (milliseconds: number) => {
	if (milliseconds > TOOL_DURATION_CRITICAL_MS) {
		return "bg-terminal-red/15 text-ink";
	}
	if (milliseconds > TOOL_DURATION_WARNING_MS) {
		return "bg-terminal-yellow/20 text-ink";
	}
	return "text-body";
};

/**
 * Renders one product result as a column in the shared comparison surface.
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
	model = null,
	reasoningEffort = null,
}: AgentComparisonCardProps) => {
	const { t } = useTranslation();
	const titleId = `comparison-${agent}-title`;

	return (
		<article
			aria-labelledby={titleId}
			className="min-w-0 border-b border-hairline p-xl last:border-b-0 lg:border-b-0 lg:border-r lg:last:border-r-0"
		>
			<header className="flex min-h-7 items-center justify-between gap-3">
				<div className="flex min-w-0 items-center gap-2.5">
					<AgentLogo agent={agent} className="size-5" />
					<h3 className="truncate text-body-sm-strong font-medium" id={titleId}>
						{t("comparisonResult", { agent: t(`agentNames.${agent}`) })}
					</h3>
				</div>
				{isRunning ? (
					<span className="flex items-center gap-sm text-caption-sm text-body">
						<Clock aria-hidden="true" className="size-3.5" />
						{t("agentRunRunning")}
					</span>
				) : result ? (
					<CircleCheck
						aria-hidden="true"
						className="size-[18px] text-terminal-green"
					/>
				) : null}
			</header>
			{model || reasoningEffort ? (
				<dl className="mt-md grid grid-cols-2 gap-md rounded-lg bg-surface-soft px-md py-sm text-caption-sm">
					<div className="min-w-0">
						<dt className="text-mute">{t("currentModel")}</dt>
						<dd className="mt-xs truncate font-mono text-ink">
							{model ?? t("metricUnavailable")}
						</dd>
					</div>
					<div className="min-w-0">
						<dt className="text-mute">{t("reasoningEffort")}</dt>
						<dd className="mt-xs truncate font-mono text-ink">
							{reasoningEffort ?? t("metricUnavailable")}
						</dd>
					</div>
				</dl>
			) : null}

			{isRunning ? (
				<div className="mt-5" aria-label={t("agentRunRunning")} role="status">
					<div className="grid grid-cols-3 gap-lg border-y border-hairline py-lg">
						{[0, 1, 2].map((item) => (
							<div key={item}>
								<Skeleton className="h-2 w-14 rounded-full" />
								<Skeleton className="mt-md h-4 w-12 rounded-full" />
							</div>
						))}
					</div>
					<div className="grid grid-cols-2 gap-lg border-b border-hairline py-lg">
						{[0, 1].map((item) => (
							<div key={item}>
								<Skeleton className="h-2 w-14 rounded-full" />
								<Skeleton className="mt-md h-4 w-12 rounded-full" />
							</div>
						))}
					</div>
					<div className="mt-xl space-y-sm">
						<Skeleton className="h-3 w-full rounded-full" />
						<Skeleton className="h-3 w-5/6 rounded-full" />
						<Skeleton className="h-3 w-2/3 rounded-full" />
					</div>
				</div>
			) : null}

			{errorMessage ? (
				<div
					className="mt-xl flex gap-md rounded-lg border border-terminal-red/30 bg-terminal-red/10 p-lg text-body-sm text-charcoal"
					role="alert"
				>
					<TriangleExclamation
						aria-hidden="true"
						className="mt-0.5 size-4 shrink-0"
					/>
					<p>{errorMessage}</p>
				</div>
			) : null}

			{result ? (
				<div className="mt-xl">
					<div className="grid grid-cols-3 gap-lg border-y border-hairline py-lg">
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
							<div className="min-w-0" key={label}>
								<p className="truncate text-caption-sm text-mute">{label}</p>
								<p className="mt-sm font-mono text-body-sm font-medium tabular-nums">
									{value}
								</p>
							</div>
						))}
					</div>
					{result.tokenUsage ? (
						<div className="mt-lg grid grid-cols-3 gap-md text-caption-sm">
							{[
								[t("inputTokens"), result.tokenUsage.inputTokens],
								[t("outputTokens"), result.tokenUsage.outputTokens],
								[t("reasoningTokens"), result.tokenUsage.reasoningOutputTokens],
							].map(([label, value]) => (
								<div key={label}>
									<p className="text-mute">{label}</p>
									<p className="mt-xs font-mono font-medium text-body tabular-nums">
										{typeof value === "number"
											? value.toLocaleString(numberLocale)
											: t("metricUnavailable")}
									</p>
								</div>
							))}
						</div>
					) : null}
					<dl className="mt-lg grid grid-cols-2 gap-lg border-t border-hairline pt-lg">
						<div>
							<dt className="text-caption-sm text-mute">
								{t("thinkingDuration")}
							</dt>
							<dd className="mt-xs font-mono text-body-sm font-medium tabular-nums">
								{formatDuration(result.thinkingDurationMs)}
							</dd>
						</div>
						<div>
							<dt className="text-caption-sm text-mute">
								{t("toolCallCount")}
							</dt>
							<dd className="mt-xs font-mono text-body-sm font-medium tabular-nums">
								{result.toolCallCount.toLocaleString(numberLocale)}
							</dd>
						</div>
					</dl>
					<section className="mt-xl" aria-labelledby={`${titleId}-tools`}>
						<h4
							className="mb-sm text-caption-sm font-medium text-charcoal"
							id={`${titleId}-tools`}
						>
							{t("toolCallsTitle")}
						</h4>
						{result.toolCalls.length > 0 ? (
							<ol className="overflow-hidden rounded-lg border border-hairline bg-canvas">
								{result.toolCalls.map((toolCall) => {
									const duration = formatDuration(toolCall.durationMs);
									return (
										<li
											className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-md border-b border-hairline px-md py-sm last:border-b-0"
											key={toolCall.sequence}
										>
											<span className="truncate font-mono text-caption-sm font-medium">
												{toolCall.name}
											</span>
											<span
												className={cn(
													"rounded-full px-sm py-xxs font-mono text-caption-sm font-medium tabular-nums",
													getToolDurationTone(toolCall.durationMs),
												)}
											>
												{duration}
											</span>
										</li>
									);
								})}
							</ol>
						) : (
							<p className="rounded-lg border border-dashed border-hairline px-md py-md text-caption-sm text-mute">
								{t("noToolCalls")}
							</p>
						)}
					</section>
					<div className="mt-xl">
						<p className="mb-sm text-caption-sm font-medium text-charcoal">
							{t("responseTitle")}
						</p>
						<pre className="m-0 max-h-80 overflow-auto whitespace-pre-wrap rounded-lg border border-hairline bg-surface-soft p-lg font-mono text-caption-sm leading-code-sm text-ink">
							{result.response}
						</pre>
					</div>
				</div>
			) : null}
		</article>
	);
};

export { AgentComparisonCard };
