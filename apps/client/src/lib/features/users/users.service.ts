import { request } from "$lib/shared/http/http"
import type { PromiseResult } from "$lib/shared/result"
import type { AppError } from "$lib/shared/errors"
import type { ManagedUser, UpdateUserRolePayload } from "$lib/features/users/types"

class UsersService {
	listCollaboratorCandidates = async (
		query?: string
	): PromiseResult<ManagedUser[], AppError> =>
		request<ManagedUser[]>({
			method: "GET",
			url: "/users/collaborator-candidates",
			params: {
				...(query ? { search: query } : {}),
			},
		})

	listAssistantCandidates = async (
		query?: string
	): PromiseResult<ManagedUser[], AppError> =>
		request<ManagedUser[]>({
			method: "GET",
			url: "/users",
			params: {
				roles: "student,assistant",
				...(query ? { search: query } : {}),
			},
		})

	setUserRole = async (
		payload: UpdateUserRolePayload
	): PromiseResult<ManagedUser, AppError> =>
		request<ManagedUser>({
			method: "PATCH",
			url: `/users/${payload.userId}/role`,
			data: {
				role: payload.role,
			},
		})
}

export const usersService = new UsersService()
