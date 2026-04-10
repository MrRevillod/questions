import * as v from "valibot"

const requiredString = (message: string) =>
	v.pipe(v.string(message), v.trim(), v.minLength(1, message))

const requiredIntegerString = (requiredMessage: string) =>
	v.pipe(
		requiredString(requiredMessage),
		v.regex(/^-?\d+$/, "Debe ser un numero entero.")
	)

const isFutureLocalDateTime = (value: string) => {
	const timestamp = new Date(value).getTime()

	if (Number.isNaN(timestamp)) {
		return false
	}

	return timestamp > Date.now()
}

const isIntegerInRange = (value: string, min: number, max: number) => {
	const parsed = Number(value)

	if (!Number.isInteger(parsed)) {
		return false
	}

	return parsed >= min && parsed <= max
}

const CertaintyWeightSchema = v.object({
	correct: v.pipe(
		requiredIntegerString("Obligatorio."),
		v.check(value => isIntegerInRange(value, 0, 100), "Debe estar entre 0 y 100.")
	),
	incorrect: v.pipe(
		requiredIntegerString("Obligatorio."),
		v.check(value => isIntegerInRange(value, -100, 0), "Debe estar entre -100 y 0.")
	),
})

const CertaintySchema = v.object({
	low: CertaintyWeightSchema,
	medium: CertaintyWeightSchema,
	high: CertaintyWeightSchema,
})

export const JoinQuizSchema = v.object({
	code: v.pipe(
		requiredString("El codigo es obligatorio."),
		v.minLength(3, "El codigo debe tener al menos 3 caracteres.")
	),
})

export const CreateQuizSchema = v.object({
	title: v.pipe(
		requiredString("El nombre del quiz es obligatorio."),
		v.maxLength(100, "El nombre del quiz no puede superar 100 caracteres.")
	),
	mode: v.picklist(["traditional", "certainty"]),
	startTimeLocal: v.pipe(
		requiredString("La fecha de inicio es obligatoria."),
		v.regex(
			/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/,
			"La fecha debe tener formato valido."
		),
		v.check(isFutureLocalDateTime, "La fecha de inicio debe estar en el futuro.")
	),
	attemptDurationMinutes: v.pipe(
		requiredString("La duracion es obligatoria."),
		v.regex(/^\d+$/, "La duracion debe ser un numero positivo."),
		v.check(
			value => isIntegerInRange(value, 1, 240),
			"La duracion debe estar entre 1 y 240 minutos."
		)
	),
	certainty: CertaintySchema,
})

export const QuizQuestionSchema = v.object({
	question: v.pipe(
		requiredString("La pregunta es obligatoria."),
		v.maxLength(1000, "La pregunta no puede superar 1000 caracteres.")
	),
	options: v.pipe(
		v.array(requiredString("Cada opcion es obligatoria.")),
		v.minLength(2, "Debe haber al menos 2 opciones."),
		v.maxLength(5, "No puede haber mas de 5 opciones.")
	),
	answer: v.pipe(
		v.number("La respuesta debe ser numerica."),
		v.integer("La respuesta debe ser un indice entero."),
		v.minValue(0, "La respuesta debe ser un indice valido.")
	),
	images: v.pipe(
		v.array(v.string("Cada imagen debe ser texto.")),
		v.maxLength(5, "No puede haber mas de 5 imagenes por pregunta.")
	),
})

export const QuestionsFileSchema = v.object({
	questions: v.pipe(
		v.array(QuizQuestionSchema),
		v.minLength(1, "Debe haber al menos una pregunta."),
		v.maxLength(100, "No puede haber mas de 100 preguntas.")
	),
})

export type JoinQuizInput = v.InferInput<typeof JoinQuizSchema>
export type CreateQuizInput = v.InferInput<typeof CreateQuizSchema>
