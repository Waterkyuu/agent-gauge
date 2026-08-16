import { Toast } from "@heroui/react";
import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "./queries/client";
import { AppRouter } from "./routers";

/**
 * Renders the application through the shared React Router configuration.
 */
const App = () => (
	<QueryClientProvider client={queryClient}>
		<AppRouter />
		<Toast.Provider maxVisibleToasts={3} placement="top end" width={420} />
	</QueryClientProvider>
);

export default App;
