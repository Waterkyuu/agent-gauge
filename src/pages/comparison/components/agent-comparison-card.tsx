import { CircleCheck, Clock, TriangleExclamation } from "@gravity-ui/icons";
import { Skeleton } from "@heroui/react";
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
