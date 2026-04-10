export type ManagedUserRole = "student" | "assistant" | "func"

export type ManagedUser = {
	id: string
	username: string
	name: string
	email: string
	role: ManagedUserRole
}

export type UpdateUserRolePayload = {
	userId: string
	role: "student" | "assistant"
}
