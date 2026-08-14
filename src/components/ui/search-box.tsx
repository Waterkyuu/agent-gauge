import { SearchField } from "@heroui/react";
import { useTranslation } from "react-i18next";

type SearchBoxProps = {
	/** Whether the search input is disabled. */
	isDisabled?: boolean;
	/** Called with the next search value whenever the input changes. */
	onValueChange: (value: string) => void;
	/** The input placeholder. Falls back to the shared translation when omitted. */
	placeholder?: string;
	/** The current search value. */
	value: string;
};

/** Renders a controlled search input with a leading search icon. */
const SearchBox = ({
	isDisabled = false,
	onValueChange,
	placeholder,
	value,
}: SearchBoxProps) => {
	const { t } = useTranslation();
	const defaultPlaceholder = t("common.search");
	const resolvedPlaceholder = placeholder ?? defaultPlaceholder;

	return (
		<SearchField
			aria-label={resolvedPlaceholder || defaultPlaceholder}
			fullWidth
			isDisabled={isDisabled}
			onChange={onValueChange}
			value={value}
		>
			<SearchField.Group>
				<SearchField.SearchIcon />
				<SearchField.Input placeholder={resolvedPlaceholder} />
				<SearchField.ClearButton aria-label={t("common.clearSearch")} />
			</SearchField.Group>
		</SearchField>
	);
};

export type { SearchBoxProps };
export { SearchBox };
