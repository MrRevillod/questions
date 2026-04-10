import { PersistedState } from 'runed'
import type {
	AttemptAnswer,
	ManagedAttemptSummary,
	AttemptResult,
	AttemptSnapshot,
	JoinQuizPreview
} from '$lib/features/quiz/types'

type ManagedQuizAttemptsPanel = {
	quizId: string
	title: string
}

type AttemptSubmittedSummary = {
	studentName: string
	joinCode: string | null
	submittedAtLabel: string
}

class QuizUiStore {
	#activeAttempt = new PersistedState<AttemptSnapshot | null>('quiz-active-attempt', null, {
		storage: 'local',
		syncTabs: false
	})

	activePanel = $state<'join' | 'create' | 'mine' | 'assistants' | null>('join')
	isFormatModalOpen = $state(false)
	isJoinCodeModalOpen = $state(false)
	createdQuizJoinCode = $state<string | null>(null)
	joinPreview = $state<JoinQuizPreview | null>(null)
	participantJoinCode = $state<string | null>(null)
	isAttemptSubmittedModalOpen = $state(false)
	attemptSubmittedSummary = $state<AttemptSubmittedSummary | null>(null)
	attemptResult = $state<AttemptResult | null>(null)
	managedAttemptsPanel = $state<ManagedQuizAttemptsPanel | null>(null)
	managedAttempts = $state<ManagedAttemptSummary[]>([])
	currentQuestionIndex = $state(0)

	get activeAttempt() {
		return this.#activeAttempt.current
	}

	get joinedQuiz() {
		return this.activeAttempt?.quiz ?? null
	}

	get attemptId() {
		return this.activeAttempt?.attemptId ?? null
	}

	get currentAttemptResult() {
		return this.attemptResult
	}

	get currentManagedAttemptsPanel() {
		return this.managedAttemptsPanel
	}

	get currentManagedAttempts() {
		return this.managedAttempts
	}

	setPanel = (panel: 'join' | 'create' | 'mine' | 'assistants') => {
		this.activePanel = panel
	}

	openFormatModal = () => {
		this.isFormatModalOpen = true
	}

	closeFormatModal = () => {
		this.isFormatModalOpen = false
	}

	openJoinCodeModal = (joinCode: string) => {
		this.createdQuizJoinCode = joinCode
		this.isJoinCodeModalOpen = true
	}

	closeJoinCodeModal = () => {
		this.isJoinCodeModalOpen = false
		this.createdQuizJoinCode = null
	}

	showJoinPreview = (preview: JoinQuizPreview, joinCode?: string) => {
		this.joinPreview = preview
		this.attemptResult = null
		this.managedAttemptsPanel = null
		this.managedAttempts = []
		this.participantJoinCode = joinCode ?? null
		this.currentQuestionIndex = 0
		this.activePanel = 'join'
	}

	startQuizAttempt = (attempt: AttemptSnapshot) => {
		this.#activeAttempt.current = attempt
		this.attemptResult = null
		this.managedAttemptsPanel = null
		this.managedAttempts = []
		this.joinPreview = null
		this.currentQuestionIndex = 0
		this.activePanel = 'join'
	}

	syncActiveAttempt = (attempt: AttemptSnapshot) => {
		this.#activeAttempt.current = attempt
		this.attemptResult = null
		this.managedAttemptsPanel = null
		this.managedAttempts = []
		this.joinPreview = null
		this.activePanel = 'join'
	}

	showAttemptResult = (result: AttemptResult) => {
		this.#activeAttempt.current = null
		this.joinPreview = null
		this.managedAttemptsPanel = null
		this.managedAttempts = []
		this.attemptResult = result
		this.currentQuestionIndex = 0
		this.activePanel = 'join'
	}

	openManagedAttemptsPanel = (quizId: string, title: string) => {
		this.#activeAttempt.current = null
		this.joinPreview = null
		this.attemptResult = null
		this.managedAttemptsPanel = { quizId, title }
		this.managedAttempts = []
		this.activePanel = 'mine'
	}

	setManagedAttempts = (attempts: ManagedAttemptSummary[]) => {
		this.managedAttempts = attempts
	}

	closeManagedAttemptsPanel = () => {
		this.managedAttemptsPanel = null
		this.managedAttempts = []
	}

	leaveQuizAttempt = () => {
		this.#activeAttempt.current = null
		this.joinPreview = null
		this.attemptResult = null
		this.managedAttemptsPanel = null
		this.managedAttempts = []
		this.participantJoinCode = null
		this.currentQuestionIndex = 0
		this.activePanel = 'join'
	}

	clearJoinPreview = () => {
		this.joinPreview = null
		this.attemptResult = null
		this.managedAttemptsPanel = null
		this.managedAttempts = []
		this.participantJoinCode = null
	}

	openAttemptSubmittedModal = (summary: AttemptSubmittedSummary) => {
		this.attemptSubmittedSummary = summary
		this.isAttemptSubmittedModalOpen = true
	}

	closeAttemptSubmittedModal = () => {
		this.isAttemptSubmittedModalOpen = false
		this.attemptSubmittedSummary = null
	}

	clearAllStores = () => {
		this.#activeAttempt.current = null
		this.joinPreview = null
		this.attemptResult = null
		this.managedAttemptsPanel = null
		this.managedAttempts = []
		this.participantJoinCode = null
		this.createdQuizJoinCode = null
		this.isJoinCodeModalOpen = false
		this.attemptSubmittedSummary = null
		this.isAttemptSubmittedModalOpen = false
		this.currentQuestionIndex = 0
		this.activePanel = 'join'
	}

	upsertAnswer = (answer: AttemptAnswer) => {
		if (!this.activeAttempt) {
			return
		}

		const answers = [...this.activeAttempt.answers]
		const index = answers.findIndex((item) => item.questionId === answer.questionId)

		if (index >= 0) {
			answers[index] = answer
		} else {
			answers.push(answer)
		}

		this.#activeAttempt.current = {
			...this.activeAttempt,
			answers
		}
	}

	goToQuestion = (index: number) => {
		if (!this.joinedQuiz) {
			return
		}

		const maxIndex = this.joinedQuiz.questions.length - 1
		this.currentQuestionIndex = Math.min(Math.max(index, 0), maxIndex)
	}
}

export const quizUiStore = new QuizUiStore()
