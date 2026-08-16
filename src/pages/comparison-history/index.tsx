import {
	ChevronRight,
	ClockArrowRotateLeft,
	TriangleExclamation,
} from "@gravity-ui/icons";
import { Button, Skeleton } from "@heroui/react";
import { cn } from "cnfast";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AgentLogo } from "@/components/agent-logo";
import { AgentComparisonCard } from "@/pages/comparison/components/agent-comparison-card";
import {
	useComparisonHistory,
	useComparisonHistoryDetail,
} from "@/queries/comparison-history";

/**
 * Renders paginated comparison history and the selected result detail.
 * @example <ComparisonHistoryPage />
 */
const ComparisonHistoryPage = () => {
	const { t, i18n } = useTranslation();
	const [selectedId, setSelectedId] = useState<number | null>(null);
	const historyQuery = useComparisonHistory();
	const detailQuery = useComparisonHistoryDetail(selectedId);
	const historyItems =
		historyQuery.data?.pages.flatMap((page) => page.items) ?? [];
	const numberLocale = i18n.resolvedLanguage ?? "en-US";

	useEffect(() => {
		if (selectedId === null && historyItems[0]) {
			setSelectedId(historyItems[0].id);
		}
	}, [historyItems, selectedId]);

	/**
	 * Selects a history row; React Query resolves its cached or persisted detail.
	 * @example selectComparison(42);
	 */
	const selectComparison = (id: number) => {
		setSelectedId(id);
	};

	return (
		<main className="mx-auto max-w-[1320px] px-4 py-8 sm:px-6 lg:px-8 lg:py-10">
			<header className="mb-8 border-b border-hairline pb-7">
				<p className="mb-sm text-body-sm font-medium text-body">
					{t("comparisonHistory.tagline")}
				</p>
				<h1 className="font-primary text-display-lg font-medium leading-display-lg sm:text-display-xl sm:leading-display-xl">
					{t("comparisonHistory.title")}
				</h1>
				<p className="mt-md max-w-[65ch] text-body-sm leading-body-md text-body sm:text-body-md">
					{t("comparisonHistory.description")}
				</p>
			</header>

			<div className="grid items-start gap-5 xl:grid-cols-[340px_minmax(0,1fr)]">
				<section
					aria-labelledby="history-list-title"
					className="overflow-hidden rounded-xl border border-hairline bg-surface-card"
				>
					<header className="border-b border-hairline px-lg py-md">
						<h2
							className="text-body-sm-strong font-medium"
							id="history-list-title"
						>
							{t("comparisonHistory.listTitle")}
						</h2>
					</header>
					{historyQuery.isPending ? (
						<div
							aria-label={t("comparisonHistory.loading")}
							className="space-y-lg p-lg"
							role="status"
						>
							{[0, 1, 2].map((item) => (
								<div key={item}>
									<Skeleton className="h-4 w-4/5 rounded-full" />
									<Skeleton className="mt-sm h-3 w-1/2 rounded-full" />
								</div>
							))}
						</div>
					) : null}
					{historyQuery.isError ? (
						<div className="p-xl text-center" role="alert">
							<TriangleExclamation
								aria-hidden="true"
								className="mx-auto size-5 text-body"
							/>
							<p className="mt-md text-body-sm text-charcoal">
								{t("comparisonHistory.loadFailed")}
							</p>
						</div>
					) : null}
					{historyQuery.isSuccess && historyItems.length === 0 ? (
						<div className="p-8 text-center">
							<ClockArrowRotateLeft
								aria-hidden="true"
								className="mx-auto size-6 text-mute"
							/>
							<h2 className="mt-md text-body-sm-strong font-medium">
								{t("comparisonHistory.emptyTitle")}
							</h2>
							<p className="mx-auto mt-sm max-w-xs text-caption-sm text-body">
								{t("comparisonHistory.emptyDescription")}
							</p>
						</div>
					) : null}
					{historyItems.length > 0 ? (
						<div>
							{historyItems.map((item) => (
								<button
									aria-pressed={selectedId === item.id}
									className={cn(
										"group block w-full border-b border-hairline px-lg py-md text-left outline-none transition-colors last:border-b-0 hover:bg-surface-soft focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-ring",
										selectedId === item.id && "bg-surface-soft",
									)}
									key={item.id}
									onClick={() => selectComparison(item.id)}
									type="button"
								>
									<span className="flex items-start gap-md">
										<span className="min-w-0 flex-1">
											<span className="line-clamp-2 text-body-sm font-medium leading-body-sm text-ink">
												{item.query}
											</span>
											<span className="mt-sm flex items-center justify-between gap-sm text-caption-sm text-body">
												<span>
													{new Intl.DateTimeFormat(numberLocale, {
														dateStyle: "medium",
														timeStyle: "short",
													}).format(item.createdAtMs)}
												</span>
												<span className="flex -space-x-1">
													{item.agents.map((agent) => (
														<span
															className="rounded-full bg-canvas p-0.5"
															key={agent.agent}
														>
															<AgentLogo
																agent={agent.agent}
																className="size-4"
															/>
														</span>
													))}
												</span>
											</span>
										</span>
										<ChevronRight
											aria-hidden="true"
											className="mt-xs size-4 shrink-0 text-mute group-hover:text-ink"
										/>
									</span>
								</button>
							))}
							{historyQuery.hasNextPage ? (
								<div className="border-t border-hairline p-md">
									<Button
										className="w-full rounded-lg text-body-sm shadow-none"
										isDisabled={historyQuery.isFetchingNextPage}
										onPress={() => historyQuery.fetchNextPage()}
										variant="ghost"
									>
										{historyQuery.isFetchingNextPage
											? t("comparisonHistory.loading")
											: t("comparisonHistory.loadMore")}
									</Button>
								</div>
							) : null}
						</div>
					) : null}
				</section>

				<section aria-live="polite" className="min-w-0">
					{selectedId === null ? (
						<div className="rounded-xl border border-dashed border-hairline p-10 text-center text-body-sm text-body">
							{t("comparisonHistory.selectPrompt")}
						</div>
					) : null}
					{detailQuery.isPending && selectedId !== null ? (
						<div
							aria-label={t("comparisonHistory.loadingDetail")}
							className="rounded-xl border border-hairline p-xl"
							role="status"
						>
							<Skeleton className="h-5 w-2/3 rounded-full" />
							<Skeleton className="mt-md h-3 w-1/3 rounded-full" />
							<Skeleton className="mt-xl h-56 w-full rounded-xl" />
						</div>
					) : null}
					{detailQuery.isError ? (
						<div
							className="rounded-xl border border-terminal-red/30 bg-terminal-red/10 p-xl text-body-sm"
							role="alert"
						>
							{t("comparisonHistory.detailFailed")}
						</div>
					) : null}
					{detailQuery.data ? (
						<div>
							<header className="mb-lg rounded-xl border border-hairline bg-surface-card p-xl">
								<p className="text-caption-sm text-body">
									{new Intl.DateTimeFormat(numberLocale, {
										dateStyle: "long",
										timeStyle: "medium",
									}).format(detailQuery.data.createdAtMs)}
								</p>
								<h2 className="mt-sm whitespace-pre-wrap text-heading-sm font-medium leading-heading-sm">
									{detailQuery.data.query}
								</h2>
							</header>
							<div className="grid overflow-hidden rounded-xl border border-hairline bg-surface-card lg:grid-cols-3">
								{detailQuery.data.results.map((item) => (
									<AgentComparisonCard
										agent={item.agent}
										errorMessage={item.errorMessage}
										isRunning={false}
										key={item.agent}
										model={item.model}
										numberLocale={numberLocale}
										reasoningEffort={item.reasoningEffort}
										result={item.result}
									/>
								))}
							</div>
						</div>
					) : null}
				</section>
			</div>
		</main>
	);
};

export default ComparisonHistoryPage;
