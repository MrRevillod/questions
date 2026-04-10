<script lang="ts">
	import { onMount } from 'svelte'
	import { CheckCircle2, Clock3, Eye, RefreshCcw, UserRoundCheck } from 'lucide-svelte'
	import { toast } from 'svelte-sonner'
	import { quizService } from '$lib/features/quiz/quiz.service'
	import { quizUiStore } from '$lib/features/quiz/quiz.store.svelte'
	import type { AppError } from '$lib/shared/errors'
	import { toUserMessage } from '$lib/shared/errors'

	type ManagedAttemptSummary = {
		attemptId: string
		quizId: string
		studentId: string
		studentName: string
		studentUsername: string
		startedAt: string
		expiresAt: string
		submittedAt: string | null
		scorePoints: number | null
		scorePointsMax: number | null
		grade: number | null
		resultsReleasedAt: string | null
		resultsViewedAt: string | null
	}

	type FinalizeAndPublishSummary = {
		quizId: string
		finalizedAttempts: number
		publishedAttempts: number
	}

	const panel = $derived((quizUiStore as unknown as Record<string, any>).managedAttemptsPanel ?? null)
	const attempts = $derived(
		((quizUiStore as unknown as Record<string, any>).managedAttempts ?? []) as ManagedAttemptSummary[]
	)

	let isLoading = $state(false)

	let isFinalizingAndPublishing = $state(false)

	const formatDate = (value: string | null) => {
		if (!value) {
			return '-'
		}

		return new Intl.DateTimeFormat('es-CL', {
			dateStyle: 'medium',
			timeStyle: 'short'
		}).format(new Date(value))
	}

	const formatGrade = (value: number | null) => {
		if (value === null) {
			return '-'
		}

		return new Intl.NumberFormat('es-CL', {
			minimumFractionDigits: 1,
			maximumFractionDigits: 2
		}).format(value)
	}

	const loadAttempts = async () => {
		if (!panel) {
			return
		}

		isLoading = true
		const { value, error } = await (quizService as Record<string, any>).getManagedAttempts(
			panel.quizId
		)

		if (error) {
			toast.error(toUserMessage(error))
			isLoading = false
			return
		}

		;(quizUiStore as Record<string, any>).setManagedAttempts(value ?? [])
		isLoading = false
	}

	const handleFinalizeAndPublish = async () => {
		if (!panel) {
			return
		}

		const confirmed = window.confirm(
			'Esta accion cerrara el quiz, finalizara intentos en progreso y publicara resultados para todos los intentos enviados.\n\nLuego nadie podra unirse al quiz.'
		)

		if (!confirmed) {
			return
		}

		isFinalizingAndPublishing = true

		const response = await (quizService as Record<string, any>).finalizeAndPublish(
			panel.quizId
		)

		const { value, error } = response as {
			value: FinalizeAndPublishSummary | null
			error: AppError | null
		}

		isFinalizingAndPublishing = false

		if (error) {
			toast.error(toUserMessage(error))
			return
		}

		if (value) {
			toast.success(
				`Quiz cerrado. Intentos finalizados: ${value.finalizedAttempts}. Resultados publicados: ${value.publishedAttempts}.`
			)
		}

		await loadAttempts()
	}

	const getResultStatus = (attempt: ManagedAttemptSummary) => {
		if (attempt.resultsViewedAt) {
			return 'Visto'
		}

		if (attempt.resultsReleasedAt) {
			return 'Liberado'
		}

		return 'Pendiente'
	}

	const isInProgress = (attempt: ManagedAttemptSummary) => attempt.submittedAt === null

	const getAttemptStatusLabel = (attempt: ManagedAttemptSummary) =>
		isInProgress(attempt) ? 'En progreso' : 'Enviado'

	onMount(() => {
		void loadAttempts()
	})
</script>

