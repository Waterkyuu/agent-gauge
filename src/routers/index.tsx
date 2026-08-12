import { lazy, Suspense } from "react";
import { useTranslation } from "react-i18next";
import {
	BrowserRouter,
	Navigate,
	Route,
	Routes,
	useLocation,
	useNavigate,
} from "react-router";
import { AppShell } from "@/components/app-shell";

/** Loads the comparison route only when it is first rendered. */
const loadComparisonPage = async () => {
	const module = await import("@/pages/comparison");
	return { default: module.ComparisonPage };
};

/** Loads the run board route only when it is first rendered. */
const loadRunBoardPage = async () => {
	const module = await import("@/pages/run-board");
	return { default: module.RunBoardPage };
};

const ComparisonPage = lazy(loadComparisonPage);
const RunBoardPage = lazy(loadRunBoardPage);

/** Displays a lightweight page skeleton while a route chunk is loading. */
const RouteLoadingFallback = () => {
	const { t } = useTranslation();

	return (
		<main
			aria-label={t("loadingPage")}
			aria-live="polite"
			className="mx-auto max-w-[1320px] px-4 py-8 sm:px-6 lg:px-8 lg:py-10"
			role="status"
		>
			<div className="motion-safe:animate-pulse">
				<div className="h-4 w-48 rounded bg-[var(--app-line)]" />
				<div className="mt-5 h-10 max-w-2xl rounded bg-[var(--app-line)]" />
				<div className="mt-4 h-5 max-w-xl rounded bg-[var(--app-line)]" />
			</div>
		</main>
	);
};

/** Connects the shared application shell to the active React Router location. */
const RoutedApplication = () => {
	const { pathname } = useLocation();
	const navigate = useNavigate();

	return (
		<AppShell currentPath={pathname} onNavigate={(path) => navigate(path)}>
			<Suspense fallback={<RouteLoadingFallback />}>
				<Routes>
					<Route element={<ComparisonPage />} path="/" />
					<Route element={<RunBoardPage />} path="/runs" />
					<Route element={<Navigate replace to="/" />} path="*" />
				</Routes>
			</Suspense>
		</AppShell>
	);
};

/** Provides browser history routing for the desktop application. */
const AppRouter = () => (
	<BrowserRouter>
		<RoutedApplication />
	</BrowserRouter>
);

export { AppRouter };
