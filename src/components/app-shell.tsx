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
		<div className="min-h-[100dvh] bg-canvas text-ink lg:flex">
			<Button
				aria-controls="desktop-sidebar"
				aria-expanded={!isSidebarCollapsed}
				aria-label={t(isSidebarCollapsed ? "expandSidebar" : "collapseSidebar")}
				className={cn(
					"fixed top-[6px] z-20 hidden rounded-md bg-transparent text-body shadow-none hover:text-ink motion-safe:transition-[left,color] motion-safe:duration-200 lg:inline-flex",
					isSidebarCollapsed ? "left-[72px]" : "left-[140px]",
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
					"sticky top-0 hidden h-[100dvh] w-[224px] max-w-[224px] shrink-0 overflow-hidden border-r border-hairline bg-surface-soft motion-safe:transition-[max-width,border-color] motion-safe:duration-200 lg:block",
					isSidebarCollapsed && "max-w-0 border-r-0",
				)}
				id="desktop-sidebar"
			>
				{!isSidebarCollapsed && (
					<div className="flex h-full w-[224px] flex-col p-lg pt-14">
						<div className="flex items-start gap-xs">
							<button
								aria-label={t("appName")}
								className="mr-9 flex min-w-0 flex-1 items-center gap-md rounded-lg p-sm text-left outline-none transition-colors hover:bg-surface-soft focus-visible:ring-2 focus-visible:ring-focus-ring"
								onClick={() => onNavigate("/")}
								type="button"
							>
								<span className="grid size-8 shrink-0 place-items-center rounded-lg bg-primary text-on-primary">
									<ChartColumn aria-hidden="true" className="size-4" />
								</span>
								<span className="min-w-0">
									<span className="block font-primary text-body-sm-strong font-semibold">
										{t("appName")}
									</span>
									<span className="block truncate text-caption-sm text-body">
										{t("appEdition")}
									</span>
								</span>
							</button>
						</div>

						<nav aria-label={t("mainNavigation")} className="mt-8 space-y-xs">
							{NAVIGATION_ITEMS.map((item) => {
								const ItemIcon = item.icon;
								const isActive = currentPath === item.path;

								return (
									<Button
										aria-label={t(item.labelKey)}
										className={cn(
											"w-full justify-start rounded-lg px-md text-body-sm text-body shadow-none",
											isActive && "bg-hairline font-medium text-ink",
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

						<div className="mt-auto border-t border-hairline pt-lg">
							<div className="mb-sm flex items-center gap-sm px-sm text-caption-sm text-body">
								<Globe aria-hidden="true" className="size-3.5" />
								{t("languageSelection")}
							</div>
							<div className="grid grid-cols-2 gap-xs rounded-lg bg-surface-soft p-xs">
								{(["zh-CN", "en-US"] as const).map((language) => (
									<Button
										aria-pressed={i18n.resolvedLanguage === language}
										className="w-full min-w-0 rounded-md px-sm text-caption-sm text-body shadow-none aria-pressed:bg-canvas aria-pressed:text-ink aria-pressed:shadow-sm"
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

			<header className="sticky top-0 z-20 flex min-h-16 items-center border-b border-hairline bg-canvas/95 px-lg backdrop-blur lg:hidden">
				<button
					aria-label={t("appName")}
					className="mr-auto flex items-center gap-sm rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-focus-ring"
					onClick={() => onNavigate("/")}
					type="button"
				>
					<span className="grid size-8 place-items-center rounded-lg bg-primary text-on-primary">
						<ChartColumn aria-hidden="true" className="size-4" />
					</span>
					<span className="font-primary text-body-sm-strong font-semibold">
						{t("appName")}
					</span>
				</button>
				<nav
					aria-label={t("mainNavigation")}
					className="flex items-center gap-xs"
				>
					{NAVIGATION_ITEMS.map((item) => {
						const ItemIcon = item.icon;

						return (
							<Button
								aria-label={t(item.labelKey)}
								className={cn(
									"rounded-lg text-body shadow-none",
									currentPath === item.path && "bg-hairline text-ink",
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
