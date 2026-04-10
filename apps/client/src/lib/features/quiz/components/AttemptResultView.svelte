<script lang="ts">
	import { BarChart3, CheckCircle2, CircleX, Trophy } from "lucide-svelte"
	import QuestionRichText from "$lib/features/quiz/components/QuestionRichText.svelte"
	import { quizUiStore } from "$lib/features/quiz/quiz.store.svelte"

	const result = $derived((quizUiStore as Record<string, any>).attemptResult ?? null)

	const gradeLabel = $derived.by(() => {
		if (!result) return "1,0"

		return new Intl.NumberFormat("es-CL", {
			minimumFractionDigits: 1,
			maximumFractionDigits: 2,
		}).format(result.grade)
	})

	const pointsLabel = $derived.by(() => {
		if (!result) return "0 / 0"

		const formatter = new Intl.NumberFormat("es-CL", {
			minimumFractionDigits: 0,
			maximumFractionDigits: 2,
		})

		return `${formatter.format(result.scorePoints)} / ${formatter.format(result.scorePointsMax)}`
	})

	const formatDate = (value: string) =>
		new Intl.DateTimeFormat("es-CL", {
			dateStyle: "medium",
			timeStyle: "short",
		}).format(new Date(value))

	const formatPoints = (value: number) =>
		new Intl.NumberFormat("es-CL", {
			minimumFractionDigits: 0,
			maximumFractionDigits: 2,
		}).format(value)

	const getOptionClass = (
		optionIndex: number,
		correctAnswerIndex: number,
		answerIndex: number | null
	) => {
		if (optionIndex === correctAnswerIndex) {
			return "border-emerald-700 bg-emerald-50 text-emerald-900"
		}

		if (
			answerIndex !== null &&
			optionIndex === answerIndex &&
			answerIndex !== correctAnswerIndex
		) {
			return "border-red-700 bg-red-50 text-red-900"
		}

		if (answerIndex !== null && optionIndex === answerIndex) {
			return "border-black bg-black text-white"
		}

		return "border-zinc-300 bg-white text-black"
	}
</script>

{#if result}
	<section class="panel-surface flex h-full min-h-0 flex-col gap-5 p-4 sm:p-5">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<p class="section-kicker m-0">Resultado del intento</p>
				<h3 class="mt-1 mb-0 text-2xl text-black">Correccion final</h3>
				<p class="mt-2 text-sm text-zinc-700">
					Enviado: {formatDate(result.submittedAt)} - Evaluado: {formatDate(
						result.evaluatedAt
					)}
				</p>
			</div>
			<button
				class="btn-secondary"
				type="button"
				onclick={() => quizUiStore.clearAllStores()}
			>
				Volver al panel
			</button>
		</div>

		<div class="grid gap-3 sm:grid-cols-2">
			<div class="panel-muted p-4">
				<p class="m-0 flex items-center gap-2 text-sm font-medium text-zinc-700">
					<Trophy size={15} /> Nota final
				</p>
				<p class="mt-2 mb-0 text-3xl font-semibold text-black">{gradeLabel}</p>
			</div>
			<div class="panel-muted p-4">
				<p class="m-0 flex items-center gap-2 text-sm font-medium text-zinc-700">
					<BarChart3 size={15} /> Puntaje
				</p>
				<p class="mt-2 mb-0 text-3xl font-semibold text-black">{pointsLabel}</p>
			</div>
		</div>

		<div class="min-h-0 space-y-4 overflow-auto pr-1">
			{#each result.questions as question, index}
				<article class="panel-muted p-4 sm:p-5">
					<div class="flex flex-wrap items-start justify-between gap-2">
						<p class="m-0 text-sm font-medium text-zinc-700">Pregunta {index + 1}</p>
						{#if question.isCorrect}
							<span class="inline-flex items-center gap-1 text-sm text-emerald-700">
								<CheckCircle2 size={14} /> Correcta
							</span>
						{:else}
							<span class="inline-flex items-center gap-1 text-sm text-red-700">
								<CircleX size={14} /> Incorrecta
							</span>
						{/if}
					</div>

					<div class="mt-3 text-base text-black sm:text-lg">
						<QuestionRichText text={question.question} />
					</div>

					{#if question.images.length > 0}
						<div class="mt-4 grid gap-3 sm:grid-cols-2">
							{#each question.images as imageUrl}
								<img
									class="w-full rounded-[4px] border border-zinc-300 bg-white"
									src={imageUrl}
									alt="Imagen de apoyo"
								/>
							{/each}
						</div>
					{/if}

					<div class="mt-4 grid gap-2.5">
						{#each question.options as option, optionIndex}
							<div
								class={`rounded-[4px] border px-4 py-3 text-left text-base leading-relaxed ${getOptionClass(optionIndex, question.correctAnswerIndex, question.answerIndex)}`}
							>
								<QuestionRichText text={option} />
							</div>
						{/each}
					</div>

					<p class="mt-4 mb-0 text-sm text-zinc-700">
						Tu respuesta: {question.answerIndex === null
							? "-"
							: question.answerIndex + 1} - Respuesta correcta:
						{question.correctAnswerIndex + 1} - Puntos: {formatPoints(
							question.awardedPoints
						)}
					</p>
				</article>
			{/each}
		</div>
	</section>
{/if}
