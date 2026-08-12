import {
	CircleCheck,
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
	/** Tailwind classes for the status icon container. */
	iconClassName: string;
};

const BOARD_STATUSES: RunBoardStatus[] = ["running", "finish", "error"];

const STATUS_PRESENTATIONS: Record<RunBoardStatus, StatusPresentation> = {
	running: {
		icon: Play,
		iconClassName: "bg-zinc-900 text-zinc-50",
	},
	finish: {
		icon: CircleCheck,
		iconClassName: "bg-zinc-200 text-zinc-700",
	},
	error: {
		icon: TriangleExclamation,
		iconClassName: "bg-zinc-200 text-zinc-600",
	},
};

/** Renders the three-column status board with localized mock runs. */
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

			<div className="grid overflow-hidden rounded-xl border border-[var(--app-line)] bg-[var(--app-surface)] lg:grid-cols-3">
				{BOARD_STATUSES.map((status) => {
					const presentation = STATUS_PRESENTATIONS[status];
					const StatusIcon = presentation.icon;
					const items = RUN_BOARD_ITEMS.filter(
						(item) => item.status === status,
					);

					return (
						<section
							aria-labelledby={`board-${status}`}
							className="min-w-0 border-b border-[var(--app-line)] last:border-b-0 lg:border-b-0 lg:border-r lg:last:border-r-0"
							key={status}
						>
							<header className="flex items-center justify-between gap-4 border-b border-[var(--app-line)] px-4 py-3.5">
								<div className="flex min-w-0 items-center gap-3">
									<span
										className={cn(
											"grid size-8 shrink-0 place-items-center rounded-lg",
											presentation.iconClassName,
										)}
									>
										<StatusIcon aria-hidden="true" className="size-4" />
									</span>
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
								<span className="font-mono text-xs text-[var(--app-muted)] tabular-nums">
									{items.length.toString().padStart(2, "0")}
								</span>
							</header>

							<div className="min-h-48 space-y-3 bg-[color:var(--app-canvas)]/45 p-3">
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
