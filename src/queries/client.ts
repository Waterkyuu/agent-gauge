import { QueryClient } from "@tanstack/react-query";

/** Shared server-state cache for the desktop application. */
const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			refetchOnWindowFocus: false,
			retry: 2,
		},
	},
});

export { queryClient };
