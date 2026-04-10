<script lang="ts">
	import { onMount } from "svelte"
	import { createMutation } from "@tanstack/svelte-query"
	import { ClipboardCheck, Play, TimerReset } from "lucide-svelte"
	import { toast } from "svelte-sonner"
	import { quizService } from "$lib/features/quiz/quiz.service"
	import { quizUiStore } from "$lib/features/quiz/quiz.store.svelte"
	import { toUserMessage } from "$lib/shared/errors"
	import type { CertaintyConfig } from "$lib/features/quiz/types"

	type PreviewWithOptionalCertaintyTable = {
		certaintyTable?: CertaintyConfig | null
		certainlyTable?: CertaintyConfig | null
	}

	const preview = $derived(quizUiStore.joinPreview)
	let now = $state(Date.now())
	const certaintyTable = $derived.by(() => {
		const currentPreview = preview as PreviewWithOptionalCertaintyTable | null

		return currentPreview?.certaintyTable ?? currentPreview?.certainlyTable ?? null
	})

	const startAttemptMutation = createMutation(() => ({
		mutationFn: (quizId: string) => quizService.startAttempt(quizId),
	}))
	const canStart = $derived.by(() => {
		if (!preview) {
			return false
		}

		return new Date(preview.startTime).getTime() <= now
	})

	onMount(() => {
		const interval = window.setInterval(() => {
			now = Date.now()
		}, 1000)

		return () => {
			window.clearInterval(interval)
		}
	})

	const formatDate = (value: string) =>
		new Intl.DateTimeFormat("es-CL", {
			dateStyle: "medium",
			timeStyle: "short",
		}).format(new Date(value))

	const handleStart = async () => {
		if (!preview || !canStart) {
			return
		}

		const { value, error } = await startAttemptMutation.mutateAsync(preview.id)

		if (error) {
			toast.error(toUserMessage(error))
			return
		}

		quizUiStore.startQuizAttempt(value)
	}
</script>

{#if preview}
	<section class="panel-surface flex h-full min-h-0 flex-col gap-5 p-6 sm:p-7">
		<div class="space-y-2">
			<p class="section-kicker m-0">Instrucciones</p>
			<h3 class="m-0 text-2xl text-black">{preview.title}</h3>
			<p class="max-w-3xl text-sm leading-relaxed text-zinc-700 sm:text-base">
				Revisa los detalles antes de comenzar. Cuando inicies, responde cada pregunta
				y finaliza al terminar.
			</p>
		</div>

		<div class="grid gap-3 sm:grid-cols-3">
			<div class="panel-muted p-4">
				<p class="m-0 text-xs tracking-[0.16em] text-zinc-600 uppercase">Tipo</p>
				<p class="mt-2 text-lg text-black">
					{preview.kind === "Certainly" ? "Certeza" : "Tradicional"}
				</p>
			</div>
			<div class="panel-muted p-4">
				<p class="m-0 text-xs tracking-[0.16em] text-zinc-600 uppercase">
					Preguntas
				</p>
				<p class="mt-2 text-lg text-black">{preview.questionCount}</p>
			</div>
			<div class="panel-muted p-4">
				<p class="m-0 text-xs tracking-[0.16em] text-zinc-600 uppercase">Duracion</p>
				<p class="mt-2 text-lg text-black">{preview.attemptDurationMinutes} min</p>
			</div>
		</div>

		<div class="panel-muted space-y-3 p-5">
			<p class="m-0 flex items-center gap-2 text-base font-medium text-black">
				<ClipboardCheck size={16} />
				Antes de comenzar
			</p>
			<ul class="space-y-2 pl-5 text-sm leading-relaxed text-zinc-700">
				<li>Inicio programado: {formatDate(preview.startTime)}</li>
				<li>Solo tienes un intento total para este quiz.</li>
				<li>El intento se entrega automaticamente si se agota el tiempo.</li>
			</ul>
		</div>

		{#if preview.kind === "Certainly" && certaintyTable}
			<div class="panel-muted space-y-3 p-5">
				<p class="m-0 text-base font-medium text-black">Puntajes de certeza</p>
				<p class="m-0 text-sm leading-relaxed text-zinc-700">
					Este quiz usa niveles de certeza. Durante el intento solo veras las
					opciones Baja, Media y Alta; los puntajes configurados son los siguientes.
				</p>
				<div class="overflow-x-auto">
					<table class="w-full table-fixed border-collapse text-sm">
						<colgroup>
							<col class="w-[22%]" />
							<col class="w-[39%]" />
							<col class="w-[39%]" />
						</colgroup>
						<thead>
							<tr class="text-left text-zinc-700">
								<th class="border border-zinc-300 bg-white p-2 text-xs font-semibold"
									>Nivel</th
								>
								<th class="border border-zinc-300 bg-white p-2 text-xs font-semibold"
									>Correcta</th
								>
								<th class="border border-zinc-300 bg-white p-2 text-xs font-semibold"
									>Incorrecta</th
								>
							</tr>
						</thead>
						<tbody>
							<tr>
								<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
									>Baja</td
								>
								<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
									>{certaintyTable.low.correct}</td
								>
								<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
									>{certaintyTable.low.incorrect}</td
								>
							</tr>
							<tr>
								<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
									>Media</td
								>
								<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
									>{certaintyTable.medium.correct}</td
								>
								<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
									>{certaintyTable.medium.incorrect}</td
								>
							</tr>
							<tr>
								<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
									>Alta</td
								>
								<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
									>{certaintyTable.high.correct}</td
								>
								<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
									>{certaintyTable.high.incorrect}</td
								>
							</tr>
						</tbody>
					</table>
				</div>
			</div>
		{/if}

		<div class="mt-auto flex flex-wrap items-center justify-between gap-3">
			<button
				class="btn-tertiary"
				type="button"
				onclick={() => quizUiStore.clearJoinPreview()}
			>
				<TimerReset size={14} class="mr-1 inline" />
				Volver
			</button>
			<button
				class="btn-primary"
				type="button"
				onclick={handleStart}
				disabled={startAttemptMutation.isPending || !canStart}
			>
				<Play size={14} class="mr-1 inline" />
				{startAttemptMutation.isPending
					? "Iniciando..."
					: canStart
						? "Comenzar intento"
						: "Disponible pronto"}
			</button>
		</div>
	</section>
{/if}
