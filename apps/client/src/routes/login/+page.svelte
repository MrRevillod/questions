<script lang="ts">
	import { createMutation } from "@tanstack/svelte-query"
	import { goto } from "$app/navigation"
	import { toast } from "svelte-sonner"
	import { authService } from "$lib/features/auth/auth.service"
	import type { LoginInput } from "$lib/features/auth/types"
	import { toUserMessage } from "$lib/shared/errors"

	let username = $state("")
	let password = $state("")

	const loginMutation = createMutation(() => ({
		mutationFn: (payload: LoginInput) => authService.login(payload),
	}))

	const loading = $derived(loginMutation.isPending)

	const handleSubmit = async (event: SubmitEvent) => {
		event.preventDefault()

		const result = await loginMutation.mutateAsync({ username, password })

		if (result.value) {
			await goto("/")
			return
		}

		toast.error(toUserMessage(result.error))
	}
</script>

<main class="min-h-dvh px-4 py-8 sm:px-6 sm:py-12">
	<div
		class="mx-auto flex min-h-[calc(100dvh-4rem)] max-w-5xl items-center justify-center"
	>
		<section class="panel-surface w-full max-w-xl p-8 sm:p-10">
			<p class="section-kicker">Acceso</p>
			<h1 class="m-0 mt-2 text-3xl leading-tight text-black sm:text-[2.15rem]">
				Iniciar sesión
			</h1>
			<p class="mt-3 mb-7 max-w-md text-base leading-relaxed text-zinc-700">
				Usa tus credenciales Pillan/LDAP para continuar.
			</p>

			<form class="grid gap-4" onsubmit={handleSubmit}>
				<label class="grid gap-1.5">
					<span class="text-sm text-zinc-800">Usuario</span>
					<input
						class="input-base"
						type="text"
						bind:value={username}
						required
						autocomplete="username"
					/>
				</label>

				<label class="grid gap-1.5">
					<span class="text-sm text-zinc-800">Contraseña</span>
					<input
						class="input-base"
						type="password"
						bind:value={password}
						required
						autocomplete="current-password"
					/>
				</label>

				<button
					class="btn-primary mt-2 w-full text-base"
					type="submit"
					disabled={loading}
				>
					{loading ? "Ingresando..." : "Ingresar"}
				</button>
			</form>
		</section>
	</div>
</main>
