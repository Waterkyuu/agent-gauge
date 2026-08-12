import {
	type ButtonProps,
	type DrawerContentProps,
	Drawer as HeroUIDrawer,
} from "@heroui/react";
import type { ReactElement, ReactNode } from "react";

type DrawerPlacement = NonNullable<DrawerContentProps["placement"]>;

type DrawerTriggerProps = Pick<ButtonProps, "onPress">;

type DrawerBaseProps = {
	/** Custom drawer content rendered below the title and description. */
	children?: ReactNode;
	/** Additional classes applied to the drawer panel. */
	className?: string;
	/** Optional description rendered below the title. */
	description?: string;
	/** Optional actions rendered in the drawer footer. */
	footer?: ReactNode;
	/** Whether outside interaction and drag gestures can close the drawer. */
	isDismissable?: boolean;
	/** The edge from which the drawer enters. Defaults to right. */
	placement?: DrawerPlacement;
	/** The drawer title. */
	title: string;
};

type DrawerControlledProps = {
	/** The open state in controlled mode. */
	isOpen: boolean;
	/** Called when the drawer requests an open state change. */
	onOpenChange: (isOpen: boolean) => void;
	/** Controlled mode does not accept an internal trigger element. */
	trigger?: never;
};

type DrawerUncontrolledProps = {
	/** Uncontrolled mode does not accept an external open state. */
	isOpen?: never;
	/** Called when the internal open state changes. */
	onOpenChange?: (isOpen: boolean) => void;
	/** A pressable element used to open the drawer. */
	trigger: ReactElement<DrawerTriggerProps>;
};

type DrawerProps = DrawerBaseProps &
	(DrawerControlledProps | DrawerUncontrolledProps);

/**
 * Renders a supplementary panel that supports controlled state or an internal trigger.
 *
 * @example
 * <Drawer title="Filters" trigger={<Button>Open filters</Button>}>
 *   <FilterForm />
 * </Drawer>
 */
const Drawer = ({
	children,
	className,
	description,
	footer,
	isDismissable = true,
	isOpen,
	onOpenChange,
	placement = "right",
	title,
	trigger,
}: DrawerProps) => {
	const isVertical = placement === "top" || placement === "bottom";

	return (
		<HeroUIDrawer isOpen={isOpen} onOpenChange={onOpenChange}>
			{trigger ?? <HeroUIDrawer.Trigger aria-hidden className="hidden" />}
			<HeroUIDrawer.Backdrop isDismissable={isDismissable}>
				<HeroUIDrawer.Content placement={placement}>
					<HeroUIDrawer.Dialog className={className}>
						{isVertical && <HeroUIDrawer.Handle />}
						<HeroUIDrawer.CloseTrigger />
						<HeroUIDrawer.Header>
							<HeroUIDrawer.Heading>{title}</HeroUIDrawer.Heading>
						</HeroUIDrawer.Header>
						{(description || children) && (
							<HeroUIDrawer.Body className="flex flex-col gap-3">
								{description && <p>{description}</p>}
								{children}
							</HeroUIDrawer.Body>
						)}
						{footer && <HeroUIDrawer.Footer>{footer}</HeroUIDrawer.Footer>}
					</HeroUIDrawer.Dialog>
				</HeroUIDrawer.Content>
			</HeroUIDrawer.Backdrop>
		</HeroUIDrawer>
	);
};

export type { DrawerProps };
export { Drawer };
