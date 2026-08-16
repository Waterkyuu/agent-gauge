import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { getComparisonHistory, listComparisonHistory } from "@/api/comparison";
import type { ComparisonCursor } from "@/types/comparison";

const COMPARISON_HISTORY_PAGE_SIZE = 30;

const comparisonHistoryKeys = {
	all: ["comparison-history"] as const,
	list: () => [...comparisonHistoryKeys.all, "list"] as const,
	detail: (id: number | null) =>
		[...comparisonHistoryKeys.all, "detail", id] as const,
};

/**
 * Loads comparison summaries with keyset pagination.
 * @example const historyQuery = useComparisonHistory();
 */
const useComparisonHistory = () =>
	useInfiniteQuery({
		queryKey: comparisonHistoryKeys.list(),
		queryFn: ({ pageParam }: { pageParam: ComparisonCursor | null }) =>
			listComparisonHistory(pageParam, COMPARISON_HISTORY_PAGE_SIZE),
		initialPageParam: null,
		getNextPageParam: (lastPage) => lastPage.nextCursor ?? undefined,
	});

/**
 * Loads and caches one immutable comparison detail.
 * @example const detailQuery = useComparisonHistoryDetail(selectedId);
 */
const useComparisonHistoryDetail = (id: number | null) =>
	useQuery({
		queryKey: comparisonHistoryKeys.detail(id),
		queryFn: () => {
			if (id === null) {
				throw new Error("A comparison id is required");
			}

			return getComparisonHistory(id);
		},
		enabled: id !== null,
		staleTime: Number.POSITIVE_INFINITY,
		gcTime: 30 * 60 * 1000,
	});

export {
	comparisonHistoryKeys,
	useComparisonHistory,
	useComparisonHistoryDetail,
};
