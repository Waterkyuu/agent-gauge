import type { AgentKind } from "@/types/agent";

type RunBoardStatus = "running" | "finish" | "error";

type RunBoardItem = {
	/** Stable identifier for the mocked run. */
	id: string;
	/** Translation key for the run title. */
	titleKey: string;
	/** Translation key for the run description. */
	descriptionKey: string;
	/** Agent product assigned to the run. */
	agent: AgentKind;
	/** Time displayed for this scheduled run. */
	time: string;
	/** Human-readable elapsed or planned duration. */
	duration: string;
	/** Current lifecycle column for the run. */
	status: RunBoardStatus;
};

const RUN_BOARD_ITEMS: RunBoardItem[] = [
	{
		id: "run-1042",
		titleKey: "runBoard.items.repositoryAudit.title",
		descriptionKey: "runBoard.items.repositoryAudit.description",
		agent: "codex",
		time: "09:30",
		duration: "18m 24s",
		status: "running",
	},
	{
		id: "run-1045",
		titleKey: "runBoard.items.releaseNotes.title",
		descriptionKey: "runBoard.items.releaseNotes.description",
		agent: "claude",
		time: "10:15",
		duration: "06m 12s",
		status: "running",
	},
	{
		id: "run-1037",
		titleKey: "runBoard.items.apiComparison.title",
		descriptionKey: "runBoard.items.apiComparison.description",
		agent: "workbuddy",
		time: "08:00",
		duration: "12m 08s",
		status: "finish",
	},
	{
		id: "run-1039",
		titleKey: "runBoard.items.testCoverage.title",
		descriptionKey: "runBoard.items.testCoverage.description",
		agent: "codex",
		time: "08:45",
		duration: "09m 41s",
		status: "finish",
	},
	{
		id: "run-1031",
		titleKey: "runBoard.items.migrationPlan.title",
		descriptionKey: "runBoard.items.migrationPlan.description",
		agent: "claude",
		time: "07:30",
		duration: "04m 56s",
		status: "error",
	},
	{
		id: "run-1033",
		titleKey: "runBoard.items.dependencyScan.title",
		descriptionKey: "runBoard.items.dependencyScan.description",
		agent: "workbuddy",
		time: "07:50",
		duration: "02m 17s",
		status: "error",
	},
];

export type { RunBoardItem, RunBoardStatus };
export { RUN_BOARD_ITEMS };
