export type Result<T, E> = { value: T; error: null } | { value: null; error: E }
export type PromiseResult<T, E> = Promise<Result<T, E>>

export const Ok = <T>(value: T): Result<T, never> => ({
	value,
	error: null,
})

export const Err = <E>(error: E): Result<never, E> => ({
	value: null,
	error,
})

export const isOk = <T, E>(
	result: Result<T, E>
): result is { value: T; error: null } => result.error === null

export const isErr = <T, E>(
	result: Result<T, E>
): result is { value: null; error: E } => result.error !== null

export const safeAsyncTry = async <T>(
	promise: Promise<T>
): PromiseResult<T, unknown> => {
	try {
		const value = await promise
		return Ok(value)
	} catch (error) {
		return Err(error)
	}
}

export const safeTry = <T>(fn: () => T): Result<T, unknown> => {
	try {
		return Ok(fn())
	} catch (error) {
		return Err(error)
	}
}
