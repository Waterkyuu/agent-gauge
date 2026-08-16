import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
	AgentActivitiesResponse,
	AgentProcessStates,
} from "@/types/agent";

/** Reads the latest cached task lifecycle snapshot from the native monitor. */
const checkAgentActivities = () =>
	invoke<AgentActivitiesResponse>("check_agent_activities");

/**
 * Subscribes to task snapshots emitted after a supported Agent source changes.
 *
 * @example
 * onAgentActivitiesChanged(setAgentActivities);
 */
const onAgentActivitiesChanged = (
	listener: (response: AgentActivitiesResponse) => void,
) =>
	listen<AgentActivitiesResponse>("agent-activities-changed", (event) => {
		listener(event.payload);
	});

/** Reads one lightweight running-process snapshot for every supported Agent. */
const checkAgentProcesses = () =>
	invoke<AgentProcessStates>("check_agent_processes");

/**
 * Subscribes to process snapshots emitted only after a supported Agent starts or stops.
 *
 * @example
 * onAgentProcessStatesChanged(setAgentProcessStates);
 */
const onAgentProcessStatesChanged = (
	listener: (states: AgentProcessStates) => void,
) =>
	listen<AgentProcessStates>("agent-process-states-changed", (event) => {
		listener(event.payload);
	});

export {
	checkAgentActivities,
	checkAgentProcesses,
	onAgentActivitiesChanged,
	onAgentProcessStatesChanged,
};
