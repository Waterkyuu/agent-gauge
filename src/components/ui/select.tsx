import {
	Description,
	FieldError,
	Select as HeroUISelect,
	Label,
	ListBox,
} from "@heroui/react";

type SelectOption<T extends string> = {
	/** Optional supporting text rendered below the option label. */
	description?: string;
	/** Whether the option cannot be selected. */
	isDisabled?: boolean;
	/** The user-visible option label. */
	label: string;
	/** The stable business value returned when the option is selected. */
	value: T;
};

type SelectProps<T extends string> = {
	/** Additional classes applied to the select root. */
	className?: string;
	/** Optional supporting text rendered below the trigger. */
	description?: string;
	/** Validation message rendered when the select is invalid. */
	errorMessage?: string;
	/** Whether the select fills the width of its container. Defaults to true. */
	fullWidth?: boolean;
	/** Whether the select is disabled. */
	isDisabled?: boolean;
	/** Whether the current value is invalid. */
	isInvalid?: boolean;
	/** Whether a value is required. */
	isRequired?: boolean;
	/** The user-visible and accessible field label. */
	label: string;
	/** The field name used during native form submission. */
	name?: string;
	/** Called with the next selected business value. */
	onChange: (value: T | null) => void;
	/** The options available for selection. */
	options: readonly SelectOption<T>[];
	/** Text displayed when no option is selected. */
	placeholder: string;
	/** The controlled selected business value. */
	value: T | null;
	/** The visual emphasis of the select. Defaults to primary. */
	variant?: "primary" | "secondary";
};

/**
 * Renders the standard controlled single-select field used by the application.
 *
 * @example
 * <Select
 *   label="Status"
 *   onChange={setStatus}
 *   options={[{ label: 'Active', value: 'active' }]}
 *   placeholder="Choose a status"
 *   value={status}
 * />
 */
const Select = <T extends string>({
	className,
	description,
	errorMessage,
	fullWidth = true,
	isDisabled = false,
	isInvalid = false,
	isRequired = false,
	label,
	name,
	onChange,
	options,
	placeholder,
	value,
	variant = "primary",
}: SelectProps<T>) => (
	<HeroUISelect<SelectOption<T>>
		className={className}
		fullWidth={fullWidth}
		isDisabled={isDisabled}
		isInvalid={isInvalid}
		isRequired={isRequired}
		name={name}
		onChange={(nextValue) => onChange(nextValue as T | null)}
		placeholder={placeholder}
		value={value}
		variant={variant}
	>
		<Label>{label}</Label>
		<HeroUISelect.Trigger>
			<HeroUISelect.Value />
			<HeroUISelect.Indicator />
		</HeroUISelect.Trigger>
		{description && <Description>{description}</Description>}
		{errorMessage && <FieldError>{errorMessage}</FieldError>}
		<HeroUISelect.Popover>
			<ListBox>
				{options.map((option) => (
					<ListBox.Item
						id={option.value}
						isDisabled={option.isDisabled}
						key={option.value}
						textValue={option.label}
					>
						<Label>{option.label}</Label>
						{option.description && (
							<Description>{option.description}</Description>
						)}
						<ListBox.ItemIndicator />
					</ListBox.Item>
				))}
			</ListBox>
		</HeroUISelect.Popover>
	</HeroUISelect>
);

export type { SelectOption, SelectProps };
export { Select };
