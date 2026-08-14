import {
	ChartColumn,
	Globe,
	LayoutColumns3,
	LayoutSideContent,
} from "@gravity-ui/icons";
import { Button } from "@heroui/react";
import { cn } from "cnfast";
import { type ReactNode, useState } from "react";
import { useTranslation } from "react-i18next";

type AppShellProps = {
	/** Active browser path used to highlight the current navigation item. */
	currentPath: string;
	/** Page content rendered inside the shared workspace. */
	children: ReactNode;
	/** Changes the active application route without reloading the page. */
	onNavigate: (path: string) => void;
};

const NAVIGATION_ITEMS = [
	{ path: "/", labelKey: "navigation.compare", icon: ChartColumn },
	{ path: "/runs", labelKey: "navigation.runs", icon: LayoutColumns3 },
] as const;

/**
 * Provides a collapsible desktop workspace rail and mobile navigation bar.
 *
 * @example
 * <AppShell currentPath="/" onNavigate={navigateTo}><main /></AppShell>
 */
const AppShell = ({ currentPath, children, onNavigate }: AppShellProps) => {
	const { t, i18n } = useTranslation();
	const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);

	/** Changes and persists the active UI language through i18next. */
	const changeLanguage = async (language: "en-US" | "zh-CN") => {
		await i18n.changeLanguage(language);
	};

	return (
		<div className="min-h-[100dvh] bg-[var(--app-canvas)] text-[var(--app-ink)] lg:flex">
			<Button
				aria-controls="desktop-sidebar"
				aria-expanded={!isSidebarCollapsed}
				aria-label={t(isSidebarCollapsed ? "expandSidebar" : "collapseSidebar")}
				className={cn(
					"fixed top-2 z-20 hidden rounded-md bg-transparent text-[var(--app-ink)] shadow-none motion-safe:transition-[left] motion-safe:duration-200 lg:inline-flex",
					isSidebarCollapsed ? "left-4" : "left-[175px]",
				)}
				isIconOnly
				onPress={() => setIsSidebarCollapsed((isCollapsed) => !isCollapsed)}
				size="sm"
				variant="ghost"
			>
				<LayoutSideContent aria-hidden="true" className="size-5" />
			</Button>
			<aside
				className={cn(
					"sticky top-0 hidden h-[100dvh] w-[224px] max-w-[224px] shrink-0 overflow-hidden border-r border-[var(--app-line)] bg-[var(--app-surface)] motion-safe:transition-[max-width,border-color] motion-safe:duration-200 lg:block",
					isSidebarCollapsed && "max-w-0 border-r-0",
				)}
				id="desktop-sidebar"
			>
				{!isSidebarCollapsed && (
					<div className="flex h-full w-[224px] flex-col p-4 pt-14">
						<div className="flex items-start gap-1">
							<button
								aria-label={t("appName")}
								className="mr-9 flex min-w-0 flex-1 items-center gap-3 rounded-lg p-2 text-left outline-none transition-colors hover:bg-[var(--app-hover)] focus-visible:ring-2 focus-visible:ring-zinc-800"
								onClick={() => onNavigate("/")}
								type="button"
							>
								<span className="grid size-8 shrink-0 place-items-center rounded-lg bg-zinc-900 text-zinc-50">
									<ChartColumn aria-hidden="true" className="size-4" />
								</span>
								<span className="min-w-0">
									<span className="block text-sm font-semibold tracking-[-0.02em]">
										{t("appName")}
									</span>
									<span className="block truncate text-[11px] text-[var(--app-muted)]">
										{t("appEdition")}
									</span>
								</span>
							</button>
						</div>

						<nav aria-label={t("mainNavigation")} className="mt-8 space-y-1">
							{NAVIGATION_ITEMS.map((item) => {
								const ItemIcon = item.icon;
								const isActive = currentPath === item.path;

								return (
									<Button
										aria-label={t(item.labelKey)}
										className={cn(
											"w-full justify-start rounded-lg px-3 text-sm text-[var(--app-muted)]",
											isActive &&
												"bg-[var(--app-hover)] font-semibold text-[var(--app-ink)]",
										)}
										key={item.path}
										onPress={() => onNavigate(item.path)}
										variant="ghost"
									>
										<ItemIcon aria-hidden="true" className="size-4" />
										{t(item.labelKey)}
									</Button>
								);
							})}
						</nav>

						<div className="mt-auto border-t border-[var(--app-line)] pt-4">
							<div className="mb-2 flex items-center gap-2 px-2 text-xs text-[var(--app-muted)]">
								<Globe aria-hidden="true" className="size-3.5" />
								{t("languageSelection")}
							</div>
							<div className="grid grid-cols-2 gap-1 rounded-lg bg-[var(--app-hover)] p-1">
								{(["zh-CN", "en-US"] as const).map((language) => (
									<Button
										aria-pressed={i18n.resolvedLanguage === language}
										className="w-full min-w-0 rounded-md px-2 text-xs text-[var(--app-muted)] aria-pressed:bg-[var(--app-surface)] aria-pressed:text-[var(--app-ink)] aria-pressed:shadow-[0_1px_2px_rgba(24,24,23,0.08)]"
										key={language}
										onPress={() => changeLanguage(language)}
										size="sm"
										variant="ghost"
									>
										{t(
											language === "zh-CN"
												? "languages.zhCN"
												: "languages.enUS",
										)}
									</Button>
								))}
							</div>
						</div>
					</div>
				)}
			</aside>

			<header className="sticky top-0 z-20 flex min-h-16 items-center border-b border-[var(--app-line)] bg-[color:var(--app-surface)/0.94] px-4 backdrop-blur lg:hidden">
				<button
					aria-label={t("appName")}
					className="mr-auto flex items-center gap-2 rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-zinc-800"
					onClick={() => onNavigate("/")}
					type="button"
				>
					<span className="grid size-8 place-items-center rounded-lg bg-zinc-900 text-zinc-50">
						<ChartColumn aria-hidden="true" className="size-4" />
					</span>
					<span className="text-sm font-semibold">{t("appName")}</span>
				</button>
				<nav
					aria-label={t("mainNavigation")}
					className="flex items-center gap-1"
				>
					{NAVIGATION_ITEMS.map((item) => {
						const ItemIcon = item.icon;

						return (
							<Button
								aria-label={t(item.labelKey)}
								className={cn(
									"rounded-lg text-[var(--app-muted)]",
									currentPath === item.path &&
										"bg-[var(--app-hover)] text-[var(--app-ink)]",
								)}
								isIconOnly
								key={item.path}
								onPress={() => onNavigate(item.path)}
								variant="ghost"
							>
								<ItemIcon aria-hidden="true" className="size-4" />
							</Button>
						);
					})}
				</nav>
			</header>

			<div className="min-w-0 flex-1 lg:pt-14">{children}</div>
		</div>
	);
};

export { AppShell };
