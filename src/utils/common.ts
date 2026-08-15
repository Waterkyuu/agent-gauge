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

export { debounce };
