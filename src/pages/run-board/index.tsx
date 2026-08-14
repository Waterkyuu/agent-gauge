import {
	CircleCheck,
	CircleQuestion,
	Clock,
	LayoutColumns3,
	LayoutRows3,
	Play,
	TriangleExclamation,
} from "@gravity-ui/icons";
import { Button, Card } from "@heroui/react";
import { cn } from "cnfast";
import { type ComponentType, type SVGProps, useState } from "react";
import { useTranslation } from "react-i18next";
import { AgentLogo } from "@/components/agent-logo";
import { RUN_BOARD_ITEMS, type RunBoardStatus } from "@/constants/run-board";

type RunBoardLayout = "vertical" | "horizontal";

type StatusPresentation = {
	/** Icon rendered beside the status name. */
	icon: ComponentType<SVGProps<SVGSVGElement>>;
	/** Tailwind color class for the status icon. */
	iconClassName: string;
};

const BOARD_STATUSES: RunBoardStatus[] = [
	"running",
	"waiting",
	"finish",
	"error",
];

const STATUS_PRESENTATIONS: Record<RunBoardStatus, StatusPresentation> = {
	running: {
		icon: Play,
		iconClassName: "text-blue-600",
	},
	waiting: {
		icon: CircleQuestion,
		iconClassName: "text-amber-500",
	},
	finish: {
		icon: CircleCheck,
		iconClassName: "text-emerald-600",
	},
	error: {
		icon: TriangleExclamation,
		iconClassName: "text-rose-600",
	},
};

/** Renders the four-state run board with localized mock runs. */
const RunBoardPage = () => {
	const { t } = useTranslation();
	const [layout, setLayout] = useState<RunBoardLayout>("vertical");

	return (
		<main className="mx-auto max-w-[1320px] px-4 py-8 sm:px-6 lg:px-8 lg:py-10">
			<header className="mb-5 flex flex-col gap-5 border-b border-[var(--app-line)] pb-7 sm:flex-row sm:items-end sm:justify-between">
				<div>
					<h1 className="text-3xl font-semibold leading-tight tracking-[-0.04em] sm:text-4xl">
						{t("runBoard.title")}
					</h1>
					<p className="mt-3 max-w-[65ch] text-sm leading-6 text-[var(--app-muted)] sm:text-base">
						{t("runBoard.description")}
					</p>
				</div>
				<fieldset
					aria-label={t("runBoard.layoutSelection")}
					className="flex shrink-0 items-center gap-1 rounded-lg bg-[var(--app-hover)] p-1"
				>
					<Button
						aria-pressed={layout === "vertical"}
						className="rounded-md px-2.5 text-xs text-[var(--app-muted)] aria-pressed:bg-[var(--app-surface)] aria-pressed:text-[var(--app-ink)] aria-pressed:shadow-[0_1px_2px_rgba(24,24,23,0.08)]"
						onPress={() => setLayout("vertical")}
						size="sm"
						variant="ghost"
					>
						<LayoutColumns3 aria-hidden="true" className="size-4" />
						{t("runBoard.verticalLayout")}
					</Button>
					<Button
						aria-pressed={layout === "horizontal"}
						className="rounded-md px-2.5 text-xs text-[var(--app-muted)] aria-pressed:bg-[var(--app-surface)] aria-pressed:text-[var(--app-ink)] aria-pressed:shadow-[0_1px_2px_rgba(24,24,23,0.08)]"
						onPress={() => setLayout("horizontal")}
						size="sm"
						variant="ghost"
					>
						<LayoutRows3 aria-hidden="true" className="size-4" />
						{t("runBoard.horizontalLayout")}
					</Button>
				</fieldset>
			</header>

			<div
				className={cn(
					"grid overflow-hidden rounded-xl border border-[var(--app-line)] bg-[var(--app-surface)]",
					layout === "vertical" &&
						"lg:grid-cols-2 xl:min-h-[40rem] xl:grid-cols-4",
				)}
				data-layout={layout}
				data-testid="run-board"
			>
				{BOARD_STATUSES.map((status) => {
					const presentation = STATUS_PRESENTATIONS[status];
					const StatusIcon = presentation.icon;
					const items = RUN_BOARD_ITEMS.filter(
						(item) => item.status === status,
					);

					return (
						<section
							aria-labelledby={`board-${status}`}
							className={cn(
								"flex min-w-0 flex-col border-b border-[var(--app-line)]",
								layout === "vertical" &&
									"lg:border-r lg:[&:nth-child(2n)]:border-r-0 lg:[&:nth-last-child(-n+2)]:border-b-0 xl:[&:nth-child(2n)]:border-r xl:[&:nth-child(4n)]:border-r-0 xl:[&:nth-last-child(-n+4)]:border-b-0",
								layout === "horizontal" && "lg:flex-row lg:last:border-b-0",
							)}
							key={status}
						>
							<header
								className={cn(
									"flex items-center border-b border-[var(--app-line)] px-4 py-3.5",
									layout === "horizontal" &&
										"lg:w-56 lg:shrink-0 lg:border-b-0 lg:border-r",
								)}
							>
								<div className="flex min-w-0 items-center gap-3">
									<StatusIcon
										aria-hidden="true"
										className={cn(
											"size-5 shrink-0",
											presentation.iconClassName,
										)}
									/>
									<div className="min-w-0">
										<h2
											className="text-sm font-semibold"
											id={`board-${status}`}
										>
											{t(`runBoard.status.${status}`)}
										</h2>
										<p className="truncate text-[11px] text-[var(--app-muted)]">
											{t(`runBoard.statusDescription.${status}`)}
										</p>
									</div>
								</div>
							</header>

							<div
								className={cn(
									"min-h-48 flex-1 space-y-3 bg-[color:var(--app-canvas)]/45 p-3",
									layout === "horizontal" &&
										"lg:flex lg:flex-wrap lg:items-start lg:gap-3 lg:space-y-0",
								)}
							>
								{items.length > 0 ? (
									items.map((item) => (
										<Card
											className={cn(
												"rounded-xl border border-[var(--app-line)] bg-[var(--app-raised)] shadow-none transition-colors hover:border-zinc-400",
												layout === "horizontal" && "lg:min-w-64 lg:flex-1",
											)}
											key={item.id}
											role="article"
										>
											<Card.Content className="p-4">
												<div className="flex items-center justify-between gap-3 text-[11px] text-[var(--app-faint)]">
													<span className="font-mono">{item.id}</span>
													<span className="flex items-center gap-1.5">
														<AgentLogo
															agent={item.agent}
															className="size-3.5"
														/>
														{t(`agentNames.${item.agent}`)}
													</span>
												</div>
												<h3 className="mt-4 text-sm font-semibold tracking-[-0.015em]">
													{t(item.titleKey)}
												</h3>
												<p className="mt-1.5 text-xs leading-5 text-[var(--app-muted)]">
													{t(item.descriptionKey)}
												</p>
												<div className="mt-4 flex items-center justify-between border-t border-[var(--app-line)] pt-3 font-mono text-[11px] text-[var(--app-faint)]">
													<span>{item.time}</span>
													<span className="flex items-center gap-1.5">
														<Clock aria-hidden="true" className="size-3.5" />
														{item.duration}
													</span>
												</div>
											</Card.Content>
										</Card>
									))
								) : (
									<p className="px-4 py-10 text-center text-xs text-[var(--app-muted)]">
										{t("runBoard.empty")}
									</p>
								)}
							</div>
						</section>
					);
				})}
			</div>
		</main>
	);
};

export default RunBoardPage;
