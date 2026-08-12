import { Check, Terminal } from "@gravity-ui/icons";
import { cn } from "@heroui/react";
import type { TFunction } from "i18next";
import { useTranslation } from "react-i18next";
import type { AgentKind, AgentRuntimeStatus } from "../types/agent";

type AgentSelectionCardProps = {
	/** Product represented by this comparison target. */
	agent: AgentKind;
	/** Current installation, login, and process message. */
	statusMessage: string;
	/** Tailwind color class for the live state indicator. */
	statusTone: string;
	/** Runtime configuration discovered from the local product. */
	runtimeStatus: AgentRuntimeStatus | null;
	/** Whether this product will participate in the next comparison. */
	isSelected: boolean;
	/** Whether product selection is temporarily unavailable. */
	isDisabled: boolean;
	/** Toggles this product in the comparison selection. */
	onToggle: (agent: AgentKind) => void;
};

/**
 * Localizes a known agent reasoning level while retaining its wire value.
 *
 * @example
 * formatReasoningEffort("high", t); // "高 (high)"
 */
const formatReasoningEffort = (effort: string | null, t: TFunction) => {
	if (!effort) {
		return t("metricUnavailable");
	}

	const localized = t(`reasoningEffortLevels.${effort}`, {
		defaultValue: effort,
	});
	return localized ? `${localized} (${effort})` : effort;
};

/**
 * Renders one Agent as a compact row in the target selection matrix.
 *
 * @example
 * <AgentSelectionCard agent="codex" statusMessage="Codex: running" statusTone="bg-zinc-800" runtimeStatus={status} isSelected isDisabled={false} onToggle={setAgent} />
 */
const AgentSelectionCard = ({
	agent,
	statusMessage,
	statusTone,
	runtimeStatus,
	isSelected,
	isDisabled,
	onToggle,
}: AgentSelectionCardProps) => {
	const { t } = useTranslation();

	return (
		<button
			aria-label={t(`agentNames.${agent}`)}
			aria-pressed={isSelected}
			className="group grid w-full gap-3 border-b border-[var(--app-line)] px-4 py-4 text-left outline-none transition-colors last:border-b-0 hover:bg-[var(--app-hover)] focus-visible:bg-[var(--app-hover)] focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-zinc-800 disabled:cursor-not-allowed disabled:opacity-45 sm:grid-cols-[minmax(0,1.2fr)_minmax(150px,1fr)_24px] sm:items-center"
			disabled={isDisabled}
			onClick={() => onToggle(agent)}
			type="button"
		>
			<span className="flex min-w-0 items-center gap-3">
				<span className="grid size-9 shrink-0 place-items-center rounded-lg border border-[var(--app-line)] bg-[var(--app-raised)] text-[var(--app-muted)]">
					<Terminal aria-hidden="true" className="size-4" />
				</span>
				<span className="min-w-0">
					<span className="flex items-center gap-2 text-sm font-semibold">
						{t(`agentNames.${agent}`)}
						<span
							aria-hidden="true"
							className={cn("size-1.5 rounded-full", statusTone)}
						/>
					</span>
					<span className="mt-0.5 block truncate text-xs text-[var(--app-muted)]">
						{statusMessage}
					</span>
				</span>
			</span>

			<span className="grid grid-cols-2 gap-3">
				<span className="block min-w-0">
					<span className="block text-[11px] text-[var(--app-faint)]">
						{t("currentModel")}
					</span>
					<span className="mt-0.5 block truncate font-mono text-[11px] font-medium text-[var(--app-ink)]">
						{runtimeStatus?.model ?? t("metricUnavailable")}
					</span>
				</span>
				<span className="block min-w-0">
					<span className="block text-[11px] text-[var(--app-faint)]">
						{t("reasoningEffort")}
					</span>
					<span className="mt-0.5 block truncate font-mono text-[11px] font-medium text-[var(--app-ink)]">
						{formatReasoningEffort(runtimeStatus?.reasoningEffort ?? null, t)}
					</span>
				</span>
			</span>

			<span
				className={cn(
					"hidden size-5 place-items-center rounded-md border border-[var(--app-line)] text-transparent transition sm:grid",
					isSelected && "border-zinc-800 bg-zinc-800 text-zinc-50",
				)}
			>
				<Check aria-hidden="true" className="size-3" />
			</span>
		</button>
	);
};

export { AgentSelectionCard };
