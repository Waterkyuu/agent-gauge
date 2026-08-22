import { ArrowLeft } from "@gravity-ui/icons";
import { Button, Skeleton } from "@heroui/react";
import { useTranslation } from "react-i18next";
import { AgentComparisonCard } from "@/pages/comparison/components/agent-comparison-card";
import type { ComparisonHistoryDetail } from "@/types/comparison";

type HistoryDetailProps = {
	/** Loaded comparison detail, when available. */
	data?: ComparisonHistoryDetail;
	/** Whether the detail request failed. */
	isError: boolean;
	/** Whether the detail request is loading. */
	isPending: boolean;
	/** Locale used to format dates and metric numbers. */
	numberLocale: string;
	/** Returns to the comparison history list. */
	onBack: () => void;
};

/**
 * Renders one routed comparison detail without the history page heading.
 * @example <HistoryDetail data={detail} numberLocale="zh-CN" onBack={goBack} />
 */
const HistoryDetail = ({
	data,
	isError,
	isPending,
	numberLocale,
	onBack,
}: HistoryDetailProps) => {
	const { t } = useTranslation();

	return (
		<section aria-live="polite" className="min-w-0">
			<Button
				aria-label={t("comparisonHistory.backToList")}
				className="mb-lg min-w-0 cursor-pointer rounded-full border border-hairline bg-canvas p-sm text-ink shadow-none hover:bg-surface-soft"
				isIconOnly
				onPress={onBack}
				variant="ghost"
			>
				<ArrowLeft aria-hidden="true" className="size-4" />
			</Button>
			{isPending ? (
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
			{isError ? (
				<div
					className="rounded-xl border border-terminal-red/30 bg-terminal-red/10 p-xl text-body-sm"
					role="alert"
				>
					{t("comparisonHistory.detailFailed")}
				</div>
			) : null}
			{data ? (
				<div>
					<header className="mb-lg border-b border-hairline pb-lg">
						<p className="text-caption-sm text-body">
							{new Intl.DateTimeFormat(numberLocale, {
								dateStyle: "long",
								timeStyle: "medium",
							}).format(data.createdAtMs)}
						</p>
						<h2 className="mt-sm whitespace-pre-wrap text-heading-sm font-medium leading-heading-sm">
							{data.query}
						</h2>
					</header>
					<div className="grid overflow-hidden rounded-xl border border-hairline bg-surface-card lg:grid-cols-3">
						{data.results.map((item) => (
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
	);
};

export { HistoryDetail };
