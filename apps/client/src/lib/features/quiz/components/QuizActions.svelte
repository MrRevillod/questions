<script lang="ts">
	import { FilePlus2, ListChecks, LogIn, Users } from "lucide-svelte"
	import { authStore } from "$lib/features/auth/auth.store.svelte"
	import { quizUiStore } from "$lib/features/quiz/quiz.store.svelte"

	const canManageQuizzes = $derived.by(() => {
		const role = authStore.session?.user.role
		return role === "func" || role === "assistant"
	})

	const isTeacher = $derived(authStore.session?.user.role === "func")
</script>

<div class="flex flex-wrap gap-3">
	<button
		class="action-tab"
		data-active={quizUiStore.activePanel === "join"}
		type="button"
		onclick={() => quizUiStore.setPanel("join")}
	>
		<LogIn size={16} class="mr-1 inline" />
		Unirse a un quiz
	</button>
	{#if canManageQuizzes}
		<button
			class="action-tab"
			data-active={quizUiStore.activePanel === "create"}
			type="button"
			onclick={() => quizUiStore.setPanel("create")}
		>
			<FilePlus2 size={16} class="mr-1 inline" />
			Crear un quiz
		</button>
		<button
			class="action-tab"
			data-active={quizUiStore.activePanel === "mine"}
			type="button"
			onclick={() => quizUiStore.setPanel("mine")}
		>
			<ListChecks size={16} class="mr-1 inline" />
			Mis quizzes
		</button>
	{/if}

	{#if isTeacher}
		<button
			class="action-tab"
			data-active={quizUiStore.activePanel === "assistants"}
			type="button"
			onclick={() => quizUiStore.setPanel("assistants")}
		>
			<Users size={16} class="mr-1 inline" />
			Ayudantes
		</button>
	{/if}
</div>
