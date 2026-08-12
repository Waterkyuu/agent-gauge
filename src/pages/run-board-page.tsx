import {
	CircleCheck,
	Clock,
	LayoutColumns3,
	Play,
	TriangleExclamation,
} from "@gravity-ui/icons";
import { Card, Chip, cn } from "@heroui/react";
import type { ComponentType, SVGProps } from "react";
import { useTranslation } from "react-i18next";
import { RUN_BOARD_ITEMS, type RunBoardStatus } from "../constants/run-board";

type StatusPresentation = {
	/** Icon rendered beside the status name. */
	icon: ComponentType<SVGProps<SVGSVGElement>>;
	/** Tailwind classes for the status icon container. */
	iconClassName: string;
	/** Tailwind classes for the item accent. */
	accentClassName: string;
};

const BOARD_STATUSES: RunBoardStatus[] = ["running", "finish", "error"];

const STATUS_PRESENTATIONS: Record<RunBoardStatus, StatusPresentation> = {
	running: {
		icon: Play,
		iconClassName: "bg-black text-white",
		accentClassName: "bg-black",
	},
	finish: {
		icon: CircleCheck,
		iconClassName: "bg-zinc-100 text-zinc-700",
		accentClassName: "bg-zinc-400",
	},
	error: {
		icon: TriangleExclamation,
		iconClassName: "bg-zinc-100 text-zinc-700",
		accentClassName: "bg-zinc-300",
	},
};

/** Renders the status-based mock run board. */
const RunBoardPage = () => {
	const { t } = useTranslation();

	return (
		<main className="mx-auto max-w-[1440px] px-5 py-8 sm:px-8 lg:px-10 lg:py-12">
			<section className="mb-8 flex flex-col justify-between gap-5 md:flex-row md:items-end">
				<div>
					<div className="mb-3 flex items-center gap-2 text-sm font-semibold text-zinc-500">
						<LayoutColumns3 aria-hidden="true" className="size-4" />
						{t("runBoard.eyebrow")}
					</div>
					<h1 className="text-4xl font-bold tracking-[-0.045em] sm:text-5xl">
						{t("runBoard.title")}
					</h1>
					<p className="mt-3 max-w-2xl text-sm leading-6 text-zinc-500 sm:text-base">
						{t("runBoard.description")}
					</p>
				</div>
				<Chip color="default" size="lg" variant="soft">
					<span className="size-2 rounded-full bg-black" />
					{t("runBoard.liveSync")}
				</Chip>
			</section>

			<div className="grid items-start gap-4 xl:grid-cols-3">
				{BOARD_STATUSES.map((status) => {
					const presentation = STATUS_PRESENTATIONS[status];
					const StatusIcon = presentation.icon;
					const items = RUN_BOARD_ITEMS.filter(
						(item) => item.status === status,
					);

					return (
						<section
							aria-labelledby={`board-${status}`}
							className="rounded-2xl border border-zinc-200 bg-zinc-50/70 p-3"
							key={status}
						>
							<div className="flex items-center justify-between px-1 pb-3 pt-1">
								<div className="flex items-center gap-2.5">
									<span
										className={cn(
											"grid size-8 place-items-center rounded-lg",
											presentation.iconClassName,
										)}
									>
										<StatusIcon aria-hidden="true" className="size-4" />
									</span>
									<div>
										<h2 className="text-sm font-bold" id={`board-${status}`}>
											{t(`runBoard.status.${status}`)}
										</h2>
										<p className="text-[11px] text-zinc-400">
											{t(`runBoard.statusDescription.${status}`)}
										</p>
									</div>
								</div>
								<span className="rounded-full border border-zinc-200 bg-white px-2.5 py-1 text-xs font-bold tabular-nums">
									{items.length}
								</span>
							</div>

							<div className="space-y-3">
								{items.map((item) => (
									<Card
										className="relative overflow-hidden border border-zinc-200 bg-white shadow-[0_8px_24px_rgba(0,0,0,0.035)] transition-transform hover:-translate-y-0.5"
										key={item.id}
										role="article"
									>
										<span
											aria-hidden="true"
											className={cn(
												"absolute inset-y-0 left-0 w-1",
												presentation.accentClassName,
											)}
										/>
										<Card.Header className="flex items-center justify-between gap-3 px-5 pb-0 pt-4">
											<span className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
												{item.id}
											</span>
											<Chip size="sm" variant="soft">
												{item.agent}
											</Chip>
										</Card.Header>
										<Card.Content className="px-5 py-4">
											<h3 className="text-sm font-bold tracking-[-0.015em]">
												{t(item.titleKey)}
											</h3>
											<p className="mt-2 text-xs leading-5 text-zinc-500">
												{t(item.descriptionKey)}
											</p>
										</Card.Content>
										<Card.Footer className="flex items-center justify-between border-t border-zinc-100 px-5 py-3 text-[11px] font-medium text-zinc-400">
											<span>{item.time}</span>
											<span className="flex items-center gap-1.5">
												<Clock aria-hidden="true" className="size-3.5" />
												{item.duration}
											</span>
										</Card.Footer>
									</Card>
								))}
							</div>
						</section>
					);
				})}
			</div>
		</main>
	);
};

export { RunBoardPage };
