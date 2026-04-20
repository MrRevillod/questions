export const QUESTION_FORMAT_EXAMPLE = `{
	"questions": [
		{
			"question": "<strong>Resuelve:</strong> \\(x^2 + 2x + 1\\)",
			"options": ["\\((x+1)^2\\)", "\\((x-1)^2\\)", "x^2 + 1", "<em>Ninguna</em>"],
			"answer": 0,
			"images": ["https://example.com/chile-map.png"]
		}
	]
}`

export const DEFAULT_CERTAINTY_VALUES = {
	low: {
		correct: "1",
		incorrect: "-1",
	},
	medium: {
		correct: "2",
		incorrect: "-2",
	},
	high: {
		correct: "3",
		incorrect: "-6",
	},
} as const

export const DEFAULT_CREATE_QUIZ_INPUT = {
	title: "",
	mode: "traditional",
	startTimeLocal: "",
	attemptDurationMinutes: "30",
	questionCount: "10",
	certainty: DEFAULT_CERTAINTY_VALUES,
} as const
