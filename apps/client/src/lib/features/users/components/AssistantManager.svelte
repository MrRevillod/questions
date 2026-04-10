<script lang="ts">
	import { onMount } from "svelte"
	import { Search, Users } from "lucide-svelte"
	import { toast } from "svelte-sonner"
	import { usersService } from "$lib/features/users/users.service"
	import type { ManagedUser } from "$lib/features/users/types"
	import { toUserMessage } from "$lib/shared/errors"

	let isLoading = $state(true)
	let search = $state("")
	let users = $state<ManagedUser[]>([])

	const filteredUsers = $derived.by(() => {
		const query = search.trim().toLowerCase()
		if (!query) {
			return users
		}

		return users.filter(user => user.username.toLowerCase().includes(query))
	})

	const loadUsers = async () => {
		isLoading = true
		const { value, error } = await usersService.listAssistantCandidates()

		if (error) {
			toast.error(toUserMessage(error))
			isLoading = false
			return
		}

		users = value ?? []
		isLoading = false
	}

	const toggleAssistant = async (user: ManagedUser, isAssistant: boolean) => {
		const { value, error } = await usersService.setUserRole({
			userId: user.id,
			role: isAssistant ? "assistant" : "student",
		})

		if (error) {
			toast.error(toUserMessage(error))
			return
		}

		users = users.map(item => (item.id === value.id ? value : item))
		toast.success(`Rol actualizado para ${value.username}.`)
	}

	onMount(loadUsers)
</script>

<section class="panel-surface flex h-full min-h-0 flex-col gap-4 p-4 sm:p-5">
	<div class="flex flex-wrap items-center justify-between gap-3">
		<h3 class="m-0 flex items-center gap-2 text-xl text-black">
			<Users size={18} class="text-black" />
			Gestion de ayudantes
		</h3>
		<button
			class="btn-secondary"
			type="button"
			onclick={loadUsers}
			disabled={isLoading}
		>
			Actualizar
		</button>
	</div>

	<label class="grid gap-1.5">
		<span class="flex items-center gap-1.5 text-sm text-zinc-800">
			<Search size={14} class="text-zinc-700" />
			Buscar por username
		</span>
		<input
			class="input-base"
			type="text"
			bind:value={search}
			placeholder="Ej: lrevillod"
		/>
	</label>

	{#if isLoading}
		<p class="m-0 text-zinc-600">Cargando usuarios...</p>
	{:else if filteredUsers.length === 0}
		<p class="m-0 text-zinc-600">No se encontraron usuarios para el filtro.</p>
	{:else}
		<div class="panel-muted min-h-0 flex-1 overflow-auto">
			<table class="min-w-full border-collapse text-sm">
				<thead class="bg-zinc-100/90 text-zinc-700">
					<tr>
						<th class="px-3 py-2 text-left font-medium">Username</th>
						<th class="px-3 py-2 text-left font-medium">Nombre</th>
						<th class="px-3 py-2 text-left font-medium">Correo</th>
						<th class="px-3 py-2 text-left font-medium">Ayudante</th>
					</tr>
				</thead>
				<tbody>
					{#each filteredUsers as user}
						<tr class="border-t border-zinc-200 bg-white/80">
							<td class="px-3 py-2 font-medium text-zinc-900">{user.username}</td>
							<td class="px-3 py-2 text-zinc-800">{user.name}</td>
							<td class="px-3 py-2 text-zinc-700">{user.email}</td>
							<td class="px-3 py-2">
								<input
									class="h-4 w-4 accent-black"
									type="checkbox"
									checked={user.role === "assistant"}
									onchange={event =>
										toggleAssistant(
											user,
											(event.currentTarget as HTMLInputElement).checked
										)}
								/>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</section>
