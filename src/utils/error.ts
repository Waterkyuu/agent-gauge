/**
 * Returns a safe message from a Tauri IPC or JavaScript error.
 *
 * @example
 * getErrorMessage(error, "请求失败");
 */
const getErrorMessage = (error: unknown, fallback: string) => {
	if (error instanceof Error && error.message) {
		return error.message;
	}
	if (typeof error === "object" && error !== null && "message" in error) {
		return String(error.message);
	}
	return fallback;
};

export { getErrorMessage };
