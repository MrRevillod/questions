export type User = {
	id: string
	username: string
	name: string
	email: string
	role: "student" | "func" | "assistant"
}

export type LoginInput = {
	username: string
	password: string
}

export type LoginResponse = {
	user: User
	accessToken: string
	refreshToken: string
}

export type AuthTokens = {
	accessToken: string
	refreshToken: string
}
