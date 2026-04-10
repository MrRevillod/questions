<script lang="ts">
	import { Code2 } from "lucide-svelte"
	import { QUESTION_FORMAT_EXAMPLE } from "$lib/features/quiz/constants"
	import { quizUiStore } from "$lib/features/quiz/quiz.store.svelte"
</script>

{#if quizUiStore.isFormatModalOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4 backdrop-blur-[1px]"
	>
		<div
			class="panel-surface w-full max-w-3xl p-5 sm:p-6"
			role="dialog"
			aria-modal="true"
		>
			<div class="mb-3 flex items-center justify-between gap-3">
				<h3 class="m-0 flex items-center gap-2 text-lg text-black">
					<Code2 size={17} class="text-black" />
					Formato JSON de preguntas
				</h3>
				<button
					class="btn-secondary"
					type="button"
					onclick={quizUiStore.closeFormatModal}
				>
					Cerrar
				</button>
			</div>

			<pre
				class="panel-muted max-h-[55vh] overflow-auto p-4 text-sm leading-relaxed text-zinc-800"><code
					>{QUESTION_FORMAT_EXAMPLE}</code
				></pre>

			<p class="mt-3 text-sm text-zinc-700">
				`answer` es el indice de `options`. Si `images` contiene URLs, las imagenes
				deben renderizarse en fila bajo el texto de la pregunta. Puedes incluir HTML
				basico en `question` y `options`, y ecuaciones LaTeX inline usando
				`\\(...\\)`.
			</p>
		</div>
	</div>
{/if}
