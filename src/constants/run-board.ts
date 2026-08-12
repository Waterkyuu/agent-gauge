type RunBoardStatus = "running" | "finish" | "error";

type RunBoardItem = {
	/** Stable identifier for the mocked run. */
	id: string;
	/** Translation key for the run title. */
	titleKey: string;
	/** Translation key for the run description. */
	descriptionKey: string;
	/** Agent product assigned to the run. */
	agent: string;
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
		agent: "Codex",
		time: "09:30",
		duration: "18m 24s",
		status: "running",
	},
	{
		id: "run-1045",
		titleKey: "runBoard.items.releaseNotes.title",
		descriptionKey: "runBoard.items.releaseNotes.description",
		agent: "Claude Code",
		time: "10:15",
		duration: "06m 12s",
		status: "running",
	},
	{
		id: "run-1037",
		titleKey: "runBoard.items.apiComparison.title",
		descriptionKey: "runBoard.items.apiComparison.description",
		agent: "WorkBuddy",
		time: "08:00",
		duration: "12m 08s",
		status: "finish",
	},
	{
		id: "run-1039",
		titleKey: "runBoard.items.testCoverage.title",
		descriptionKey: "runBoard.items.testCoverage.description",
		agent: "Codex",
		time: "08:45",
		duration: "09m 41s",
		status: "finish",
	},
	{
		id: "run-1031",
		titleKey: "runBoard.items.migrationPlan.title",
		descriptionKey: "runBoard.items.migrationPlan.description",
		agent: "Claude Code",
		time: "07:30",
		duration: "04m 56s",
		status: "error",
	},
	{
		id: "run-1033",
		titleKey: "runBoard.items.dependencyScan.title",
		descriptionKey: "runBoard.items.dependencyScan.description",
		agent: "WorkBuddy",
		time: "07:50",
		duration: "02m 17s",
		status: "error",
	},
];

const RUN_BOARD_DATES = [
	{ dayKey: "weekdays.mon", date: 10 },
	{ dayKey: "weekdays.tue", date: 11 },
	{ dayKey: "weekdays.wed", date: 12 },
	{ dayKey: "weekdays.thu", date: 13 },
	{ dayKey: "weekdays.fri", date: 14 },
	{ dayKey: "weekdays.sat", date: 15 },
	{ dayKey: "weekdays.sun", date: 16 },
];

export type { RunBoardItem, RunBoardStatus };
export { RUN_BOARD_DATES, RUN_BOARD_ITEMS };
