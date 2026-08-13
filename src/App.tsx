import { Toast } from "@heroui/react";
import { AppRouter } from "./routers";

/**
 * Renders the application through the shared React Router configuration.
 */
const App = () => (
	<>
		<AppRouter />
		<Toast.Provider maxVisibleToasts={3} placement="top end" width={420} />
	</>
);

export default App;
