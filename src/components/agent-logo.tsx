import { cn } from "cnfast";
import claudeCodeLogo from "@/assets/images/claude-code.svg";
import codexLogo from "@/assets/images/codex.svg";
import workBuddyLogo from "@/assets/images/workbuddy.svg";
import type { AgentKind } from "@/types/agent";

type AgentLogoProps = {
	/** Agent product whose brand mark should be rendered. */
	agent: AgentKind;
	/** Additional classes used to size the logo for its surrounding surface. */
	className?: string;
};

const AGENT_LOGO_SOURCES: Record<AgentKind, string> = {
	claude: claudeCodeLogo,
	codex: codexLogo,
	workbuddy: workBuddyLogo,
};

/**
 * Renders the official logo for an Agent as a decorative brand mark.
 *
 * @example
 * <AgentLogo agent="codex" className="size-5" />
 */
const AgentLogo = ({ agent, className }: AgentLogoProps) => (
	<img
		alt=""
		aria-hidden="true"
		className={cn("block shrink-0 object-contain", className)}
		draggable={false}
		src={AGENT_LOGO_SOURCES[agent]}
	/>
);

export { AgentLogo };
