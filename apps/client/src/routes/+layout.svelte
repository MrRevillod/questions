<script lang="ts">
	import { onMount } from "svelte"
	import { QueryClientProvider } from "@tanstack/svelte-query"
	import { Toaster } from "svelte-sonner"
	import "./layout.css"
	import { setupApiInterceptors } from "$lib/shared/http/api.interceptors"
	import { queryClient } from "$lib/shared/query.client"

	let { children } = $props()

	onMount(() => setupApiInterceptors())
</script>

<QueryClientProvider client={queryClient}>
	<Toaster richColors position="top-right" />
	{@render children()}
</QueryClientProvider>
