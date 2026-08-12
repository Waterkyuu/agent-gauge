import { Description, Switch as HeroUISwitch } from "@heroui/react";
import type { ReactNode } from "react";

type SwitchProps = {
	/** Additional classes applied to the switch root. */
	className?: string;
	/** Supporting text rendered below the clickable switch content. */
	description?: string;
	/** Whether the switch is disabled. */
	isDisabled?: boolean;
	/** The controlled selected state. */
	isSelected: boolean;
	/** The clickable label rendered beside the switch control. */
	label: ReactNode;
	/** Called with the next selected state. */
	onChange: (isSelected: boolean) => void;
	/** The visual size of the switch. Defaults to md. */
	size?: "sm" | "md" | "lg";
};

/**
 * Renders the standard controlled switch structure used by the application.
 *
 * @example
 * <Switch
 *   description="Receive updates about account activity."
 *   isSelected={enabled}
 *   label="Enable notifications"
 *   onChange={setEnabled}
 * />
 */
const Switch = ({
	className,
	description,
	isDisabled = false,
	isSelected,
	label,
	onChange,
	size = "md",
}: SwitchProps) => (
	<HeroUISwitch
		className={className}
		isDisabled={isDisabled}
		isSelected={isSelected}
		onChange={onChange}
		size={size}
	>
		<HeroUISwitch.Content>
			<HeroUISwitch.Control>
				<HeroUISwitch.Thumb />
			</HeroUISwitch.Control>
			{label}
		</HeroUISwitch.Content>
		{description && <Description>{description}</Description>}
	</HeroUISwitch>
);

export type { SwitchProps };
export { Switch };
