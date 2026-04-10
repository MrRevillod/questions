import axios, { isAxiosError } from "axios"
import type { AxiosRequestConfig } from "axios"
import type { AppError } from "$lib/shared/errors"
import { Err, Ok, safeAsyncTry, type PromiseResult } from "$lib/shared/result"

type ApiResponse<T = unknown> = {
	code: number
	success: boolean
	message: string
	timestamp: string
	data?: T | null
	error?: unknown
}

export type ApiRequestConfig<TData = unknown> = AxiosRequestConfig<TData> & {
	skipAuth?: boolean
	skipRefresh?: boolean
	_retry?: boolean
}

export const apiClient = axios.create({
	baseURL: "/api",
	headers: {
		"Content-Type": "application/json",
	},
})

const UNKNOWN_ERROR: AppError = {
	type: "Unknown",
	message: "Ocurrió un error desconocido.",
}

const isApiResponse = (payload: unknown): payload is ApiResponse<unknown> => {
	if (!payload || typeof payload !== "object") {
		return false
	}

	const response = payload as Partial<ApiResponse<unknown>>

	return (
		typeof response.code === "number" &&
		typeof response.success === "boolean" &&
		typeof response.message === "string" &&
		typeof response.timestamp === "string"
	)
}

export const request = async <T>(
	config: ApiRequestConfig
): PromiseResult<T, AppError> => {
	const { value: response, error: requestError } = await safeAsyncTry(
		apiClient.request<ApiResponse<T>>(config)
	)

	if (requestError) {
		if (!isAxiosError(requestError) || !requestError.response?.data) {
			return Err({ ...UNKNOWN_ERROR, details: requestError })
		}

		const payload = requestError.response.data

		if (!isApiResponse(payload)) {
			return Err({ ...UNKNOWN_ERROR, details: requestError.response.data })
		}

		return Err({
			type: payload.code === 401 || payload.code === 403 ? "Unauthorized" : "Server",
			message: payload.message,
			status: payload.code,
			details: payload.error ?? null,
		})
	}

	if (!response || !isApiResponse(response.data)) {
		return Err({ ...UNKNOWN_ERROR, details: response?.data ?? null })
	}

	const payload = response.data

	if (!payload.success) {
		return Err({
			type: payload.code === 401 || payload.code === 403 ? "Unauthorized" : "Server",
			message: payload.message,
			status: payload.code,
			details: payload.error ?? null,
		})
	}

	return Ok(payload.data as T)
}
