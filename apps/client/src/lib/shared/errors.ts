export type AppErrorType =
	| "Domain"
	| "Unauthorized"
	| "Network"
	| "Server"
	| "InvalidResponse"
	| "Unknown"

export type AppError = {
	type: AppErrorType
	message: string
	status?: number
	details?: unknown
}

export const toUserMessage = (error: AppError | null): string => {
	if (!error) {
		return ""
	}

	return error.message
}
