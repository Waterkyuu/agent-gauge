import { Spinner } from "@heroui/react";
import { useTranslation } from "react-i18next";

type LoadingProps = {
	/** The layout scope occupied by the loading indicator. */
	variant: "page" | "section";
};

const loadingStyles = {
	page: {
		className: "flex min-h-dvh items-center justify-center",
		size: "lg",
	},
	section: {
		className: "flex min-h-40 items-center justify-center py-8",
		size: "md",
	},
} as const;

/** Renders a loading indicator for a full page or a content section. */
const Loading = ({ variant }: LoadingProps) => {
	const { t } = useTranslation();
	const { className, size } = loadingStyles[variant];

	return (
		<div aria-busy="true" className={className}>
			<Spinner aria-label={t("common.loading")} role="status" size={size} />
		</div>
	);
};

export type { LoadingProps };
export { Loading };
