import { useTranslation } from "react-i18next";
import { useNavigate, useParams } from "react-router";
import { HistoryDetail } from "@/pages/comparison-history/components/history-detail";
import { HistoryList } from "@/pages/comparison-history/components/history-list";
import {
	useComparisonHistory,
	useComparisonHistoryDetail,
} from "@/queries/comparison-history";

/**
 * Routes between the comparison history list and one selected detail.
 * @example <ComparisonHistoryPage />
 */
const ComparisonHistoryPage = () => {
	const { t, i18n } = useTranslation();
	const navigate = useNavigate();
	const { comparisonId } = useParams();
	const selectedId =
		comparisonId && /^\d+$/.test(comparisonId) ? Number(comparisonId) : null;
	const historyQuery = useComparisonHistory();
	const detailQuery = useComparisonHistoryDetail(selectedId);
	const historyItems =
		historyQuery.data?.pages.flatMap((page) => page.items) ?? [];
	const numberLocale = i18n.resolvedLanguage ?? "en-US";

	/**
	 * Opens one history record at its stable detail route.
	 * @example selectComparison(42);
	 */
	const selectComparison = (id: number) => {
		navigate(`/comparison-history/${id}`);
	};

	/** Returns from one comparison detail to the history list. */
	const returnToHistory = () => {
		navigate("/comparison-history");
	};

	return (
		<main className="mx-auto max-w-[1320px] px-4 py-8 sm:px-6 lg:px-8 lg:py-10">
			{selectedId === null ? (
				<>
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
					<HistoryList
						hasNextPage={historyQuery.hasNextPage}
						isError={historyQuery.isError}
						isFetchingNextPage={historyQuery.isFetchingNextPage}
						isPending={historyQuery.isPending}
						isSuccess={historyQuery.isSuccess}
						items={historyItems}
						numberLocale={numberLocale}
						onLoadMore={() => historyQuery.fetchNextPage()}
						onSelect={selectComparison}
					/>
				</>
			) : (
				<HistoryDetail
					data={detailQuery.data}
					isError={detailQuery.isError}
					isPending={detailQuery.isPending}
					numberLocale={numberLocale}
					onBack={returnToHistory}
				/>
			)}
		</main>
	);
};

export default ComparisonHistoryPage;
