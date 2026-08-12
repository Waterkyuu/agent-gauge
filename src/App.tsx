import { useEffect, useState } from "react";
import { AppShell } from "./components/app-shell";
import { ComparisonPage } from "./pages/comparison-page";
import { RunBoardPage } from "./pages/run-board-page";

type AppPath = "/" | "/runs";

/**
 * Normalizes browser locations to routes supported by the desktop application.
 *
 * @example
 * resolveAppPath("/runs"); // "/runs"
 */
const resolveAppPath = (pathname: string): AppPath =>
	pathname === "/runs" ? "/runs" : "/";

/**
 * Renders the shared application shell and switches pages through the History API.
 */
const App = () => {
	const [currentPath, setCurrentPath] = useState<AppPath>(() =>
		resolveAppPath(window.location.pathname),
	);

	useEffect(() => {
		/** Keeps the visible page synchronized with browser back and forward actions. */
		const syncBrowserPath = () => {
			setCurrentPath(resolveAppPath(window.location.pathname));
		};

		window.addEventListener("popstate", syncBrowserPath);
		return () => window.removeEventListener("popstate", syncBrowserPath);
	}, []);

	/**
	 * Navigates without a full document reload.
	 *
	 * @example
	 * navigateTo("/runs");
	 */
	const navigateTo = (path: string) => {
		const nextPath = resolveAppPath(path);
		if (nextPath !== currentPath) {
			window.history.pushState({}, "", nextPath);
			setCurrentPath(nextPath);
		}
	};

	return (
		<AppShell currentPath={currentPath} onNavigate={navigateTo}>
			{currentPath === "/runs" ? <RunBoardPage /> : <ComparisonPage />}
		</AppShell>
	);
};

export default App;
