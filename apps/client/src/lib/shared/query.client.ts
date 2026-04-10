import { QueryClient } from "@tanstack/svelte-query"

export const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			staleTime: 30_000,
			refetchOnWindowFocus: false,
			retry: false,
		},
		mutations: {
			retry: false,
		},
	},
})
