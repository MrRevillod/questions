import { redirect } from "@sveltejs/kit"
import { authService } from "$lib/features/auth/auth.service"
import { authStore } from "$lib/features/auth/auth.store.svelte"

export const load = async () => {
	await authService.bootstrapSession()

	if (!authStore.isAuthenticated()) {
		throw redirect(302, "/login")
	}
}
