<script lang="ts">
	import { createMutation } from "@tanstack/svelte-query"
	import { Field, Form, createForm, reset, useField } from "@formisch/svelte"
	import {
		CalendarDays,
		Clock3,
		Code2,
		FileJson,
		RefreshCw,
		Search,
		Users,
	} from "lucide-svelte"
	import { toast } from "svelte-sonner"
	import { quizService } from "$lib/features/quiz/quiz.service"
	import { quizUiStore } from "$lib/features/quiz/quiz.store.svelte"
	import { usersService } from "$lib/features/users/users.service"
	import { DEFAULT_CREATE_QUIZ_INPUT } from "$lib/features/quiz/constants"
	import { CreateQuizSchema, type CreateQuizInput } from "$lib/features/quiz/schema"
	import {
		buildCertaintyConfig,
		getMinStartDateTimeLocal,
		parseNumber,
		parseQuestionsFile,
		toUtcIso,
		validateQuestionCount,
	} from "$lib/features/quiz/utils"
	import type {
		CertaintyConfig,
		QuizMode,
		QuizQuestion,
	} from "$lib/features/quiz/types"
	import type { ManagedUser } from "$lib/features/users/types"
	import { toUserMessage } from "$lib/shared/errors"
	import CertaintyTable from "$lib/features/quiz/components/CertaintyTable.svelte"

	const createQuizForm = createForm({
		schema: CreateQuizSchema,
		initialInput: DEFAULT_CREATE_QUIZ_INPUT,
	})

	const createQuizMutation = createMutation(() => ({
		mutationFn: (payload: {
			title: string
			mode: QuizMode
			startTimeUtc: string
			attemptDurationMinutes: number
			questionCount: number
			collaboratorIds: string[]
			questions: QuizQuestion[]
			certaintyConfig: CertaintyConfig | null
		}) => quizService.createQuiz(payload),
	}))

	const modeField = useField(createQuizForm, { path: ["mode"] })
	const selectedMode = $derived.by(() =>
		modeField.input === "certainty" ? "certainty" : "traditional"
	)

	let selectedFile = $state<File | null>(null)
	let fileError = $state("")
	let isFileButtonHovered = $state(false)
	let collaboratorCandidates = $state<ManagedUser[]>([])
	let collaboratorSearch = $state("")
	let selectedCollaboratorIds = $state<string[]>([])
	let showCollaboratorDropdown = $state(false)
	let isCollaboratorLoading = $state(false)
	let collaboratorSearchTimer: ReturnType<typeof setTimeout> | null = null
	let hasInitializedCollaborators = false

	const filteredCollaboratorCandidates = $derived.by(() => {
		const query = collaboratorSearch.trim().toLowerCase()

		const base = collaboratorCandidates.filter(
			user => !selectedCollaboratorIds.includes(user.id)
		)
		if (!query) {
			return base
		}

		return base.filter(user => {
			return (
				user.username.toLowerCase().includes(query) ||
				user.name.toLowerCase().includes(query)
			)
		})
	})

	const selectedCollaborators = $derived.by(() => {
		return collaboratorCandidates.filter(user =>
			selectedCollaboratorIds.includes(user.id)
		)
	})

	const roleLabel = (role: ManagedUser["role"]) => {
		if (role === "func") return "Profesor"
		if (role === "assistant") return "Ayudante"
		return "Estudiante"
	}

	const loadCollaboratorCandidates = async (query?: string) => {
		isCollaboratorLoading = true
		const { value, error } = await usersService.listCollaboratorCandidates(query)

		if (error) {
			toast.error(toUserMessage(error))
			isCollaboratorLoading = false
			return
		}

		collaboratorCandidates = value ?? []
		isCollaboratorLoading = false
	}

	const addCollaborator = (user: ManagedUser) => {
		if (selectedCollaboratorIds.includes(user.id)) {
			return
		}

		selectedCollaboratorIds = [...selectedCollaboratorIds, user.id]
		collaboratorSearch = ""
		showCollaboratorDropdown = false
	}

	const removeCollaborator = (id: string) => {
		selectedCollaboratorIds = selectedCollaboratorIds.filter(userId => userId !== id)
	}

	$effect(() => {
		const query = collaboratorSearch.trim()

		if (collaboratorSearchTimer) {
			clearTimeout(collaboratorSearchTimer)
		}

		if (!hasInitializedCollaborators) {
			hasInitializedCollaborators = true
			void loadCollaboratorCandidates()
			return
		}

		collaboratorSearchTimer = setTimeout(() => {
			void loadCollaboratorCandidates(query || undefined)
		}, 250)

		return () => {
			if (collaboratorSearchTimer) {
				clearTimeout(collaboratorSearchTimer)
			}
		}
	})

	const onFileChange = (event: Event) => {
		const input = event.currentTarget as HTMLInputElement
		const file = input.files?.[0] ?? null

		selectedFile = file
		fileError = ""

		if (!file) {
			return
		}

		if (!file.name.toLowerCase().endsWith(".json")) {
			fileError = "Debes seleccionar un archivo .json."
			selectedFile = null
		}
	}

	const handleCreateSubmit = async (output: CreateQuizInput) => {
		if (!selectedFile) {
			fileError = "El JSON de preguntas es obligatorio."
			return
		}

		const parsedQuestions = await parseQuestionsFile(selectedFile)

		if (parsedQuestions.error !== null) {
			fileError = parsedQuestions.error
			return
		}

		const questions = parsedQuestions.questions

		const durationMinutes = parseNumber(output.attemptDurationMinutes)
		const questionCount = parseNumber(output.questionCount)

		if (!durationMinutes || durationMinutes <= 0) {
			toast.error("La duracion debe ser mayor que cero.")
			return
		}

		if (!questionCount || questionCount <= 0) {
			toast.error("La cantidad de preguntas debe ser mayor que cero.")
			return
		}

		const questionCountValidation = validateQuestionCount(
			questionCount,
			questions.length
		)

		if (!questionCountValidation.success) {
			toast.error(questionCountValidation.issues[0]?.message ?? "Cantidad invalida")
			return
		}

		const certaintyConfig = buildCertaintyConfig(output)

		if (output.mode === "certainty" && !certaintyConfig) {
			toast.error("La tabla de certeza contiene valores invalidos.")
			return
		}

		const { value, error } = await createQuizMutation.mutateAsync({
			title: output.title,
			mode: output.mode as QuizMode,
			startTimeUtc: toUtcIso(output.startTimeLocal),
			attemptDurationMinutes: durationMinutes,
			questionCount,
			collaboratorIds: selectedCollaboratorIds,
			questions,
			certaintyConfig,
		})

		if (error) {
			toast.error(toUserMessage(error))
			return
		}

		const joinCode = value.joinCode

		if (joinCode) {
			quizUiStore.openJoinCodeModal(joinCode)
		}

		handleReset()
	}

	const handleReset = () => {
		reset(createQuizForm, { initialInput: DEFAULT_CREATE_QUIZ_INPUT })
		selectedFile = null
		fileError = ""
		collaboratorSearch = ""
		selectedCollaboratorIds = []
		showCollaboratorDropdown = false
	}
