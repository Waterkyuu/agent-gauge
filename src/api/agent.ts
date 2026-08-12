import { invoke } from "@tauri-apps/api/core";
import type { AgentProcessStates } from "@/types/agent";

/** Reads one lightweight running-process snapshot for every supported Agent. */
const checkAgentProcesses = () =>
	invoke<AgentProcessStates>("check_agent_processes");

export { checkAgentProcesses };
