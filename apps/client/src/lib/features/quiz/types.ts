export type QuizMode = "traditional" | "certainty"

export type QuizKind = "Traditional" | "Certainly"

export type CertaintyWeight = {
	correct: number
	incorrect: number
}

export type CertaintyConfig = {
	low: CertaintyWeight
	medium: CertaintyWeight
	high: CertaintyWeight
}

export type QuizQuestion = {
	question: string
	options: string[]
	answer: number
	images: string[]
}

export type CreateQuizPayload = {
	title: string
	mode: QuizMode
	startTimeUtc: string
	attemptDurationMinutes: number
	questionCount: number
	collaboratorIds: string[]
	questions: QuizQuestion[]
	certaintyConfig: CertaintyConfig | null
}

export type JoinQuizPayload = {
	code: string
}

export type QuizSummary = {
	id: string
	ownerId: string
	title: string
	kind: QuizKind
	joinCode: string
	questionCount: number
	startTime: string
	attemptDurationMinutes: number
	closedAt: string | null
	createdAt: string
}

export type QuizDetailQuestion = {
	id: string
	question: string
	options: string[]
	answer: number
	images: string[]
}

export type QuizDetail = {
	id: string
	ownerId: string
	title: string
	kind: QuizKind
	joinCode: string
	questions: QuizDetailQuestion[]
	certaintyTable: CertaintyConfig | null
	certainlyTable?: CertaintyConfig | null
	startTime: string
	attemptDurationMinutes: number
	closedAt: string | null
	createdAt: string
	updatedAt: string
}

export type JoinQuizPreview = {
	id: string
	title: string
	kind: QuizKind
	questionCount: number
	certaintyTable: CertaintyConfig | null
	certainlyTable?: CertaintyConfig | null
	startTime: string
	attemptDurationMinutes: number
	closedAt: string | null
}

export type QuizParticipantQuestion = {
	questionId: string
	question: string
	options: string[]
	images: string[]
}

export type QuizParticipant = {
	id: string
	title: string
	kind: QuizKind
	questions: QuizParticipantQuestion[]
	certaintyTable: CertaintyConfig | null
	certainlyTable?: CertaintyConfig | null
	startTime: string
	attemptDurationMinutes: number
	closedAt: string | null
}

export type AttemptStatus = "in_progress" | "submitted"

export type AttemptCertaintyLevel = "low" | "medium" | "high"

export type AttemptAnswer = {
	questionId: string
	answerIndex: number
	certaintyLevel: AttemptCertaintyLevel | null
}

export type AttemptSnapshot = {
	attemptId: string
	quizId: string
	startedAt: string
	expiresAt: string
	status: AttemptStatus
	quiz: QuizParticipant
	answers: AttemptAnswer[]
}

export type AttemptQuestionResult = {
	questionId: string
	question: string
	options: string[]
	images: string[]
	answerIndex: number | null
	correctAnswerIndex: number
	certaintyLevel: AttemptCertaintyLevel | null
	isCorrect: boolean
	awardedPoints: number
}

export type AttemptResult = {
	attemptId: string
	quizId: string
	status: AttemptStatus
	submittedAt: string
	evaluatedAt: string
	scorePoints: number
	scorePointsMax: number
	grade: number
	resultsReleasedAt: string
	resultsViewedAt: string | null
	questions: AttemptQuestionResult[]
}

export type ManagedAttemptSummary = {
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

export type FinalizeAndPublishSummary = {
	quizId: string
	finalizedAttempts: number
	publishedAttempts: number
}