</script>

<section
	class="panel-surface flex h-full min-h-0 flex-col overflow-hidden p-4 sm:p-5"
>
	<div
		class="flex flex-col gap-3 border-b border-zinc-200 pb-4 lg:flex-row lg:items-start lg:justify-between lg:gap-6"
	>
		<div class="lg:basis-[63%]">
			<h3 class="m-0 flex items-center gap-2 text-lg text-black">
				<Code2 size={18} class="text-black" />
				Crear un nuevo quiz
			</h3>
			<p class="mt-1 max-w-2xl text-sm leading-relaxed text-zinc-700">
				Carga preguntas desde JSON, agenda el inicio y define los colaboradores en un
				solo flujo.
			</p>
		</div>

		<div class="flex w-full flex-col gap-2 lg:max-w-md lg:items-end">
			<div class="flex flex-wrap items-center gap-2 lg:justify-end">
				<input
					id="questions-file-header"
					class="sr-only"
					type="file"
					accept=".json,application/json"
					onchange={onFileChange}
				/>
				<label
					for="questions-file-header"
					class={`btn-primary group inline-flex max-w-full cursor-pointer items-center ${selectedFile ? "justify-between gap-2 pr-3" : ""}`}
					title={selectedFile ? selectedFile.name : "Subir archivo JSON"}
					onmouseenter={() => (isFileButtonHovered = true)}
					onmouseleave={() => (isFileButtonHovered = false)}
				>
					{#if selectedFile}
						<RefreshCw size={14} class="shrink-0" />
						<span class="max-w-56 truncate text-left text-sm">
							{isFileButtonHovered ? "Reemplazar archivo" : selectedFile.name}
						</span>
					{:else}
						<FileJson size={14} class="mr-1 inline" />
						Subir preguntas
					{/if}
				</label>
				<button
					class="btn-secondary"
					type="button"
					onclick={quizUiStore.openFormatModal}
				>
					<Code2 size={14} class="mr-1 inline text-black" />
					Ver formato JSON
				</button>
			</div>
		</div>
	</div>

	{#if fileError}
		<p class="mt-3 text-sm text-red-700">{fileError}</p>
	{/if}

	<Form
		class="flex min-h-0 flex-1 flex-col"
		of={createQuizForm}
		onsubmit={handleCreateSubmit}
	>
		<div class="min-h-0 flex-1 overflow-y-auto pt-4">
			<div
				class="grid gap-4 lg:grid-cols-[minmax(0,1.6fr)_minmax(18rem,1fr)] lg:items-start"
			>
				<div class="grid gap-4">
					<div class="panel-muted grid gap-4 p-4 sm:p-5">
						<div>
							<p class="section-kicker m-0">Datos base</p>
						</div>

						<div class="grid gap-3 sm:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
							<Field of={createQuizForm} path={["title"]}>
								{#snippet children(field)}
									<label class="grid min-w-0 content-start gap-1.5">
										<span class="text-sm text-zinc-800">Nombre del quiz</span>
										<input
											{...field.props}
											class="input-base m-0 block"
											type="text"
											placeholder="Ej: Control Semana 3"
											value={field.input}
										/>
										<p class="m-0 min-h-4 text-xs leading-4 text-red-700">
											{field.errors?.[0] ?? ""}
										</p>
									</label>
								{/snippet}
							</Field>

							<Field of={createQuizForm} path={["mode"]}>
								{#snippet children(field)}
									<label class="grid min-w-0 content-start gap-1.5">
										<span class="text-sm text-zinc-800">Formato del quiz</span>
										<select
											{...field.props}
											class="input-base m-0 block"
											value={field.input}
										>
											<option value="traditional">Tradicional</option>
											<option value="certainty">Certeza</option>
										</select>
									</label>
								{/snippet}
							</Field>
						</div>
					</div>

					<div class="panel-muted grid gap-4 p-4 sm:p-5">
						<div>
							<p class="section-kicker m-0">Programacion</p>
						</div>
						<div
							class="grid gap-3 sm:grid-cols-[minmax(0,2fr)_minmax(0,1fr)_minmax(0,1fr)]"
						>
							<Field of={createQuizForm} path={["startTimeLocal"]}>
								{#snippet children(field)}
									<label class="grid min-w-0 content-start gap-1.5">
										<span
											class="flex min-h-5 items-center gap-1.5 text-sm text-zinc-800"
										>
											<CalendarDays size={15} class="text-black" />
											Fecha y hora de inicio
										</span>
										<input
											{...field.props}
											class="input-base"
											type="datetime-local"
											min={getMinStartDateTimeLocal()}
											value={field.input}
										/>
										<p class="m-0 text-xs leading-4 text-zinc-500">
											No se permiten fechas ni horas en el pasado.
										</p>
										<p class="m-0 min-h-4 text-xs leading-4 text-red-700">
											{field.errors?.[0] ?? ""}
										</p>
									</label>
								{/snippet}
							</Field>

							<Field of={createQuizForm} path={["attemptDurationMinutes"]}>
								{#snippet children(field)}
									<label class="grid min-w-0 content-start gap-1.5">
										<span
											class="flex min-h-5 items-center gap-1.5 text-sm text-zinc-800"
										>
											<Clock3 size={15} class="text-black" />
											Duracion (min)
										</span>
										<input
											{...field.props}
											class="input-base"
											type="number"
											min="1"
											value={field.input}
										/>
										<p
											class="m-0 text-xs leading-4 text-transparent"
											aria-hidden="true"
										>
											No se permiten fechas ni horas en el pasado.
										</p>
										<p class="m-0 min-h-4 text-xs leading-4 text-red-700">
											{field.errors?.[0] ?? ""}
										</p>
									</label>
								{/snippet}
							</Field>

							<Field of={createQuizForm} path={["questionCount"]}>
								{#snippet children(field)}
									<label class="grid min-w-0 content-start gap-1.5">
										<span class="min-h-5 text-sm text-zinc-800">
											Preguntas en la prueba
										</span>
										<input
											{...field.props}
											class="input-base"
											type="number"
											min="1"
											max="100"
											value={field.input}
										/>
										<p
											class="m-0 text-xs leading-4 text-transparent"
											aria-hidden="true"
										>
											No se permiten fechas ni horas en el pasado.
										</p>
										<p class="m-0 min-h-4 text-xs leading-4 text-red-700">
											{field.errors?.[0] ?? ""}
										</p>
									</label>
								{/snippet}
							</Field>
						</div>
					</div>
				</div>

				<section class="panel-muted flex flex-col gap-3 overflow-hidden p-4 sm:p-5">
					<p
						class="m-0 flex items-center gap-2 text-[11px] font-semibold tracking-[0.16em] text-zinc-700 uppercase"
					>
						<Users size={15} />
						Colaboradores
					</p>
					<p class="m-0 text-sm leading-relaxed text-zinc-700">
						Agrega profesores y ayudantes que podran gestionar este quiz contigo.
					</p>

					<label class="block">
						<div class="relative">
							<Search
								size={13}
								class="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-zinc-500"
							/>
							<input
								class="input-base py-2 pr-3 pl-8 text-sm"
								type="text"
								bind:value={collaboratorSearch}
								placeholder="Ej: Luciano"
								onfocus={() => (showCollaboratorDropdown = true)}
								onblur={() =>
									setTimeout(() => (showCollaboratorDropdown = false), 120)}
							/>

							{#if showCollaboratorDropdown && filteredCollaboratorCandidates.length > 0}
								<div
									class="panel-surface absolute z-20 mt-1 max-h-48 w-full overflow-auto p-1 shadow-none"
								>
									{#each filteredCollaboratorCandidates as user (user.id)}
										<button
											class="block w-full rounded-sm px-2.5 py-2 text-left text-sm text-zinc-800 hover:bg-zinc-100"
											type="button"
											onclick={() => addCollaborator(user)}
										>
											{user.name} - {roleLabel(user.role)}
										</button>
									{/each}
								</div>
							{/if}
						</div>
					</label>

					<div
						class="min-h-0 flex-1 overflow-auto rounded-sm border border-zinc-200 bg-white p-2"
					>
						{#if isCollaboratorLoading}
							<p class="m-0 px-2 py-3 text-sm text-zinc-600">
								Cargando colaboradores...
							</p>
						{:else if selectedCollaborators.length === 0}
							<p class="m-0 px-2 py-3 text-sm text-zinc-600">
								No has agregado colaboradores.
							</p>
						{:else}
							<div class="flex flex-wrap gap-2">
								{#each selectedCollaborators as user (user.id)}
									<button
										class="rounded-sm border border-zinc-300 bg-zinc-100 px-2.5 py-1.5 text-xs text-zinc-800 hover:bg-zinc-200"
										type="button"
										onclick={() => removeCollaborator(user.id)}
									>
										{user.name} ({roleLabel(user.role)}) ×
									</button>
								{/each}
							</div>
						{/if}
					</div>
				</section>
			</div>

			{#if selectedMode === "certainty"}
				<CertaintyTable form={createQuizForm} />
			{/if}

			<div class="flex flex-col-reverse justify-end gap-2 pt-4 sm:flex-row">
				<button class="btn-tertiary" type="button" onclick={handleReset}>
					Reiniciar
				</button>
				<button
					class="btn-primary"
					type="submit"
					disabled={createQuizMutation.isPending}
				>
					{createQuizMutation.isPending ? "Creando..." : "Crear quiz"}
				</button>
			</div>
		</div>
	</Form>
</section>
