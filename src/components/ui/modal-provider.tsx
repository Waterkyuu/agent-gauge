import {
	type ButtonProps,
	Modal as HeroUIModal,
	type ModalContainerProps,
} from "@heroui/react";
import type { ReactElement, ReactNode } from "react";

type ModalProviderSize = NonNullable<ModalContainerProps["size"]>;

type ModalProviderTriggerProps = Pick<ButtonProps, "onPress">;

type ModalProviderBaseProps = {
	/** Custom modal content rendered below the title and description. */
	children?: ReactNode;
	/** Additional classes applied to the modal content card. */
	className?: string;
	/** Optional description rendered below the title. */
	description?: string;
	/** Optional actions rendered in the modal footer. */
	footer?: ReactNode;
	/** The size forwarded to the underlying Modal. Defaults to md. */
	size?: ModalProviderSize;
	/** The modal title. */
	title: string;
};

type ModalProviderControlledProps = {
	/** The open state in controlled mode. */
	isOpen: boolean;
	/** Called when the modal requests an open state change. */
	onOpenChange: (isOpen: boolean) => void;
	/** Controlled mode does not accept an internal trigger element. */
	trigger?: never;
};

type ModalProviderUncontrolledProps = {
	/** Uncontrolled mode does not accept an external open state. */
	isOpen?: never;
	/** Called when the internal open state changes. */
	onOpenChange?: (isOpen: boolean) => void;
	/** A pressable element used to open the modal. */
	trigger: ReactElement<ModalProviderTriggerProps>;
};

type ModalProviderProps = ModalProviderBaseProps &
	(ModalProviderControlledProps | ModalProviderUncontrolledProps);

/**
 * Renders a general-purpose modal that supports controlled state or an internal trigger.
 *
 * @example
 * <ModalProvider
 *   title="Edit profile"
 *   trigger={<Button>Edit</Button>}
 * >
 *   <ProfileForm />
 * </ModalProvider>
 */
const ModalProvider = ({
	children,
	className,
	description,
	footer,
	isOpen,
	onOpenChange,
	size = "md",
	title,
	trigger,
}: ModalProviderProps) => (
	<HeroUIModal isOpen={isOpen} onOpenChange={onOpenChange}>
		{trigger ?? <HeroUIModal.Trigger aria-hidden className="hidden" />}
		<HeroUIModal.Backdrop>
			<HeroUIModal.Container size={size}>
				<HeroUIModal.Dialog className={className}>
					<HeroUIModal.CloseTrigger />
					<HeroUIModal.Header>
						<HeroUIModal.Heading>{title}</HeroUIModal.Heading>
					</HeroUIModal.Header>
					{(description || children) && (
						<HeroUIModal.Body className="flex flex-col gap-3">
							{description && <p>{description}</p>}
							{children}
						</HeroUIModal.Body>
					)}
					{footer && <HeroUIModal.Footer>{footer}</HeroUIModal.Footer>}
				</HeroUIModal.Dialog>
			</HeroUIModal.Container>
		</HeroUIModal.Backdrop>
	</HeroUIModal>
);

export type { ModalProviderProps };
export { ModalProvider };
