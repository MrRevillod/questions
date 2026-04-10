import * as v from "valibot"
import type { CertaintyConfig, QuizQuestion } from "$lib/features/quiz/types"
import { QuestionsFileSchema, type CreateQuizInput } from "$lib/features/quiz/schema"

type ParseQuestionsSuccess = {
	questions: QuizQuestion[]
	error: null
}

type ParseQuestionsError = {
	questions: null
	error: string
}

export type ParseQuestionsResult = ParseQuestionsSuccess | ParseQuestionsError

const INVALID_STRUCTURE_ERROR = "El archivo JSON no posee la estructura requerida"

export const toUtcIso = (localDateTime: string) =>
	new Date(localDateTime).toISOString()

export const parseNumber = (value: string) => {
	const parsed = Number(value)

	if (Number.isNaN(parsed)) {
		return null
	}

	return parsed
}

const hasValidQuestionShape = (question: QuizQuestion) => {
	if (question.options.length < 2) {
		return false
	}

	if (!Number.isInteger(question.answer)) {
		return false
	}

	return question.answer >= 0 && question.answer < question.options.length
}

export const parseQuestionsFile = async (
	file: File
): Promise<ParseQuestionsResult> => {
	try {
		const raw = await file.text()
		const payload = JSON.parse(raw) as unknown
		const parsed = v.safeParse(QuestionsFileSchema, payload)

		if (!parsed.success) {
			return {
				questions: null,
				error: INVALID_STRUCTURE_ERROR,
			}
		}

		const questions = parsed.output.questions as QuizQuestion[]

		if (!questions.every(hasValidQuestionShape)) {
			return {
				questions: null,
				error: INVALID_STRUCTURE_ERROR,
			}
		}

		return {
			questions,
			error: null,
		}
	} catch {
		return {
			questions: null,
			error: INVALID_STRUCTURE_ERROR,
		}
	}
}

export const buildCertaintyConfig = (
	input: CreateQuizInput
): CertaintyConfig | null => {
	if (input.mode !== "certainty") {
		return null
	}

	const lowCorrect = parseNumber(input.certainty.low.correct)
	const lowIncorrect = parseNumber(input.certainty.low.incorrect)
	const mediumCorrect = parseNumber(input.certainty.medium.correct)
	const mediumIncorrect = parseNumber(input.certainty.medium.incorrect)
	const highCorrect = parseNumber(input.certainty.high.correct)
	const highIncorrect = parseNumber(input.certainty.high.incorrect)

	if (
		lowCorrect === null ||
		lowIncorrect === null ||
		mediumCorrect === null ||
		mediumIncorrect === null ||
		highCorrect === null ||
		highIncorrect === null
	) {
		return null
	}

	if (
		!Number.isInteger(lowCorrect) ||
		!Number.isInteger(lowIncorrect) ||
		!Number.isInteger(mediumCorrect) ||
		!Number.isInteger(mediumIncorrect) ||
		!Number.isInteger(highCorrect) ||
		!Number.isInteger(highIncorrect)
	) {
		return null
	}

	return {
		low: { correct: lowCorrect, incorrect: lowIncorrect },
		medium: { correct: mediumCorrect, incorrect: mediumIncorrect },
		high: { correct: highCorrect, incorrect: highIncorrect },
	}
}

export const getMinStartDateTimeLocal = () => {
	const now = new Date(Date.now() + 60_000)
	const offset = now.getTimezoneOffset()
	const local = new Date(now.getTime() - offset * 60_000)

	return local.toISOString().slice(0, 16)
}
