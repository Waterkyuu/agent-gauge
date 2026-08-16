import { invoke } from "@tauri-apps/api/core";
import type {
	ComparisonCursor,
	ComparisonHistoryDetail,
	ComparisonHistoryPage,
	SaveComparisonHistoryRequest,
} from "@/types/comparison";

/**
 * Persists one completed comparison and returns its history identifier.
 * @example await saveComparisonHistory({ query, results });
 */
const saveComparisonHistory = (request: SaveComparisonHistoryRequest) =>
	invoke<{ id: number }>("save_comparison_history", { request });

/**
 * Loads one bounded newest-first page without response bodies.
 * @example await listComparisonHistory(null, 30);
 */
const listComparisonHistory = (
	cursor: ComparisonCursor | null = null,
	limit = 30,
) =>
	invoke<ComparisonHistoryPage>("list_comparison_history", {
		request: { cursor, limit },
	});

/**
 * Loads one complete comparison for the history detail surface.
 * @example await getComparisonHistory(42);
 */
const getComparisonHistory = (id: number) =>
	invoke<ComparisonHistoryDetail>("get_comparison_history", {
		request: { id },
	});

export { getComparisonHistory, listComparisonHistory, saveComparisonHistory };
