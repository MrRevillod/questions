import { browser } from "$app/environment"
import { AxiosHeaders, type AxiosError } from "axios"
import { authService } from "$lib/features/auth/auth.service"
import { authStore } from "$lib/features/auth/auth.store.svelte"
import { apiClient } from "$lib/shared/http/http"

import type { ApiRequestConfig } from "$lib/shared/http/http"

let refreshRequest: Promise<string | null> | null = null

const refreshAccessToken = async (): Promise<string | null> => {
	if (!refreshRequest) {
		refreshRequest = authService
			.refresh()
			.then(({ value: tokens, error: refreshError }) => {
				if (refreshError || !tokens) return null
				return tokens.accessToken
			})
			.finally(() => {
				refreshRequest = null
			})
	}

	return refreshRequest
}

const handleUnauthorized = async (originalConfig: ApiRequestConfig) => {
	originalConfig._retry = true

	const refreshedAccessToken = await refreshAccessToken()

	if (!refreshedAccessToken) {
		authStore.clearAllStores()
		return Promise.reject({
			type: "Unauthorized",
			message: "La sesión expiró. Debes iniciar sesión nuevamente.",
			status: 401,
		})
	}

	const headers = AxiosHeaders.from(originalConfig.headers as AxiosHeaders)

	headers.set("Authorization", `Bearer ${refreshedAccessToken}`)
	originalConfig.headers = headers

	return apiClient.request(originalConfig)
}

export const setupApiInterceptors = () => {
	const requestInterceptorId = apiClient.interceptors.request.use(config => {
		const requestConfig = config as ApiRequestConfig

		if (requestConfig.skipAuth) {
			return config
		}

		const accessToken = authStore.accessToken
		if (!accessToken) return config

		const headers = AxiosHeaders.from(config.headers)
		headers.set("Authorization", `Bearer ${accessToken}`)
		config.headers = headers

		return config
	})

	const responseInterceptorId = apiClient.interceptors.response.use(
		response => response,
		async (error: AxiosError) => {
			const originalConfig = error.config as ApiRequestConfig | undefined

			if (!originalConfig) return Promise.reject(error)

			if (
				browser &&
				error.response?.status === 401 &&
				window.location.pathname === "/login"
			) {
				return Promise.reject(error)
			}

			if (
				error.response?.status === 401 &&
				!originalConfig.skipRefresh &&
				!originalConfig._retry
			) {
				return handleUnauthorized(originalConfig)
			}

			return Promise.reject(error)
		}
	)

	return () => {
		apiClient.interceptors.request.eject(requestInterceptorId)
		apiClient.interceptors.response.eject(responseInterceptorId)
	}
}
