import {
	type AlertDialogContainerProps,
	Button,
	type ButtonProps,
	AlertDialog as HeroUIAlertDialog,
} from "@heroui/react";
import type { ReactElement, ReactNode } from "react";
import { useTranslation } from "react-i18next";

type AlertDialogSize = NonNullable<AlertDialogContainerProps["size"]>;

type AlertDialogTriggerProps = Pick<ButtonProps, "onPress">;

type AlertDialogBaseProps = {
	/** The cancel button label. Falls back to the shared translation when omitted. */
	cancelText?: string;
	/** Custom dialog content rendered below the title and description. */
	children?: ReactNode;
	/** Additional classes applied to the dialog content card. */
	className?: string;
	/** The confirm button label. Falls back to the shared translation when omitted. */
	confirmText?: string;
	/** Whether the confirm button is disabled. */
	isConfirmDisabled?: boolean;
	/** Business callback invoked when the confirm button is pressed. */
	onConfirm: () => void;
	/** The size forwarded to the underlying AlertDialog. Defaults to md. */
	size?: AlertDialogSize;
	/** Optional description rendered below the title. */
	description?: string;
	/** The dialog title. */
	title: string;
};

type AlertDialogControlledProps = {
	/** The open state in controlled mode. */
	isOpen: boolean;
	/** Called when the dialog requests an open state change. */
	onOpenChange: (isOpen: boolean) => void;
	/** Controlled mode does not accept an internal trigger element. */
	trigger?: never;
};

type AlertDialogUncontrolledProps = {
	/** Uncontrolled mode does not accept an external open state. */
	isOpen?: never;
	/** Called when the internal open state changes. */
	onOpenChange?: (isOpen: boolean) => void;
	/** A pressable element used to open the dialog. */
	trigger: ReactElement<AlertDialogTriggerProps>;
};

type AlertDialogProps = AlertDialogBaseProps &
	(AlertDialogControlledProps | AlertDialogUncontrolledProps);

/**
 * Renders a confirmation dialog that supports controlled state or an internal trigger.
 *
 * @example
 * <AlertDialog
 *   title="Delete project?"
 *   trigger={<Button>Delete</Button>}
 *   onConfirm={deleteProject}
 * />
 */
const AlertDialog = ({
	cancelText,
	children,
	className,
	confirmText,
	isConfirmDisabled = false,
	isOpen,
	onConfirm,
	onOpenChange,
	size = "md",
	description,
	title,
	trigger,
}: AlertDialogProps) => {
	const { t } = useTranslation();

	return (
		<HeroUIAlertDialog isOpen={isOpen} onOpenChange={onOpenChange}>
			{trigger ?? <HeroUIAlertDialog.Trigger aria-hidden className="hidden" />}
			<HeroUIAlertDialog.Backdrop>
				<HeroUIAlertDialog.Container size={size}>
					<HeroUIAlertDialog.Dialog className={className}>
						<HeroUIAlertDialog.Header>
							<HeroUIAlertDialog.Icon />
							<HeroUIAlertDialog.Heading>{title}</HeroUIAlertDialog.Heading>
						</HeroUIAlertDialog.Header>
						{(description || children) && (
							<HeroUIAlertDialog.Body className="flex flex-col gap-3">
								{description && <p>{description}</p>}
								{children}
							</HeroUIAlertDialog.Body>
						)}
						<HeroUIAlertDialog.Footer>
							<Button slot="close" variant="tertiary">
								{cancelText ?? t("common.cancel")}
							</Button>
							<Button
								isDisabled={isConfirmDisabled}
								onPress={onConfirm}
								slot="close"
								variant="danger"
							>
								{confirmText ?? t("common.confirm")}
							</Button>
						</HeroUIAlertDialog.Footer>
					</HeroUIAlertDialog.Dialog>
				</HeroUIAlertDialog.Container>
			</HeroUIAlertDialog.Backdrop>
		</HeroUIAlertDialog>
	);
};

export { AlertDialog };
