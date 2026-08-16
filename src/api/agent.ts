import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AgentProcessStates } from "@/types/agent";

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

export { checkAgentProcesses, onAgentProcessStatesChanged };
