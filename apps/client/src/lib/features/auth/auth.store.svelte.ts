import { PersistedState } from "runed"
import { quizUiStore } from "$lib/features/quiz/quiz.store.svelte"
import type { AuthTokens, LoginResponse, User } from "$lib/features/auth/types"

class AuthStore {
	#tokens = new PersistedState<AuthTokens | null>("auth-tokens", null, {
		storage: "local",
		syncTabs: false,
	})

	#user = new PersistedState<User | null>("auth-user", null, {
		storage: "local",
		syncTabs: false,
	})

	isReady = $state(false)
	isBootstrapping = $state(false)

	get tokens() {
		return this.#tokens.current
	}

	get user() {
		return this.#user.current
	}

	get accessToken() {
		return this.tokens?.accessToken ?? null
	}

	get refreshToken() {
		return this.tokens?.refreshToken ?? null
	}

	get session(): LoginResponse | null {
		if (!this.tokens || !this.user) {
			return null
		}

		return {
			accessToken: this.tokens.accessToken,
			refreshToken: this.tokens.refreshToken,
			user: this.user,
		}
	}

	isAuthenticated = () => Boolean(this.accessToken && this.refreshToken && this.user)

	setSession = (session: LoginResponse) => {
		quizUiStore.clearAllStores()
		this.#tokens.current = {
			accessToken: session.accessToken,
			refreshToken: session.refreshToken,
		}
		this.#user.current = session.user
	}

	updateTokens = (tokens: AuthTokens) => {
		if (!this.user) {
			return
		}

		this.#tokens.current = tokens
	}

	clearSession = () => {
		this.#tokens.current = null
		this.#user.current = null
	}

	clearAllStores = () => {
		this.clearSession()
		quizUiStore.clearAllStores()
	}
}

export const authStore = new AuthStore()
