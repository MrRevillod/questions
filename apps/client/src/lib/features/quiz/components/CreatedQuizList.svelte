<script lang="ts">
	import { onMount } from "svelte"
	import { Clipboard, ListChecks, RotateCw, UsersRound } from "lucide-svelte"
	import { toast } from "svelte-sonner"
	import { quizService } from "$lib/features/quiz/quiz.service"
	import { quizUiStore } from "$lib/features/quiz/quiz.store.svelte"
	import type { QuizSummary } from "$lib/features/quiz/types"
	import { toUserMessage } from "$lib/shared/errors"

	let quizzes = $state<QuizSummary[]>([])
	let isLoading = $state(true)

	const formatDate = (value: string) =>
		new Intl.DateTimeFormat("es-CL", {
			dateStyle: "medium",
			timeStyle: "short",
		}).format(new Date(value))

	const getQuizStatus = (closedAt: string | null) =>
		closedAt ? "Cerrado" : "Abierto"

	const copyCode = async (code: string) => {
		await navigator.clipboard.writeText(code)
		toast.success("Codigo copiado al portapapeles.")
	}

	const openAttempts = (quiz: QuizSummary) => {
		;(quizUiStore as Record<string, any>).openManagedAttemptsPanel(
			quiz.id,
			quiz.title
		)
	}

	const getClosedAt = (quiz: QuizSummary) =>
		(quiz as unknown as { closedAt?: string | null }).closedAt ?? null

	const loadQuizzes = async () => {
		isLoading = true
		const { value, error } = await quizService.getMyQuizzes()

		if (error) {
			toast.error(toUserMessage(error))
			isLoading = false
			return
		}

		quizzes = value ?? []
		isLoading = false
	}

	onMount(loadQuizzes)
</script>

<section class="panel-surface flex h-full min-h-0 flex-col gap-4 p-4 sm:p-5">
	<div class="flex items-center justify-between gap-3">
		<h3 class="m-0 flex items-center gap-2 text-xl text-black">
			<ListChecks size={18} class="text-black" />
			Quizzes creados
		</h3>
		<button
			class="btn-secondary"
			type="button"
			onclick={loadQuizzes}
			disabled={isLoading}
		>
			<RotateCw size={14} class="mr-1 inline" />
			Actualizar
		</button>
	</div>

	{#if isLoading}
		<p class="m-0 text-zinc-600">Cargando quizzes...</p>
	{:else if quizzes.length === 0}
		<p class="m-0 text-zinc-600">Aun no tienes quizzes creados.</p>
	{:else}
		<div class="min-h-0 overflow-y-auto pr-1">
			<div class="grid gap-3">
				{#each quizzes as quiz}
					<article class="panel-muted p-4 sm:p-5">
						<div class="flex flex-wrap items-start justify-between gap-3">
							<div class="space-y-1.5">
								<p class="m-0 text-lg leading-tight font-semibold text-black">
									{quiz.title}
								</p>
								<p class="m-0 text-sm text-zinc-700">
									{quiz.kind === "Certainly" ? "Certeza" : "Tradicional"} - {quiz.questionCount}{" "}
									preguntas
								</p>
								<p class="m-0 text-sm text-zinc-600">
									Inicio: {formatDate(quiz.startTime)}
								</p>
								<p class="m-0 text-sm text-zinc-600">
									Duracion: {quiz.attemptDurationMinutes} min - Creado: {formatDate(
										quiz.createdAt
									)}
								</p>
								<p class="m-0 text-sm text-zinc-700">
									Estado: {getQuizStatus(getClosedAt(quiz))}
								</p>
							</div>

							<div class="flex flex-wrap items-center gap-2">
								<span class="code-chip">
									{quiz.joinCode}
								</span>
								<button
									class="btn-secondary"
									type="button"
									onclick={() => openAttempts(quiz)}
								>
									<UsersRound size={14} class="mr-1 inline" />
									Intentos
								</button>
								<button
									class="btn-primary"
									type="button"
									onclick={() => copyCode(quiz.joinCode)}
								>
									<Clipboard size={14} class="mr-1 inline" />
									Copiar
								</button>
							</div>
						</div>
					</article>
				{/each}
			</div>
		</div>
	{/if}
</section>
