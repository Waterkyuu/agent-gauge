import {
	Dropdown,
	type DropdownPopoverProps,
	Header,
	Label,
	Separator,
} from "@heroui/react";
import type { ReactElement } from "react";
import { useTranslation } from "react-i18next";

type DropdownMenuItemProps<T extends string> = {
	/** Whether the item uses the danger visual treatment. */
	danger?: boolean;
	/** Stable business action returned by the menu-level callback. */
	id: T;
	/** Whether the item is disabled. */
	isDisabled?: boolean;
	/** Translation key used as both the visible label and accessible text value. */
	labelKey: string;
	/** Business callback invoked when this item is selected. */
	onAction?: () => void;
	/** Whether to insert a separator before this item. */
	separated?: boolean;
	/** Test identifier rendered as data-testid. */
	testId?: string;
};

type DropdownMenuProps<T extends string> = {
	/** Additional classes applied to the floating popover. */
	className?: string;
	/** Whether the menu closes after an item is selected. Defaults to true. */
	closeOnSelect?: boolean;
	/** Translation key rendered as the optional menu header. */
	headerKey?: string;
	/** Additional classes applied to every menu item. */
	itemClassName?: string;
	/** Business actions rendered by the menu. */
	items: readonly DropdownMenuItemProps<T>[];
	/** Additional classes applied to the menu list. */
	menuClassName?: string;
	/** Distance in pixels between the trigger and popover. Defaults to 4. */
	offset?: DropdownPopoverProps["offset"];
	/** Called after the selected item's own callback with its business action. */
	onAction?: (action: T) => void;
	/** Preferred popover placement. Defaults to bottom start. */
	placement?: DropdownPopoverProps["placement"];
	/** Pressable element that opens the menu, such as a HeroUI Button. */
	trigger: ReactElement;
};

/**
 * Renders a constrained business action menu with translated labels and stable action identifiers.
 *
 * @example
 * <DropdownMenu
 *   items={[{ id: 'delete', labelKey: 'project.delete', danger: true }]}
 *   onAction={(action) => setActiveDialog(action)}
 *   trigger={<Button aria-label="Project actions">Actions</Button>}
 * />
 */
const DropdownMenu = <T extends string>({
	className,
	closeOnSelect = true,
	headerKey,
	itemClassName,
	items,
	menuClassName,
	offset = 4,
	onAction,
	placement = "bottom start",
	trigger,
}: DropdownMenuProps<T>) => {
	const { t } = useTranslation();

	const renderedItems = items.flatMap((item, index) => {
		const elements: ReactElement[] = [];
		const label = t(item.labelKey);

		if (item.separated && index > 0) {
			elements.push(<Separator key={`${item.id}-separator`} />);
		}

		elements.push(
			<Dropdown.Item
				className={itemClassName}
				data-testid={item.testId}
				id={item.id}
				isDisabled={item.isDisabled}
				key={item.id}
				textValue={label}
				variant={item.danger ? "danger" : undefined}
			>
				<Label>{label}</Label>
			</Dropdown.Item>,
		);

		return elements;
	});

	return (
		<Dropdown>
			{trigger}
			<Dropdown.Popover
				className={className}
				offset={offset}
				placement={placement}
			>
				<Dropdown.Menu
					className={menuClassName}
					onAction={(key) => {
						const action = key as T;
						const selectedItem = items.find((item) => item.id === action);

						selectedItem?.onAction?.();
						onAction?.(action);
					}}
					shouldCloseOnSelect={closeOnSelect}
				>
					{headerKey ? (
						<Dropdown.Section>
							<Header>{t(headerKey)}</Header>
							{renderedItems}
						</Dropdown.Section>
					) : (
						renderedItems
					)}
				</Dropdown.Menu>
			</Dropdown.Popover>
		</Dropdown>
	);
};

export type { DropdownMenuItemProps, DropdownMenuProps };
export { DropdownMenu };
