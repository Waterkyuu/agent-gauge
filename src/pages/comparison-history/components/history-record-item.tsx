import { Ellipsis } from "@gravity-ui/icons";
import { Button } from "@heroui/react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { AgentLogo } from "@/components/agent-logo";
import { AlertDialog } from "@/components/ui/alert-dialog";
import { DropdownMenu } from "@/components/ui/dropdown-menu";
import { ModalProvider } from "@/components/ui/modal-provider";
import type { ComparisonSummary } from "@/types/comparison";

type HistoryRecordItemProps = {
	/** Persisted comparison summary rendered by this row. */
	item: ComparisonSummary;
	/** Locale used to format the comparison timestamp. */
	numberLocale: string;
	/** Opens the selected comparison detail route. */
	onSelect: (id: number) => void;
};

/**
 * Renders one compact history row with its independent action menu and dialogs.
 * @example <HistoryRecordItem item={summary} numberLocale="zh-CN" onSelect={openDetail} />
 */
const HistoryRecordItem = ({
	item,
	numberLocale,
	onSelect,
}: HistoryRecordItemProps) => {
	const { t } = useTranslation();
	const [isRenameOpen, setIsRenameOpen] = useState(false);
	const [isDeleteOpen, setIsDeleteOpen] = useState(false);
	const createdAt = new Intl.DateTimeFormat(numberLocale, {
		dateStyle: "medium",
		timeStyle: "short",
	}).format(item.createdAtMs);

	return (
		<>
			<div className="group grid grid-cols-[minmax(0,1fr)_auto] items-stretch transition-colors hover:bg-surface-soft">
				<button
					aria-label={item.query}
					className="grid min-w-0 cursor-pointer grid-cols-[minmax(0,1fr)_auto] items-center gap-md px-sm py-sm text-left outline-none active:bg-hairline focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus-ring sm:grid-cols-[minmax(0,1fr)_auto_auto] sm:gap-xl sm:px-md"
					onClick={() => onSelect(item.id)}
					type="button"
				>
					<span className="min-w-0">
						<span className="block truncate text-body-sm font-medium leading-body-sm text-ink">
							{item.query}
						</span>
						<span className="mt-0.5 block text-caption-sm text-body sm:hidden">
							{createdAt}
						</span>
					</span>
					<span className="hidden whitespace-nowrap text-caption-sm text-body sm:block">
						{createdAt}
					</span>
					<span className="flex -space-x-1">
						{item.agents.map((agent) => (
							<span
								className="rounded-full border border-hairline bg-canvas p-px"
								key={agent.agent}
							>
								<AgentLogo agent={agent.agent} className="size-4" />
							</span>
						))}
					</span>
				</button>
				<DropdownMenu
					items={[
						{
							id: "rename",
							labelKey: "comparisonHistory.rename",
							onAction: () => setIsRenameOpen(true),
						},
						{
							danger: true,
							id: "delete",
							labelKey: "comparisonHistory.delete",
							onAction: () => setIsDeleteOpen(true),
							separated: true,
						},
					]}
					placement="bottom end"
					trigger={
						<Button
							aria-label={t("comparisonHistory.recordActions", {
								query: item.query,
							})}
							className="my-auto mr-xs min-w-0 cursor-pointer rounded-md p-sm text-body shadow-none hover:bg-hairline hover:text-ink"
							isIconOnly
							size="sm"
							variant="ghost"
						>
							<Ellipsis aria-hidden="true" className="size-4" />
						</Button>
					}
				/>
			</div>
			<ModalProvider
				description={t("comparisonHistory.renameDescription", {
					query: item.query,
				})}
				isOpen={isRenameOpen}
				onOpenChange={setIsRenameOpen}
				title={t("comparisonHistory.renameTitle")}
			/>
			<AlertDialog
				confirmText={t("comparisonHistory.deleteConfirm")}
				description={t("comparisonHistory.deleteDescription", {
					query: item.query,
				})}
				isOpen={isDeleteOpen}
				onConfirm={() => setIsDeleteOpen(false)}
				onOpenChange={setIsDeleteOpen}
				title={t("comparisonHistory.deleteTitle")}
			/>
		</>
	);
};

export { HistoryRecordItem };
