import {
	CircleCheck,
	CircleQuestion,
	Clock,
	Play,
	TriangleExclamation,
} from "@gravity-ui/icons";
import { Card } from "@heroui/react";
import { cn } from "cnfast";
import type { ComponentType, SVGProps } from "react";
import { useTranslation } from "react-i18next";
import { AgentLogo } from "@/components/agent-logo";
import { RUN_BOARD_ITEMS, type RunBoardStatus } from "@/constants/run-board";

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

	return (
		<main className="mx-auto max-w-[1320px] px-4 py-8 sm:px-6 lg:px-8 lg:py-10">
			<header className="mb-5 border-b border-[var(--app-line)] pb-7">
				<h1 className="text-3xl font-semibold leading-tight tracking-[-0.04em] sm:text-4xl">
					{t("runBoard.title")}
				</h1>
				<p className="mt-3 max-w-[65ch] text-sm leading-6 text-[var(--app-muted)] sm:text-base">
					{t("runBoard.description")}
				</p>
			</header>

			<div className="grid overflow-hidden rounded-xl border border-[var(--app-line)] bg-[var(--app-surface)] lg:grid-cols-2 xl:min-h-[40rem] xl:grid-cols-4">
				{BOARD_STATUSES.map((status) => {
					const presentation = STATUS_PRESENTATIONS[status];
					const StatusIcon = presentation.icon;
					const items = RUN_BOARD_ITEMS.filter(
						(item) => item.status === status,
					);

					return (
						<section
							aria-labelledby={`board-${status}`}
							className="flex min-w-0 flex-col border-b border-[var(--app-line)] lg:border-r lg:[&:nth-child(2n)]:border-r-0 lg:[&:nth-last-child(-n+2)]:border-b-0 xl:[&:nth-child(2n)]:border-r xl:[&:nth-child(4n)]:border-r-0 xl:[&:nth-last-child(-n+4)]:border-b-0"
							key={status}
						>
							<header className="flex items-center border-b border-[var(--app-line)] px-4 py-3.5">
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

							<div className="min-h-48 flex-1 space-y-3 bg-[color:var(--app-canvas)]/45 p-3">
								{items.length > 0 ? (
									items.map((item) => (
										<Card
											className="rounded-xl border border-[var(--app-line)] bg-[var(--app-raised)] shadow-none transition-colors hover:border-zinc-400"
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
