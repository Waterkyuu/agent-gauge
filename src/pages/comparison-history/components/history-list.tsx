import { ClockArrowRotateLeft, TriangleExclamation } from "@gravity-ui/icons";
import { Button, Skeleton } from "@heroui/react";
import { useTranslation } from "react-i18next";
import type { ComparisonSummary } from "@/types/comparison";
import { HistoryRecordItem } from "./history-record-item";

type HistoryListProps = {
	/** Whether another history page is available. */
	hasNextPage: boolean;
	/** Whether the initial history request failed. */
	isError: boolean;
	/** Whether another history page is loading. */
	isFetchingNextPage: boolean;
	/** Whether the initial history request is loading. */
	isPending: boolean;
	/** Whether the initial history request completed successfully. */
	isSuccess: boolean;
	/** Loaded comparison summaries ordered newest first. */
	items: ComparisonSummary[];
	/** Locale used to format row timestamps. */
	numberLocale: string;
	/** Loads the next history page. */
	onLoadMore: () => void;
	/** Opens one comparison detail route. */
	onSelect: (id: number) => void;
};

/**
 * Renders the compact history list and all of its request states.
 * @example <HistoryList items={items} isSuccess onSelect={openDetail} />
 */
const HistoryList = ({
	hasNextPage,
	isError,
	isFetchingNextPage,
	isPending,
	isSuccess,
	items,
	numberLocale,
	onLoadMore,
	onSelect,
}: HistoryListProps) => {
	const { t } = useTranslation();

	return (
		<section aria-labelledby="history-list-title" className="overflow-hidden">
			<header className="border-b border-hairline px-sm py-sm sm:px-md">
				<h2 className="text-body-sm-strong font-medium" id="history-list-title">
					{t("comparisonHistory.listTitle")}
				</h2>
			</header>
			{isPending ? (
				<div
					aria-label={t("comparisonHistory.loading")}
					className="divide-y divide-hairline border-b border-hairline"
					role="status"
				>
					{[0, 1, 2].map((item) => (
						<div className="flex items-center gap-xl px-md py-md" key={item}>
							<Skeleton className="h-4 flex-1 rounded-full" />
							<Skeleton className="h-3 w-32 rounded-full" />
						</div>
					))}
				</div>
			) : null}
			{isError ? (
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
			{isSuccess && items.length === 0 ? (
				<div className="p-8 text-center">
					<ClockArrowRotateLeft
						aria-hidden="true"
						className="mx-auto size-6 text-mute"
					/>
					<h2 className="mt-md text-body-sm-strong font-medium">
						{t("comparisonHistory.emptyTitle")}
					</h2>
					<p className="mx-auto mt-sm w-full max-w-[20rem] text-caption-sm text-body">
						{t("comparisonHistory.emptyDescription")}
					</p>
				</div>
			) : null}
			{items.length > 0 ? (
				<div className="divide-y divide-hairline border-b border-hairline">
					{items.map((item) => (
						<HistoryRecordItem
							item={item}
							key={item.id}
							numberLocale={numberLocale}
							onSelect={onSelect}
						/>
					))}
					{hasNextPage ? (
						<div className="border-t border-hairline p-md">
							<Button
								className="w-full rounded-lg text-body-sm shadow-none"
								isDisabled={isFetchingNextPage}
								onPress={onLoadMore}
								variant="ghost"
							>
								{isFetchingNextPage
									? t("comparisonHistory.loading")
									: t("comparisonHistory.loadMore")}
							</Button>
						</div>
					) : null}
				</div>
			) : null}
		</section>
	);
};

export { HistoryList };
