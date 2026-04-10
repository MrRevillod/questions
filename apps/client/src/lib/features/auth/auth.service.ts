import { authStore } from "$lib/features/auth/auth.store.svelte"
import { request } from "$lib/shared/http/http"
import { Err, Ok, type PromiseResult } from "$lib/shared/result"
import type { AppError } from "$lib/shared/errors"
import type { AuthTokens, LoginInput, LoginResponse } from "$lib/features/auth/types"

class AuthService {
	#bootstrapPromise: PromiseResult<void, AppError> | null = null

	#clearClientSession = () => {
		authStore.clearAllStores()
	}

	login = async (input: LoginInput): PromiseResult<LoginResponse, AppError> => {
		const { value: session, error: loginError } = await request<LoginResponse>({
			method: "POST",
			url: "/auth/login",
			data: input,
			skipAuth: true,
			skipRefresh: true,
		})

		if (loginError) {
			this.#clearClientSession()
			return Err(loginError)
		}

		authStore.setSession(session)
		return Ok(session)
	}

	refresh = async (): PromiseResult<AuthTokens, AppError> => {
		if (!authStore.refreshToken) {
			this.#clearClientSession()
			return Err({
				type: "Domain",
				message: "No se encontró el token de refresco de la sesión.",
				status: 401,
			})
		}

		const { value: tokens, error: refreshError } = await request<AuthTokens>({
			method: "POST",
			url: "/auth/refresh",
			data: null,
			headers: {
				Authorization: `Bearer ${authStore.refreshToken}`,
			},
			skipAuth: true,
			skipRefresh: true,
		})

		if (refreshError) {
			this.#clearClientSession()
			return Err(refreshError)
		}

		if (!authStore.user) {
			this.#clearClientSession()
			return Err({
				type: "InvalidResponse",
				message: "No es posible refrescar tokens sin un usuario autenticado.",
			})
		}

		authStore.updateTokens(tokens)
		return Ok(tokens)
	}

	logout = async (): PromiseResult<void, AppError> => {
		if (!authStore.accessToken) {
			this.#clearClientSession()
			return Ok(undefined)
		}

		const { error: logoutError } = await request<null>({
			method: "POST",
			url: "/auth/logout",
			data: null,
			headers: {
				Authorization: `Bearer ${authStore.accessToken}`,
			},
			skipRefresh: true,
		})

		this.#clearClientSession()

		if (logoutError) {
			return Err(logoutError)
		}

		return Ok(undefined)
	}

	bootstrapSession = async (): PromiseResult<void, AppError> => {
		if (authStore.isReady) {
			return Ok(undefined)
		}

		if (this.#bootstrapPromise) {
			return this.#bootstrapPromise
		}

		this.#bootstrapPromise = (async () => {
			authStore.isBootstrapping = true

			if (!authStore.refreshToken) {
				this.#clearClientSession()
				authStore.isReady = true
				authStore.isBootstrapping = false
				return Ok(undefined)
			}

			const { error: refreshError } = await this.refresh()

			authStore.isReady = true
			authStore.isBootstrapping = false

			if (refreshError) {
				this.#clearClientSession()
				return Err(refreshError)
			}

			return Ok(undefined)
		})()

		const result = await this.#bootstrapPromise
		this.#bootstrapPromise = null

		return result
	}
}

export const authService = new AuthService()
