<script lang="ts">
	import { ClipboardList } from 'lucide-svelte'
	import QuizActions from '$lib/features/quiz/components/QuizActions.svelte'
	import JoinQuizForm from '$lib/features/quiz/components/JoinQuizForm.svelte'
	import JoinQuizInstructions from '$lib/features/quiz/components/JoinQuizInstructions.svelte'
	import CreateQuizForm from '$lib/features/quiz/components/CreateQuizForm.svelte'
	import CreatedQuizList from '$lib/features/quiz/components/CreatedQuizList.svelte'
	import JoinedQuizRunner from '$lib/features/quiz/components/JoinedQuizRunner.svelte'
	import AssistantManager from '$lib/features/users/components/AssistantManager.svelte'
	import QuizFormatModal from '$lib/features/quiz/components/QuizFormatModal.svelte'
	import QuizJoinCodeModal from '$lib/features/quiz/components/QuizJoinCodeModal.svelte'
import AttemptSubmittedModal from '$lib/features/quiz/components/AttemptSubmittedModal.svelte'
import AttemptResultView from '$lib/features/quiz/components/AttemptResultView.svelte'
import ManagedQuizAttempts from '$lib/features/quiz/components/ManagedQuizAttempts.svelte'
import { authStore } from '$lib/features/auth/auth.store.svelte'
import { quizUiStore } from '$lib/features/quiz/quiz.store.svelte'

	const canManageQuizzes = $derived.by(() => {
		const role = authStore.session?.user.role
		return role === 'func' || role === 'assistant'
	})
	const isTeacher = $derived(authStore.session?.user.role === 'func')

	$effect(() => {
		if (!canManageQuizzes && quizUiStore.activePanel !== 'join') {
			quizUiStore.activePanel = 'join'
		}

		if (!isTeacher && quizUiStore.activePanel === 'assistants') {
			quizUiStore.activePanel = 'join'
		}
	})
</script>

{#if quizUiStore.joinedQuiz}
	<div class="h-full min-h-0">
		<JoinedQuizRunner />
	</div>
{:else if (quizUiStore as unknown as Record<string, unknown>).attemptResult}
	<div class="h-full min-h-0">
		<AttemptResultView />
	</div>
{:else if (quizUiStore as unknown as Record<string, unknown>).managedAttemptsPanel}
	<div class="h-full min-h-0">
		<ManagedQuizAttempts />
	</div>
{:else if quizUiStore.joinPreview}
	<div class="h-full min-h-0">
		<JoinQuizInstructions />
	</div>
{:else}
	<section class="grid h-full min-h-0 grid-rows-[auto_auto_minmax(0,1fr)] gap-5">
		<div class="flex flex-col gap-2.5">
			<p class="section-kicker m-0">Panel principal</p>
			<h2 class="m-0 flex items-center gap-2 text-xl leading-tight text-black sm:text-2xl">
				<ClipboardList size={20} class="text-black" />
				Comenzar
			</h2>
			<p class="max-w-3xl text-sm leading-relaxed text-zinc-700 sm:text-base">
				Elige una accion para unirte a una actividad existente o crear un nuevo quiz.
			</p>
		</div>

		<QuizActions />

		<div class="min-h-0 overflow-hidden">
			{#if quizUiStore.activePanel === 'join'}
				<JoinQuizForm />
			{/if}

			{#if canManageQuizzes && quizUiStore.activePanel === 'create'}
				<CreateQuizForm />
			{/if}

			{#if canManageQuizzes && quizUiStore.activePanel === 'mine'}
				<CreatedQuizList />
			{/if}

			{#if isTeacher && quizUiStore.activePanel === 'assistants'}
				<AssistantManager />
			{/if}
		</div>
	</section>

		<QuizFormatModal />
		<QuizJoinCodeModal />
		<AttemptSubmittedModal />
{/if}
