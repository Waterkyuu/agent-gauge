import { ChartColumn, Globe, LayoutColumns3 } from "@gravity-ui/icons";
import { Button, cn } from "@heroui/react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

type AppShellProps = {
	/** Active browser path used to highlight the current navigation item. */
	currentPath: string;
	/** Page content rendered below the shared navigation. */
	children: ReactNode;
	/** Changes the active application route without reloading the page. */
	onNavigate: (path: string) => void;
};

/**
 * Provides the product header, route navigation, and language controls.
 *
 * @example
 * <AppShell currentPath="/" onNavigate={navigateTo}><main /></AppShell>
 */
const AppShell = ({ currentPath, children, onNavigate }: AppShellProps) => {
	const { t, i18n } = useTranslation();

	/** Changes and persists the active UI language through i18next. */
	const changeLanguage = async (language: "en-US" | "zh-CN") => {
		await i18n.changeLanguage(language);
	};

	return (
		<div className="min-h-screen bg-white text-zinc-950">
			<header className="sticky top-0 z-40 border-b border-zinc-200/80 bg-white/90 backdrop-blur-xl">
				<div className="mx-auto flex max-w-[1440px] items-center gap-3 px-5 py-3 sm:px-8 lg:px-10">
					<button
						aria-label={t("appName")}
						className="mr-auto flex items-center gap-3 rounded-lg text-left outline-none ring-black focus-visible:ring-2 focus-visible:ring-offset-2"
						onClick={() => onNavigate("/")}
						type="button"
					>
						<span className="grid size-9 place-items-center rounded-xl bg-black text-white shadow-sm">
							<ChartColumn aria-hidden="true" className="size-[18px]" />
						</span>
						<span>
							<span className="block text-sm font-bold tracking-[-0.02em]">
								{t("appName")}
							</span>
							<span className="hidden text-[10px] font-medium uppercase tracking-[0.18em] text-zinc-400 sm:block">
								{t("appEdition")}
							</span>
						</span>
					</button>

					<nav
						aria-label={t("mainNavigation")}
						className="flex items-center gap-1"
					>
						<Button
							className={cn(
								"hidden rounded-lg px-3 text-sm sm:flex",
								currentPath === "/" && "bg-zinc-100 font-semibold text-black",
							)}
							onPress={() => onNavigate("/")}
							variant="ghost"
						>
							<ChartColumn aria-hidden="true" className="size-4" />
							{t("navigation.compare")}
						</Button>
						<Button
							aria-label={t("navigation.runs")}
							className={cn(
								"rounded-lg px-3 text-sm",
								currentPath === "/runs" &&
									"bg-zinc-100 font-semibold text-black",
							)}
							onPress={() => onNavigate("/runs")}
							variant="ghost"
						>
							<LayoutColumns3 aria-hidden="true" className="size-4" />
							<span className="hidden sm:inline">{t("navigation.runs")}</span>
						</Button>
					</nav>

					{currentPath === "/runs" ? (
						<div className="ml-1 flex items-center gap-1 border-l border-zinc-200 pl-2">
							<Globe
								aria-hidden="true"
								className="hidden size-4 text-zinc-400 md:block"
							/>
							{(["zh-CN", "en-US"] as const).map((language) => (
								<Button
									aria-pressed={i18n.resolvedLanguage === language}
									className="min-w-0 rounded-lg px-2 text-xs text-zinc-500 aria-pressed:bg-black aria-pressed:text-white"
									key={language}
									onPress={() => changeLanguage(language)}
									size="sm"
									variant="ghost"
								>
									{t(
										language === "zh-CN" ? "languages.zhCN" : "languages.enUS",
									)}
								</Button>
							))}
						</div>
					) : null}
				</div>
			</header>
			{children}
		</div>
	);
};

export { AppShell };