{#if panel}
	<section class="panel-surface flex h-full min-h-0 flex-col gap-4 p-4 sm:p-5">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<div>
				<p class="section-kicker m-0">Gestion de intentos</p>
				<h3 class="mt-1 mb-0 text-xl text-black">{panel.title}</h3>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				<button
					class="btn-primary"
					type="button"
					onclick={handleFinalizeAndPublish}
					disabled={isLoading || isFinalizingAndPublishing}
				>
					{isFinalizingAndPublishing
						? 'Finalizando y publicando...'
						: 'Finalizar y publicar resultados'}
				</button>
				<button class="btn-secondary" type="button" onclick={loadAttempts} disabled={isLoading}>
					<RefreshCcw size={14} class="mr-1 inline" />
					Actualizar
				</button>
				<button
					class="btn-tertiary"
					type="button"
					onclick={() => (quizUiStore as unknown as Record<string, any>).closeManagedAttemptsPanel()}
				>
					Volver
				</button>
			</div>
		</div>

		{#if isLoading}
			<p class="m-0 text-sm text-zinc-600">Cargando intentos...</p>
		{:else if attempts.length === 0}
			<p class="m-0 text-sm text-zinc-600">Aun no hay intentos para este quiz.</p>
		{:else}
			<div class="min-h-0 overflow-auto">
				<table class="w-full border-collapse text-sm">
					<thead>
						<tr class="text-left text-zinc-700">
							<th class="border border-zinc-300 bg-white p-2">Estudiante</th>
							<th class="border border-zinc-300 bg-white p-2">Estado</th>
							<th class="border border-zinc-300 bg-white p-2">Inicio</th>
							<th class="border border-zinc-300 bg-white p-2">Entrega</th>
							<th class="border border-zinc-300 bg-white p-2">Nota</th>
							<th class="border border-zinc-300 bg-white p-2">Resultados</th>
							<th class="border border-zinc-300 bg-white p-2">Accion</th>
						</tr>
					</thead>
					<tbody>
						{#each attempts as attempt}
							<tr>
								<td class="border border-zinc-300 bg-white p-2 text-zinc-800">
									<div class="flex items-center gap-2">
										<UserRoundCheck size={14} class="text-zinc-500" />
										<div>
											<p class="m-0">{attempt.studentName}</p>
											<p class="m-0 text-xs text-zinc-500">@{attempt.studentUsername}</p>
										</div>
									</div>
								</td>
								<td class="border border-zinc-300 bg-white p-2 text-zinc-800">
									{#if isInProgress(attempt)}
										<span class="inline-flex items-center gap-1 text-amber-700">
											<Clock3 size={14} /> {getAttemptStatusLabel(attempt)}
										</span>
									{:else}
										<span class="text-emerald-700">{getAttemptStatusLabel(attempt)}</span>
									{/if}
								</td>
								<td class="border border-zinc-300 bg-white p-2 text-zinc-800">{formatDate(attempt.startedAt)}</td>
								<td class="border border-zinc-300 bg-white p-2 text-zinc-800">{formatDate(attempt.submittedAt)}</td>
								<td class="border border-zinc-300 bg-white p-2 text-zinc-800">{formatGrade(attempt.grade)}</td>
								<td class="border border-zinc-300 bg-white p-2 text-zinc-800">
									{#if attempt.resultsViewedAt}
										<span class="inline-flex items-center gap-1 text-zinc-700">
											<Eye size={14} /> {getResultStatus(attempt)}
										</span>
									{:else if attempt.resultsReleasedAt}
										<span class="inline-flex items-center gap-1 text-emerald-700">
											<CheckCircle2 size={14} /> {getResultStatus(attempt)}
										</span>
									{:else}
										<span class="text-amber-700">{getResultStatus(attempt)}</span>
									{/if}
								</td>
								<td class="border border-zinc-300 bg-white p-2">
									<span class="text-xs text-zinc-500">Usa accion global</span>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
{/if}
