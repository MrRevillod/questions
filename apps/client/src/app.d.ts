// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
import "axios"

declare module "axios" {
	export interface AxiosRequestConfig {
		skipAuth?: boolean
		skipRefresh?: boolean
	}
}

declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {}
