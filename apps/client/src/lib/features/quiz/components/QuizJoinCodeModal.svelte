<script lang="ts">
	import { Check, Copy, Hash } from "lucide-svelte"
	import { quizUiStore } from "$lib/features/quiz/quiz.store.svelte"

	const joinCode = $derived(quizUiStore.createdQuizJoinCode)
	let hasCopied = $state(false)
	let copiedTimer: ReturnType<typeof setTimeout> | null = null

	const copyCode = async () => {
		if (!joinCode) {
			return
		}

		await navigator.clipboard.writeText(joinCode)
		hasCopied = true

		if (copiedTimer) {
			clearTimeout(copiedTimer)
		}

		copiedTimer = setTimeout(() => {
			hasCopied = false
			copiedTimer = null
		}, 1800)
	}

	$effect(() => {
		return () => {
			if (copiedTimer) {
				clearTimeout(copiedTimer)
			}
		}
	})
</script>

{#if quizUiStore.isJoinCodeModalOpen && joinCode}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4 backdrop-blur-[1px]"
	>
		<div
			class="panel-surface w-full max-w-xl p-5 sm:p-6"
			role="dialog"
			aria-modal="true"
		>
			<div class="mb-4 flex items-center justify-between gap-3">
				<h3 class="m-0 flex items-center gap-2 text-lg text-black">
					<Hash size={17} class="text-black" />
					Codigo de union del quiz
				</h3>
				<button
					class="btn-secondary"
					type="button"
					onclick={quizUiStore.closeJoinCodeModal}
				>
					Cerrar
				</button>
			</div>

			<div
				class="panel-muted flex flex-col items-center gap-4 p-5 text-center sm:p-6"
			>
				<p class="m-0 text-sm leading-relaxed text-zinc-700">
					Comparte este codigo con tus estudiantes para que puedan unirse al quiz.
				</p>
				<div
					class="rounded-[6px] border border-zinc-300 bg-white px-5 py-4 font-mono text-3xl tracking-[0.24em] text-black sm:text-4xl"
				>
					{joinCode}
				</div>
				<button class="btn-primary" type="button" onclick={copyCode}>
					{#if hasCopied}
						<Check size={14} class="mr-1 inline" />
						Copiado
					{:else}
						<Copy size={14} class="mr-1 inline" />
						Copiar codigo
					{/if}
				</button>
			</div>
		</div>
	</div>
{/if}
