import { Checkbox as HeroUICheckbox } from "@heroui/react";
import type { ReactNode } from "react";

type CheckBoxProps = {
	/** Additional classes applied to the checkbox root. */
	className?: string;
	/** Whether the checkbox is disabled. */
	isDisabled?: boolean;
	/** Whether the checkbox displays an indeterminate state. */
	isIndeterminate?: boolean;
	/** The controlled selected state. */
	isSelected: boolean;
	/** The clickable label rendered beside the checkbox control. */
	label: ReactNode;
	/** Called with the next selected state. */
	onChange: (isSelected: boolean) => void;
	/** The visual emphasis of the checkbox. Defaults to primary. */
	variant?: "primary" | "secondary";
};

/**
 * Renders the standard controlled checkbox structure used by the application.
 *
 * @example
 * <CheckBox isSelected={enabled} label="Enable notifications" onChange={setEnabled} />
 */
const CheckBox = ({
	className,
	isDisabled = false,
	isIndeterminate = false,
	isSelected,
	label,
	onChange,
	variant = "primary",
}: CheckBoxProps) => (
	<HeroUICheckbox
		className={className}
		isDisabled={isDisabled}
		isIndeterminate={isIndeterminate}
		isSelected={isSelected}
		onChange={onChange}
		variant={variant}
	>
		<HeroUICheckbox.Content>
			<HeroUICheckbox.Control>
				<HeroUICheckbox.Indicator />
			</HeroUICheckbox.Control>
			{label}
		</HeroUICheckbox.Content>
	</HeroUICheckbox>
);

export type { CheckBoxProps };
export { CheckBox };
