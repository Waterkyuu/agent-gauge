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
 * Renders one selectable Agent with its live state and runtime configuration.
 *
 * @example
 * <AgentSelectionCard agent="codex" statusMessage="Codex: running" statusTone="bg-emerald-400" runtimeStatus={status} isSelected isDisabled={false} onToggle={setAgent} />
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
			className="group block w-full rounded-2xl text-left outline-none ring-black transition focus-visible:ring-2 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-45"
			disabled={isDisabled}
			onClick={() => onToggle(agent)}
			type="button"
		>
			<Card
				className={cn(
					"h-full overflow-hidden border border-zinc-200 bg-white shadow-[0_8px_30px_rgba(0,0,0,0.035)] transition group-hover:-translate-y-0.5 group-hover:border-zinc-400 group-hover:shadow-[0_14px_38px_rgba(0,0,0,0.07)]",
					isSelected && "border-black ring-1 ring-black",
				)}
			>
				<Card.Content className="p-5">
					<div className="flex items-start justify-between gap-3">
						<span className="grid size-11 place-items-center rounded-xl border border-zinc-200 bg-zinc-50">
							<Terminal aria-hidden="true" className="size-5" />
						</span>
						<span
							className={cn(
								"grid size-6 place-items-center rounded-full border border-zinc-200 text-transparent transition",
								isSelected && "border-black bg-black text-white",
							)}
						>
							<Check aria-hidden="true" className="size-3.5" />
						</span>
					</div>
					<div className="mt-5 flex items-center gap-2">
						<span className="text-base font-bold tracking-[-0.02em]">
							{t(`agentNames.${agent}`)}
						</span>
						<span
							aria-hidden="true"
							className={cn("size-2 rounded-full", statusTone)}
						/>
					</div>
					<p className="mt-1.5 min-h-8 text-xs leading-4 text-zinc-500">
						{statusMessage}
					</p>
				</Card.Content>
				<Card.Footer className="grid grid-cols-2 gap-3 border-t border-zinc-100 bg-zinc-50/60 px-5 py-4">
					<div className="min-w-0">
						<p className="text-[10px] font-semibold uppercase tracking-[0.1em] text-zinc-400">
							{t("currentModel")}
						</p>
						<p className="mt-1.5 truncate text-xs font-semibold text-zinc-700">
							{runtimeStatus?.model ?? t("metricUnavailable")}
						</p>
					</div>
					<div className="min-w-0 border-l border-zinc-200 pl-3">
						<p className="text-[10px] font-semibold uppercase tracking-[0.1em] text-zinc-400">
							{t("reasoningEffort")}
						</p>
						<p className="mt-1.5 truncate text-xs font-semibold text-zinc-700">
							{formatReasoningEffort(runtimeStatus?.reasoningEffort ?? null, t)}
						</p>
					</div>
				</Card.Footer>
			</Card>
		</button>
	);
};

export { AgentSelectionCard };

import { Check, Terminal } from "@gravity-ui/icons";
import { Card, cn } from "@heroui/react";
