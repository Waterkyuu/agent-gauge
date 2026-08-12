import {
	Popover as HeroUIPopover,
	type PopoverContentProps,
} from "@heroui/react";
import type { ReactElement, ReactNode } from "react";

type PopoverPlacement = NonNullable<PopoverContentProps["placement"]>;

type PopoverBaseProps = {
	/** The rich content rendered below the popover title. */
	children: ReactNode;
	/** Additional classes applied to the popover content container. */
	className?: string;
	/** The preferred placement relative to the trigger. Defaults to bottom. */
	placement?: PopoverPlacement;
	/** Whether to render an arrow pointing to the trigger. */
	showArrow?: boolean;
	/** The accessible title rendered at the top of the popover. */
	title: string;
	/** The element that opens the popover. */
	trigger: ReactElement;
};

type PopoverControlledProps = {
	/** The open state in controlled mode. */
	isOpen: boolean;
	/** Called when the popover requests an open state change. */
	onOpenChange: (isOpen: boolean) => void;
};

type PopoverUncontrolledProps = {
	/** Uncontrolled mode does not accept an external open state. */
	isOpen?: never;
	/** Called when the internal open state changes. */
	onOpenChange?: (isOpen: boolean) => void;
};

type PopoverProps = PopoverBaseProps &
	(PopoverControlledProps | PopoverUncontrolledProps);

/**
 * Renders a titled popover that supports controlled or internal open state.
 *
 * @example
 * <Popover title="Account" trigger={<Button>Open</Button>}>
 *   <AccountSummary />
 * </Popover>
 */
const Popover = ({
	children,
	className,
	isOpen,
	onOpenChange,
	placement = "bottom",
	showArrow = false,
	title,
	trigger,
}: PopoverProps) => (
	<HeroUIPopover isOpen={isOpen} onOpenChange={onOpenChange}>
		<HeroUIPopover.Trigger>{trigger}</HeroUIPopover.Trigger>
		<HeroUIPopover.Content className={className} placement={placement}>
			{showArrow && <HeroUIPopover.Arrow />}
			<HeroUIPopover.Dialog>
				<HeroUIPopover.Heading>{title}</HeroUIPopover.Heading>
				{children}
			</HeroUIPopover.Dialog>
		</HeroUIPopover.Content>
	</HeroUIPopover>
);

export type { PopoverProps };
export { Popover };
