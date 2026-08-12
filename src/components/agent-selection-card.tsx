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
			className="rounded-2xl border border-white/10 bg-white/5 p-4 text-left transition hover:border-white/20 disabled:cursor-not-allowed disabled:opacity-60 aria-pressed:border-indigo-400 aria-pressed:bg-indigo-500/10"
			disabled={isDisabled}
			onClick={() => onToggle(agent)}
			type="button"
		>
			<div className="flex items-center justify-between gap-3">
				<span className="font-semibold">{t(`agentNames.${agent}`)}</span>
				<span
					aria-hidden="true"
					className={`size-2 rounded-full ${statusTone}`}
				/>
			</div>
			<p className="mt-2 text-xs text-slate-400">{statusMessage}</p>
			<div className="mt-4 grid grid-cols-2 gap-3 border-t border-white/10 pt-3">
				<div>
					<p className="text-[11px] text-slate-500">{t("currentModel")}</p>
					<p className="mt-1 truncate text-xs text-slate-200">
						{runtimeStatus?.model ?? t("metricUnavailable")}
					</p>
				</div>
				<div>
					<p className="text-[11px] text-slate-500">{t("reasoningEffort")}</p>
					<p className="mt-1 truncate text-xs text-slate-200">
						{formatReasoningEffort(runtimeStatus?.reasoningEffort ?? null, t)}
					</p>
				</div>
			</div>
		</button>
	);
};

export { AgentSelectionCard };
