type UtilityCallback = (...args: never[]) => unknown;

type ControlledFunction<T extends UtilityCallback> = ((
	...args: Parameters<T>
) => void) & {
	/** Cancels the pending callback invocation. */
	cancel: () => void;
};

/**
 * Creates a function that invokes the callback after calls have stopped for the given delay.
 *
 * @example
 * const handleSearch = debounce((keyword: string) => search(keyword), 300);
 */
const debounce = <T extends UtilityCallback>(
	callback: T,
	delay = 300,
): ControlledFunction<T> => {
	let timer: ReturnType<typeof setTimeout> | undefined;
	const wait = Math.max(0, delay);

	/**
	 * Restarts the pending callback timer with the latest arguments.
	 *
	 * @example
	 * debounced('latest value');
	 */
	const debounced = (...args: Parameters<T>) => {
		if (timer !== undefined) {
			clearTimeout(timer);
		}

		timer = setTimeout(() => {
			timer = undefined;
			callback(...args);
		}, wait);
	};

	/**
	 * Prevents the currently pending callback from running.
	 *
	 * @example
	 * debounced.cancel();
	 */
	debounced.cancel = () => {
		if (timer !== undefined) {
			clearTimeout(timer);
			timer = undefined;
		}
	};

	return debounced;
};

/**
 * Creates a function that invokes the callback at most once per delay, including the latest
 * trailing call.
 *
 * @example
 * const handleScroll = throttle((position: number) => updatePosition(position), 100);
 */
const throttle = <T extends UtilityCallback>(
	callback: T,
	delay = 300,
): ControlledFunction<T> => {
	let timer: ReturnType<typeof setTimeout> | undefined;
	let lastInvocationTime = 0;
	let latestArgs: Parameters<T> | undefined;
	const wait = Math.max(0, delay);

	/**
	 * Runs the callback with the latest arguments and updates the throttle window.
	 *
	 * @example
	 * invokeCallback();
	 */
	const invokeCallback = () => {
		const args = latestArgs;

		timer = undefined;
		latestArgs = undefined;
		lastInvocationTime = Date.now();

		if (args !== undefined) {
			callback(...args);
		}
	};

	/**
	 * Runs immediately when possible and otherwise schedules the latest call.
	 *
	 * @example
	 * throttled(120);
	 */
	const throttled = (...args: Parameters<T>) => {
		const remainingTime = wait - (Date.now() - lastInvocationTime);

		latestArgs = args;

		if (remainingTime <= 0 || remainingTime > wait) {
			if (timer !== undefined) {
				clearTimeout(timer);
			}

			invokeCallback();
			return;
		}

		if (timer === undefined) {
			timer = setTimeout(invokeCallback, remainingTime);
		}
	};

	/**
	 * Prevents the trailing callback and resets the throttle window.
	 *
	 * @example
	 * throttled.cancel();
	 */
	throttled.cancel = () => {
		if (timer !== undefined) {
			clearTimeout(timer);
		}

		timer = undefined;
		latestArgs = undefined;
		lastInvocationTime = 0;
	};

	return throttled;
};

const DATE_TIME_TOKEN_PATTERN = /YYYY|MM|DD|HH|mm|ss/g;

/**
 * Pads a date part to two digits for a consistent output format.
 *
 * @example
 * padDatePart(7);
 */
const padDatePart = (value: number) => value.toString().padStart(2, "0");

/**
 * Converts a date value to a local date-time string using common format tokens.
 *
 * @example
 * formatDateTime('2026-07-31T08:30:00', 'YYYY/MM/DD HH:mm');
 */
const formatDateTime = (
	value: Date | number | string,
	format = "YYYY-MM-DD HH:mm:ss",
): string => {
	const date = value instanceof Date ? value : new Date(value);

	if (Number.isNaN(date.getTime())) {
		return "";
	}

	const dateParts: Record<string, string> = {
		YYYY: date.getFullYear().toString(),
		MM: padDatePart(date.getMonth() + 1),
		DD: padDatePart(date.getDate()),
		HH: padDatePart(date.getHours()),
		mm: padDatePart(date.getMinutes()),
		ss: padDatePart(date.getSeconds()),
	};

	return format.replace(
		DATE_TIME_TOKEN_PATTERN,
		(token) => dateParts[token] ?? token,
	);
};

const FILE_SIZE_UNITS = ["B", "KB", "MB", "GB"] as const;

/**
 * Format the byte count to a concise file size.
 *
 * example：
 * ```ts
 * const label = formatFileSize(1536);
 * ```
 */
const formatFileSize = (size: number): string => {
	let unitIndex = 0;
	let value = size;
	while (value >= 1024 && unitIndex < FILE_SIZE_UNITS.length - 1) {
		value /= 1024;
		unitIndex += 1;
	}

	return `${Number(value.toFixed(1))} ${FILE_SIZE_UNITS[unitIndex]}`;
};

export { debounce, formatDateTime, formatFileSize, throttle };
