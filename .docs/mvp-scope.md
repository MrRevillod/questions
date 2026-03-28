# Alcance del MVP

## Proposito

Definir el alcance funcional minimo que debe funcionar despues del refactor del backend.

Este alcance esta limitado intencionalmente para que el refactor pueda cerrarse con fronteras de dominio estables antes de agregar mas funcionalidades.

## Capacidades Incluidas

### Autenticacion

- login con LDAP
- refresh de sesion
- logout
- obtener el perfil del usuario autenticado

### Gestion de quizzes

- crear quiz
- listar quizzes gestionables por el usuario autenticado
- leer el detalle de gestion de un quiz
- actualizar metadata y contenido del quiz
- agregar un collaborator a un quiz
- quitar un collaborator de un quiz

### Participacion de estudiantes

- unirse a un quiz por codigo
- ver una pantalla de instrucciones antes de iniciar
- recibir preguntas en orden aleatorio
- iniciar un intento
- guardar respuestas individualmente durante la rendicion
- entregar un intento

### Gestion de roles

- `func` puede promover `student` a `assistant`
- `func` puede degradar `assistant` a `student`

## Fuera de Alcance para Este Refactor

- promover un usuario a `func`
- transferir ownership de un quiz
- dashboards de analitica de quizzes
- flujos de revision de correccion para profesores
- gestion de asignacion de estudiantes
- correccion y consulta de notas
- UI de audit logs
- personalizacion avanzada de permisos mas alla de roles y policies por recurso

## Superficie Minima de Endpoints Esperada

### Auth

- `POST /auth/login`
- `POST /auth/refresh`
- `POST /auth/logout`
- `GET /users/me`

### Quizzes

- `POST /quizzes`
- `GET /quizzes/me`
- `GET /quizzes/{quizId}`
- `PATCH /quizzes/{quizId}`
- `GET /quizzes/{quizId}/collaborators`
- `PUT /quizzes/{quizId}/collaborators/{userId}`
- `DELETE /quizzes/{quizId}/collaborators/{userId}`
- `POST /quizzes/join-by-code`

### Attempts

- `POST /quizzes/{quizId}/attempts`
- `GET /quizzes/{quizId}/attempts/me`
- `PUT /attempts/{attemptId}/answers/{questionId}` o equivalente
- `POST /attempts/{attemptId}/submit`

### Users

- `GET /users`
- `PATCH /users/{userId}/role`

## Definicion de Hecho para la Fase 0

La Fase 0 se considera completa cuando:

- las reglas de roles estan documentadas
- las reglas de ownership y colaboracion estan documentadas
- las reglas de union por codigo e intentos de estudiantes estan documentadas
- la superficie de endpoints objetivo del MVP esta documentada
- no hay ambiguedad sobre lo que debe implementar el siguiente refactor de base de datos y API
