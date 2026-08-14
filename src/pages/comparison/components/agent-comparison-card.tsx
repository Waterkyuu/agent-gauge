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
 * getToolDurationTone(21_000); // "bg-red-50 text-red-700"
 */
const getToolDurationTone = (milliseconds: number) => {
	if (milliseconds > TOOL_DURATION_CRITICAL_MS) {
		return "bg-red-100 text-red-900";
	}
	if (milliseconds > TOOL_DURATION_WARNING_MS) {
		return "bg-red-50 text-red-700";
	}
	return "text-[var(--app-muted)]";
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
}: AgentComparisonCardProps) => {
	const { t } = useTranslation();
	const titleId = `comparison-${agent}-title`;

	return (
		<article
			aria-labelledby={titleId}
			className="min-w-0 border-b border-[var(--app-line)] p-5 last:border-b-0 lg:border-b-0 lg:border-r lg:last:border-r-0"
		>
			<header className="flex min-h-7 items-center justify-between gap-3">
				<div className="flex min-w-0 items-center gap-2.5">
					<AgentLogo agent={agent} className="size-5" />
					<h3 className="truncate text-sm font-semibold" id={titleId}>
						{t("comparisonResult", { agent: t(`agentNames.${agent}`) })}
					</h3>
				</div>
				{isRunning ? (
					<span className="flex items-center gap-1.5 text-xs text-[var(--app-muted)]">
						<Clock aria-hidden="true" className="size-3.5" />
						{t("agentRunRunning")}
					</span>
				) : result ? (
					<CircleCheck
						aria-hidden="true"
						className="size-[18px] text-[var(--app-muted)]"
					/>
				) : null}
			</header>

			{isRunning ? (
				<div className="mt-5" aria-label={t("agentRunRunning")} role="status">
					<div className="grid grid-cols-3 gap-4 border-y border-[var(--app-line)] py-4">
						{[0, 1, 2].map((item) => (
							<div key={item}>
								<Skeleton className="h-2 w-14 rounded" />
								<Skeleton className="mt-3 h-4 w-12 rounded" />
							</div>
						))}
					</div>
					<div className="grid grid-cols-2 gap-4 border-b border-[var(--app-line)] py-4">
						{[0, 1].map((item) => (
							<div key={item}>
								<Skeleton className="h-2 w-14 rounded" />
								<Skeleton className="mt-3 h-4 w-12 rounded" />
							</div>
						))}
					</div>
					<div className="mt-5 space-y-2">
						<Skeleton className="h-3 w-full rounded" />
						<Skeleton className="h-3 w-5/6 rounded" />
						<Skeleton className="h-3 w-2/3 rounded" />
					</div>
				</div>
			) : null}

			{errorMessage ? (
				<div
					className="mt-5 flex gap-3 rounded-lg border border-[var(--app-line)] bg-[var(--app-hover)] p-4 text-sm text-[var(--app-muted)]"
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
				<div className="mt-5">
					<div className="grid grid-cols-3 gap-4 border-y border-[var(--app-line)] py-4">
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
								<p className="truncate text-[11px] text-[var(--app-faint)]">
									{label}
								</p>
								<p className="mt-1.5 font-mono text-sm font-semibold tabular-nums">
									{value}
								</p>
							</div>
						))}
					</div>
					{result.tokenUsage ? (
						<div className="mt-4 grid grid-cols-3 gap-3 text-[11px]">
							{[
								[t("inputTokens"), result.tokenUsage.inputTokens],
								[t("outputTokens"), result.tokenUsage.outputTokens],
								[t("reasoningTokens"), result.tokenUsage.reasoningOutputTokens],
							].map(([label, value]) => (
								<div key={label}>
									<p className="text-[var(--app-faint)]">{label}</p>
									<p className="mt-1 font-mono font-medium text-[var(--app-muted)] tabular-nums">
										{typeof value === "number"
											? value.toLocaleString(numberLocale)
											: t("metricUnavailable")}
									</p>
								</div>
							))}
						</div>
					) : null}
					<dl className="mt-4 grid grid-cols-2 gap-4 border-t border-[var(--app-line)] pt-4">
						<div>
							<dt className="text-[11px] text-[var(--app-faint)]">
								{t("thinkingDuration")}
							</dt>
							<dd className="mt-1 font-mono text-sm font-semibold tabular-nums">
								{formatDuration(result.thinkingDurationMs)}
							</dd>
						</div>
						<div>
							<dt className="text-[11px] text-[var(--app-faint)]">
								{t("toolCallCount")}
							</dt>
							<dd className="mt-1 font-mono text-sm font-semibold tabular-nums">
								{result.toolCallCount.toLocaleString(numberLocale)}
							</dd>
						</div>
					</dl>
					<section className="mt-5" aria-labelledby={`${titleId}-tools`}>
						<h4
							className="mb-2 text-xs font-medium text-[var(--app-muted)]"
							id={`${titleId}-tools`}
						>
							{t("toolCallsTitle")}
						</h4>
						{result.toolCalls.length > 0 ? (
							<ol className="overflow-hidden rounded-lg border border-[var(--app-line)] bg-[var(--app-raised)]">
								{result.toolCalls.map((toolCall) => {
									const duration = formatDuration(toolCall.durationMs);
									return (
										<li
											className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3 border-b border-[var(--app-line)] px-3 py-2.5 last:border-b-0"
											key={toolCall.sequence}
										>
											<span className="truncate font-mono text-xs font-medium">
												{toolCall.name}
											</span>
											<span
												className={cn(
													"rounded px-1.5 py-0.5 font-mono text-[11px] font-semibold tabular-nums",
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
							<p className="rounded-lg border border-dashed border-[var(--app-line)] px-3 py-3 text-xs text-[var(--app-faint)]">
								{t("noToolCalls")}
							</p>
						)}
					</section>
					<div className="mt-5">
						<p className="mb-2 text-xs font-medium text-[var(--app-muted)]">
							{t("responseTitle")}
						</p>
						<pre className="m-0 max-h-80 overflow-auto whitespace-pre-wrap rounded-lg border border-[var(--app-line)] bg-[var(--app-canvas)] p-4 font-mono text-xs leading-6 text-[var(--app-ink)]">
							{result.response}
						</pre>
					</div>
				</div>
			) : null}
		</article>
	);
};

export { AgentComparisonCard };
