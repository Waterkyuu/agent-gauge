import {
	CircleCheck,
	CircleQuestion,
	Clock,
	LayoutColumns3,
	LayoutRows3,
	Play,
	TriangleExclamation,
} from "@gravity-ui/icons";
import { Button, Card, Tooltip } from "@heroui/react";
import { cn } from "cnfast";
import { type ComponentType, type SVGProps, useState } from "react";
import { useTranslation } from "react-i18next";
import { AgentLogo } from "@/components/agent-logo";
import { SearchBox } from "@/components/ui/search-box";
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
		iconClassName: "text-ink",
	},
	waiting: {
		icon: CircleQuestion,
		iconClassName: "text-terminal-yellow",
	},
	finish: {
		icon: CircleCheck,
		iconClassName: "text-terminal-green",
	},
	error: {
		icon: TriangleExclamation,
		iconClassName: "text-terminal-red",
	},
};

/** Renders the searchable four-state run board with localized mock runs. */
const RunBoardPage = () => {
	const { t } = useTranslation();
	const [layout, setLayout] = useState<RunBoardLayout>("vertical");
	const [agentQuery, setAgentQuery] = useState("");
	const agentSearchTerm = agentQuery.trim().toLocaleLowerCase();

	return (
		<main className="mx-auto max-w-[1320px] px-4 py-8 sm:px-6 lg:px-8 lg:py-10">
			<header className="mb-5 flex flex-col gap-5 border-b border-hairline pb-7 sm:flex-row sm:items-end sm:justify-between">
				<div>
					<h1 className="font-primary text-display-lg font-medium leading-display-lg sm:text-display-xl sm:leading-display-xl">
						{t("runBoard.title")}
					</h1>
					<p className="mt-md max-w-[65ch] text-body-sm leading-body-md text-body sm:text-body-md">
						{t("runBoard.description")}
					</p>
				</div>
				<div className="flex w-full flex-col gap-3 sm:w-auto sm:flex-row sm:items-center">
					<div className="w-full sm:w-72">
						<SearchBox
							onValueChange={setAgentQuery}
							placeholder={t("runBoard.searchPlaceholder")}
							value={agentQuery}
						/>
					</div>
					<fieldset
						aria-label={t("runBoard.layoutSelection")}
						className="flex shrink-0 items-center gap-xs self-start rounded-lg bg-surface-soft p-xs sm:self-auto"
					>
						<Tooltip delay={0}>
							<Button
								aria-pressed={layout === "vertical"}
								aria-label={t("runBoard.verticalLayout")}
								className="rounded-md px-2.5 text-caption-sm text-body shadow-none aria-pressed:bg-canvas aria-pressed:text-ink aria-pressed:shadow-sm"
								onPress={() => setLayout("vertical")}
								size="sm"
								variant="ghost"
							>
								<LayoutColumns3 aria-hidden="true" className="size-4" />
							</Button>
							<Tooltip.Content
								className="w-max max-w-none whitespace-nowrap break-normal"
								placement="bottom"
							>
								{t("runBoard.verticalLayout")}
							</Tooltip.Content>
						</Tooltip>
						<Tooltip delay={0}>
							<Button
								aria-pressed={layout === "horizontal"}
								aria-label={t("runBoard.horizontalLayout")}
								className="rounded-md px-2.5 text-caption-sm text-body shadow-none aria-pressed:bg-canvas aria-pressed:text-ink aria-pressed:shadow-sm"
								onPress={() => setLayout("horizontal")}
								size="sm"
								variant="ghost"
							>
								<LayoutRows3 aria-hidden="true" className="size-4" />
							</Button>
							<Tooltip.Content
								className="w-max max-w-none whitespace-nowrap break-normal"
								placement="bottom"
							>
								{t("runBoard.horizontalLayout")}
							</Tooltip.Content>
						</Tooltip>
					</fieldset>
				</div>
			</header>

			<div
				className={cn(
					"grid overflow-hidden rounded-xl border border-hairline bg-surface-card",
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
						(item) =>
							item.status === status &&
							t(`agentNames.${item.agent}`)
								.toLocaleLowerCase()
								.includes(agentSearchTerm),
					);

					return (
						<section
							aria-labelledby={`board-${status}`}
							className={cn(
								"flex min-w-0 flex-col border-b border-hairline",
								layout === "vertical" &&
									"lg:border-r lg:[&:nth-child(2n)]:border-r-0 lg:[&:nth-last-child(-n+2)]:border-b-0 xl:[&:nth-child(2n)]:border-r xl:[&:nth-child(4n)]:border-r-0 xl:[&:nth-last-child(-n+4)]:border-b-0",
								layout === "horizontal" && "lg:flex-row lg:last:border-b-0",
							)}
							key={status}
						>
							<header
								className={cn(
									"flex items-center border-b border-hairline px-4 py-3.5",
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
											className="text-body-sm-strong font-medium"
											id={`board-${status}`}
										>
											{t(`runBoard.status.${status}`)}
										</h2>
										<p className="truncate text-caption-sm text-body">
											{t(`runBoard.statusDescription.${status}`)}
										</p>
									</div>
								</div>
							</header>

							<div
								className={cn(
									"min-h-48 flex-1 space-y-3 bg-surface-soft/50 p-3",
									layout === "horizontal" &&
										"lg:flex lg:flex-wrap lg:items-start lg:gap-3 lg:space-y-0",
								)}
							>
								{items.length > 0 ? (
									items.map((item) => (
										<Card
											className={cn(
												"h-48 w-[18rem] max-w-full overflow-hidden rounded-xl border border-hairline bg-surface-card shadow-none transition-colors hover:border-hairline-strong",
											)}
											key={item.id}
											role="article"
										>
											<Card.Content className="p-3">
												<div className="flex items-center justify-between gap-3 text-caption-sm text-mute">
													<span className="font-mono">{item.id}</span>
													<span className="flex min-w-0 max-w-[55%] items-center gap-1.5 truncate">
														<AgentLogo
															agent={item.agent}
															className="size-3.5"
														/>
														<span className="truncate">
															{t(`agentNames.${item.agent}`)}
														</span>
													</span>
												</div>
												<h3 className="mt-3 line-clamp-2 overflow-hidden text-body-sm-strong font-medium">
													{t(item.titleKey)}
												</h3>
												<p className="mt-1 line-clamp-2 overflow-hidden text-caption-sm leading-body-sm text-body">
													{t(item.descriptionKey)}
												</p>
												<div className="mt-3 flex items-center justify-between border-t border-hairline pt-2 font-mono text-caption-sm text-mute">
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
									<p className="px-4 py-10 text-center text-caption-sm text-body">
										{agentSearchTerm
											? t("runBoard.noSearchResults")
											: t("runBoard.empty")}
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
