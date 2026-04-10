<script lang="ts">
	import { CircleCheckBig, Hash, UserRound } from "lucide-svelte"
	import { quizUiStore } from "$lib/features/quiz/quiz.store.svelte"

	const summary = $derived(quizUiStore.attemptSubmittedSummary)
</script>

{#if quizUiStore.isAttemptSubmittedModalOpen && summary}
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
					<CircleCheckBig size={17} class="text-black" />
					Intento enviado
				</h3>
				<button
					class="btn-secondary"
					type="button"
					onclick={quizUiStore.closeAttemptSubmittedModal}
				>
					Cerrar
				</button>
			</div>

			<div class="panel-muted grid gap-4 p-5 sm:p-6">
				<p class="m-0 text-sm leading-relaxed text-zinc-700">
					Tu intento fue enviado correctamente.
				</p>

				<div class="grid gap-3 sm:grid-cols-2">
					<div class="rounded-[6px] border border-zinc-200 bg-white p-4">
						<p
							class="m-0 flex items-center gap-2 text-xs tracking-[0.16em] text-zinc-600 uppercase"
						>
							<UserRound size={13} />
							Estudiante
						</p>
						<p class="mt-2 mb-0 text-base text-black">{summary.studentName}</p>
					</div>

					<div class="rounded-[6px] border border-zinc-200 bg-white p-4">
						<p class="m-0 text-xs tracking-[0.16em] text-zinc-600 uppercase">
							Enviado
						</p>
						<p class="mt-2 mb-0 text-base text-black">{summary.submittedAtLabel}</p>
					</div>
				</div>

				{#if summary.joinCode}
					<div class="rounded-[6px] border border-zinc-200 bg-white p-4">
						<p
							class="m-0 flex items-center gap-2 text-xs tracking-[0.16em] text-zinc-600 uppercase"
						>
							<Hash size={13} />
							Codigo del quiz
						</p>
						<p class="mt-2 mb-0 font-mono text-lg tracking-[0.18em] text-black">
							{summary.joinCode}
						</p>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
