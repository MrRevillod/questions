<script lang="ts">
	import { onMount } from "svelte"
	import { createMutation } from "@tanstack/svelte-query"
	import { toast } from "svelte-sonner"
	import { ChevronRight, Send } from "lucide-svelte"
	import { authStore } from "$lib/features/auth/auth.store.svelte"
	import { attemptService } from "$lib/features/attempt/attempt.service"
	import { quizService } from "$lib/features/quiz/quiz.service"
	import { quizUiStore } from "$lib/features/quiz/quiz.store.svelte"
	import QuestionRichText from "$lib/features/quiz/components/QuestionRichText.svelte"
	import { toUserMessage, type AppError } from "$lib/shared/errors"
	import type { AttemptCertaintyLevel } from "$lib/features/quiz/types"

	const activeAttempt = $derived(quizUiStore.activeAttempt)
	const activeAttemptId = $derived(activeAttempt?.attemptId ?? null)
	const activeQuiz = $derived(activeAttempt?.quiz ?? null)
	const currentQuestion = $derived.by(() => {
		if (!activeQuiz) {
			return null
		}

		return activeQuiz.questions[quizUiStore.currentQuestionIndex] ?? null
	})

	const selectedAnswer = $derived.by(() => {
		if (!currentQuestion || !activeAttempt) {
			return undefined
		}

		return activeAttempt.answers.find(
			answer => answer.questionId === currentQuestion.questionId
		)?.answerIndex
	})
	const selectedCertainty = $derived.by(() => {
		if (!currentQuestion || !activeAttempt) {
			return null
		}

		return (
			activeAttempt.answers.find(
				answer => answer.questionId === currentQuestion.questionId
			)?.certaintyLevel ?? null
		)
	})
	const isCertaintyQuiz = $derived(activeQuiz?.kind === "Certainly")
	const canContinueCurrentQuestion = $derived.by(() => {
		if (selectedAnswer === undefined) {
			return false
		}

		if (isCertaintyQuiz && !selectedCertainty) {
			return false
		}

		return true
	})

	const totalQuestions = $derived(activeQuiz?.questions.length ?? 0)
	const answeredCount = $derived(activeAttempt?.answers.length ?? 0)
	const isLastQuestion = $derived(
		quizUiStore.currentQuestionIndex >= totalQuestions - 1
	)
	let now = $state(Date.now())
	const progress = $derived.by(() => {
		if (!totalQuestions) {
			return 0
		}

		return Math.round((answeredCount / totalQuestions) * 100)
	})
	const remainingMs = $derived.by(() => {
		if (!activeAttempt) {
			return 0
		}

		return Math.max(new Date(activeAttempt.expiresAt).getTime() - now, 0)
	})
	const remainingLabel = $derived.by(() => {
		const totalSeconds = Math.floor(remainingMs / 1000)
		const minutes = Math.floor(totalSeconds / 60)
		const seconds = totalSeconds % 60

		return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`
	})
	const isExpired = $derived(remainingMs <= 0)

	let autoSubmitStarted = $state(false)
	let submissionStarted = $state(false)
	let isRevalidatingAttempt = $state(true)
	let saveInFlightByQuestionId = $state<Record<string, boolean>>({})

	onMount(() => {
		const interval = window.setInterval(() => {
			now = Date.now()
		}, 1000)

		return () => {
			window.clearInterval(interval)
		}
	})

	$effect(() => {
		if (activeAttemptId) {
			isRevalidatingAttempt = true
			autoSubmitStarted = false
			submissionStarted = false
			return
		}

		isRevalidatingAttempt = true
		autoSubmitStarted = false
		submissionStarted = false
	})

	const isTerminalAttemptError = (error: AppError | null) => {
		if (!error) {
			return false
		}

		if (
			error.status === 403 ||
			error.status === 404 ||
			error.status === 409 ||
			error.status === 410
		) {
			return true
		}

		const message = error.message.toLowerCase()

		return (
			message.includes("forbidden") ||
			message.includes("expired") ||
			message.includes("expir") ||
			message.includes("already submitted") ||
			message.includes("ya fue entregado") ||
			message.includes("not found")
		)
	}

	const closeAttemptSilently = () => {
		quizUiStore.leaveQuizAttempt()
	}

	const revalidateAttempt = async () => {
		if (!activeAttempt) {
			isRevalidatingAttempt = false
			return
		}

		const { value, error } = await quizService.getMyActiveAttempt(
			activeAttempt.quiz.id
		)

		if (error) {
			if (isTerminalAttemptError(error)) {
				closeAttemptSilently()
				return
			}

			toast.error(toUserMessage(error))
			closeAttemptSilently()
			return
		}

		quizUiStore.syncActiveAttempt(value)
		isRevalidatingAttempt = false
	}

	onMount(() => {
		void revalidateAttempt()
	})

	const submitMutation = createMutation(() => ({
		mutationFn: (attemptId: string) => attemptService.submitAttempt(attemptId),
	}))

	const upsertLocalAnswer = (
		questionId: string,
		answerIndex: number,
		certaintyLevel?: AttemptCertaintyLevel | null
	) => {
		quizUiStore.upsertAnswer({
			questionId,
			answerIndex,
			certaintyLevel: certaintyLevel ?? null,
		})
	}

	const saveAnswer = async (
		questionId: string,
		answerIndex: number,
		certaintyLevel?: AttemptCertaintyLevel | null
	) => {
		if (!activeAttempt || !activeQuiz || isExpired || submissionStarted) {
			return
		}

		upsertLocalAnswer(questionId, answerIndex, certaintyLevel)

		saveInFlightByQuestionId = {
			...saveInFlightByQuestionId,
			[questionId]: true,
		}

		const { error } = await attemptService.saveAnswer(
			activeAttempt.attemptId,
			questionId,
			{
				answerIndex,
				certaintyLevel,
			}
		)

		saveInFlightByQuestionId = {
			...saveInFlightByQuestionId,
			[questionId]: false,
		}

		if (error) {
			if (isTerminalAttemptError(error)) {
				closeAttemptSilently()
				return
			}

			toast.error(toUserMessage(error))
		}
	}

	const handleOptionSelect = (optionIndex: number) => {
		if (!currentQuestion) {
			return
		}

		if (!isCertaintyQuiz) {
			void saveAnswer(currentQuestion.questionId, optionIndex)
			return
		}

		if (selectedCertainty) {
			void saveAnswer(currentQuestion.questionId, optionIndex, selectedCertainty)
			return
		}

		upsertLocalAnswer(currentQuestion.questionId, optionIndex, null)
	}

	const handleCertaintySelect = (level: AttemptCertaintyLevel) => {
		if (!currentQuestion || selectedAnswer === undefined) {
			return
		}

		void saveAnswer(currentQuestion.questionId, selectedAnswer, level)
	}

	const handleFinish = async () => {
		if (!activeAttempt || submissionStarted) {
			return
		}

		submissionStarted = true

		const { error } = await submitMutation.mutateAsync(activeAttempt.attemptId)

		if (error) {
			if (isTerminalAttemptError(error)) {
				closeAttemptSilently()
				return
			}

			submissionStarted = false
			toast.error(toUserMessage(error))
			return
		}

		const submittedAtLabel = new Intl.DateTimeFormat("es-CL", {
			dateStyle: "medium",
			timeStyle: "short",
		}).format(new Date())

		quizUiStore.openAttemptSubmittedModal({
			studentName:
				authStore.session?.user.name ??
				authStore.session?.user.username ??
				"Estudiante",
			joinCode: quizUiStore.participantJoinCode,
			submittedAtLabel,
		})
		closeAttemptSilently()
	}

	$effect(() => {
		if (!isExpired || autoSubmitStarted || !activeAttempt) {
			return
		}

		autoSubmitStarted = true
		void handleFinish()
	})
</script>

{#if activeAttempt && activeQuiz && currentQuestion && isRevalidatingAttempt}
	<section
		class="panel-surface flex h-full min-h-0 items-center justify-center p-4 sm:p-5"
	>
		<p class="m-0 text-sm text-zinc-600">Cargando intento...</p>
	</section>
{:else if activeAttempt && activeQuiz && currentQuestion}
	<section class="panel-surface flex h-full min-h-0 flex-col gap-4 p-4 sm:p-5">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h3 class="m-0 text-xl text-black">{activeQuiz.title}</h3>
				<p class="mt-1 text-sm leading-relaxed text-zinc-700">
					Pregunta {quizUiStore.currentQuestionIndex + 1} de {totalQuestions} - Respondidas:
					{answeredCount}
				</p>
				<div class="mt-3 w-full max-w-xs">
					<div class="h-2 overflow-hidden rounded-full bg-zinc-200">
						<div
							class="h-full rounded-full bg-black transition-[width] duration-200"
							style={`width: ${progress}%`}
						></div>
					</div>
					<p class="mt-1 text-xs text-zinc-600">{progress}% completado</p>
				</div>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				<span class="code-chip">{remainingLabel}</span>
				<button
					class="btn-secondary"
					type="button"
					onclick={() => quizUiStore.leaveQuizAttempt()}
				>
					Salir
				</button>
			</div>
		</div>

		<article class="panel-muted min-h-0 flex-1 overflow-auto p-4 sm:p-5">
			<div class="m-0 text-lg text-black sm:text-xl">
				<QuestionRichText text={currentQuestion.question} />
			</div>

			{#if currentQuestion.images.length > 0}
				<div class="mt-4 grid gap-3 sm:grid-cols-2">
					{#each currentQuestion.images as imageUrl (imageUrl)}
						<img
							class="w-full rounded-sm border border-zinc-300 bg-white"
							src={imageUrl}
							alt="Imagen de apoyo"
						/>
					{/each}
				</div>
			{/if}

			<div class="mt-5 grid gap-2.5">
				{#each currentQuestion.options as option, optionIndex (`${optionIndex}:${option}`)}
					<button
						class={`rounded-sm border px-4 py-3 text-left text-base leading-relaxed transition ${
							selectedAnswer === optionIndex
								? "border-black bg-black text-white shadow-[0_10px_20px_rgba(0,0,0,0.08)]"
								: "border-zinc-300 bg-white text-black hover:border-zinc-400 hover:bg-zinc-50"
						}`}
						type="button"
						onclick={() => handleOptionSelect(optionIndex)}
						disabled={isExpired || submitMutation.isPending || submissionStarted}
					>
						<QuestionRichText text={option} />
					</button>
				{/each}
			</div>

			{#if isCertaintyQuiz}
				<div class="mt-5 grid gap-3 border-t border-zinc-200 pt-5">
					<div class="space-y-1">
						<p class="m-0 text-sm font-medium text-black">Nivel de certeza</p>
						<p class="m-0 text-sm text-zinc-600">
							Indica cuanta seguridad tienes en la alternativa seleccionada.
						</p>
					</div>
					<div class="grid gap-2 sm:grid-cols-3">
						{#each [{ level: "low", label: "Baja" }, { level: "medium", label: "Media" }, { level: "high", label: "Alta" }] as item (item.level)}
							<button
								class={`rounded-sm border px-4 py-3 text-left transition ${
									selectedCertainty === item.level
										? "border-black bg-black text-white shadow-[0_10px_20px_rgba(0,0,0,0.08)]"
										: "border-zinc-300 bg-white text-black hover:border-zinc-400 hover:bg-zinc-50"
								}`}
								type="button"
								onclick={() =>
									handleCertaintySelect(item.level as AttemptCertaintyLevel)}
								disabled={selectedAnswer === undefined ||
									isExpired ||
									submitMutation.isPending ||
									submissionStarted}
							>
								<span class="block text-sm font-medium">{item.label}</span>
							</button>
						{/each}
					</div>
				</div>
			{/if}
		</article>

		<div class="flex flex-wrap items-center justify-between gap-3">
			<p class="m-0 text-sm text-zinc-600">
				{#if isExpired}
					Tiempo agotado.
				{:else if selectedAnswer === undefined}
					Selecciona una alternativa para continuar.
				{:else if isCertaintyQuiz && !selectedCertainty}
					Selecciona tu nivel de certeza para continuar.
				{:else if saveInFlightByQuestionId[currentQuestion.questionId]}
					Guardando respuesta...
				{:else}
					Respuesta guardada.
				{/if}
			</p>
			<div class="flex flex-wrap items-center gap-2">
				{#if !isLastQuestion}
					<button
						class="btn-secondary"
						type="button"
						onclick={() =>
							quizUiStore.goToQuestion(quizUiStore.currentQuestionIndex + 1)}
						disabled={saveInFlightByQuestionId[currentQuestion.questionId] ||
							!canContinueCurrentQuestion ||
							submissionStarted}
					>
						Siguiente
						<ChevronRight size={14} class="ml-1 inline" />
					</button>
				{:else}
					<button
						class="btn-primary"
						type="button"
						onclick={handleFinish}
						disabled={submitMutation.isPending ||
							isExpired ||
							submissionStarted ||
							(selectedAnswer !== undefined && !canContinueCurrentQuestion)}
					>
						<Send size={14} class="mr-1 inline" />
						{submitMutation.isPending ? "Entregando..." : "Finalizar intento"}
					</button>
				{/if}
			</div>
		</div>
	</section>
{/if}
