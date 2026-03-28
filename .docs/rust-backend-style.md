# Estilo Rust Backend

## Proposito

Definir el estilo de codigo, patrones de organizacion y decisiones de arquitectura para el backend en Rust.

Este documento debe usarse como referencia al implementar nuevas tareas o refactors en `apps/server`.

## Objetivos

- mantener consistencia en modulos, nombres y capas
- evitar mezclar transporte HTTP, dominio y persistencia
- favorecer codigo pequeno, predecible y facil de testear
- adaptar buenas practicas de Rust al estilo modular de Sword

## Aclaracion importante

- Este proyecto no usa Clean Architecture estricta.
- No se deben introducir capas artificiales como `application/`, `domain/`, `infrastructure/` o `presentation/` si el proyecto no las necesita.
- La arquitectura base del proyecto es modular por dominio, con archivos planos por modulo.
- Si un modulo crece mucho, puede subdividirse, pero siempre siguiendo el estilo real del proyecto y no forzando una arquitectura academica.

## Principios generales

- El backend se organiza por modulo de dominio, no por tipo tecnico global.
- Cada modulo debe tener una responsabilidad de negocio clara.
- Los controllers deben ser finos.
- La logica de negocio debe vivir en services o use cases.
- La persistencia debe vivir en repositories.
- La autorizacion debe vivir en guards para permisos gruesos y en policies para permisos por recurso.
- Las entidades de BD no deben exponerse directamente como respuestas HTTP.

## Estructura objetivo por modulo

Cada modulo de negocio deberia seguir, en lo posible, esta estructura:

```text
modulo/
  controller.rs
  service.rs
  repository.rs
  entity.rs
  dtos.rs
  views.rs
  errors.rs
  mod.rs
```

Regla practica:

- empezar simple con archivos directos
- dividir cuando el modulo ya no sea facil de mantener
- si hace falta subdividir, preferir submodulos pequenos orientados al negocio, por ejemplo `attempts/policy.rs` o `quizzes/views.rs`, sin convertir el proyecto en una Clean Architecture formal

## Convenciones de modulos Sword

### `mod.rs`

- `mod.rs` declara submodulos internos
- `mod.rs` expone solo lo necesario usando `pub use`
- `mod.rs` registra adapters y components del modulo

Ejemplo esperado:

- `controller` se registra en `AdapterRegistry`
- `service` y `repository` se registran en `ComponentRegistry`

### `main.rs`

- `main.rs` solo compone la aplicacion
- no debe tener logica de negocio
- solo registra modulos y layers globales

## Controllers

Los controllers representan la capa HTTP.

### Responsabilidades

- declarar rutas y metodos HTTP
- leer params, query y body
- aplicar interceptors
- delegar al service o use case correspondiente
- mapear el resultado a `JsonResponse`

### Lo que no deben hacer

- no deben contener logica de negocio compleja
- no deben hablar directamente con SQL
- no deben decidir reglas de autorizacion finas del dominio
- no deben transformar entidades de BD manualmente si existe una `View`

### Reglas de estilo

- un endpoint, una responsabilidad clara
- extraer `req.param`, `req.query` o `req.body_validator` al inicio
- usar nombres de metodos HTTP expresivos, no abreviados
- devolver `Result<JsonResponse, JsonResponse>` o `AppResult` segun el patron del modulo

## Services y use cases

Los services implementan la logica principal del modulo.

### Responsabilidades

- orquestar validaciones de negocio
- coordinar repositories
- aplicar policies de dominio
- construir views o devolver entidades segun el caso interno

### Reglas de estilo

- usar metodos pequenos y centrados en un caso de uso
- si un service crece demasiado, separarlo por casos de uso
- no meter codigo HTTP ni detalles del request dentro del service
- preferir firmas explicitas sobre argumentos ambiguos

### Recomendacion

Para casos de uso importantes, preferir nombres concretos:

- `create_quiz`
- `update_quiz`
- `start_attempt`
- `submit_attempt`

Nota:

- esto no implica crear una carpeta `use_cases/`.
- puede resolverse perfectamente con metodos bien nombrados dentro de `service.rs`, mientras el archivo siga siendo mantenible.

## Repositories

Los repositories encapsulan acceso a datos.

### Responsabilidades

- ejecutar queries SQL
- mapear resultados hacia entidades de persistencia
- ocultar detalles de `sqlx` al resto del modulo

### Reglas de estilo

- no mezclar reglas de autorizacion con consultas de persistencia, salvo filtros claros de lectura
- usar nombres de metodos que expresen intencion de dominio
- devolver `Option<T>` cuando un recurso pueda no existir
- mantener las queries cerca del metodo que las usa
- si una consulta se vuelve demasiado compleja, considerar dividir repositories por agregado o subdominio

## Entities, DTOs y Views

### `entity.rs`

- representa el modelo de persistencia o dominio interno
- puede usar derives de `sqlx`, `serde`, `Type`, `FromRow`, etc., solo si realmente corresponden
- no debe diseñarse pensando en la respuesta HTTP publica

### `dtos.rs`

- contiene requests de entrada y payloads de comandos
- se usa para validar input externo
- nombres recomendados:
  - `CreateQuizRequest`
  - `UpdateQuizRequest`
  - `JoinQuizRequest`
  - `SubmitAttemptRequest`

### `views.rs`

- contiene responses de salida
- una view por caso de uso, no una view universal
- las views deben filtrar campos sensibles segun el actor y el flujo

Regla clave:

- no reutilizar la misma struct para entity, request y response

## Errores

### `errors.rs` por modulo

- cada modulo debe definir sus errores de dominio propios
- usar `thiserror::Error`
- usar `HttpError` cuando el error deba mapearse directamente a HTTP

### `shared/errors.rs`

- centraliza el error global de la aplicacion
- compone errores de modulos (`QuizError`, `UsersError`, etc.)
- define `AppResult<T>`

### Reglas de estilo

- usar errores especificos, no strings genericos por todo el codigo
- no devolver `BadRequest(String)` si existe un error de dominio mas expresivo
- separar claramente:
  - validacion de input
  - recurso no encontrado
  - conflicto de estado
  - falta de permisos

## Autorizacion

### Regla general

- autenticacion y permisos globales en interceptors/guards
- permisos finos por recurso en policies o services

### Recomendacion objetivo

- `SessionCheck` para autenticar
- guard o interceptor para dejar el `User` autenticado disponible en la request
- `RoleGuard` para permisos globales simples
- `QuizPolicy`, `AttemptPolicy`, `UserPolicy` para permisos por recurso

### Lo que no debe hacerse

- no meter toda la autorizacion del sistema en un enum gigante y un solo service
- no duplicar `if role == ...` en todos los controllers

## Nombres y convenciones

### Tipos

- structs y enums en `PascalCase`
- traits en `PascalCase`
- enums de dominio con nombres del negocio, no tecnicos

### Funciones y metodos

- `snake_case`
- nombres verbales y especificos

### Variables

- nombres cortos si el contexto es obvio
- nombres largos si mejoran claridad
- evitar abreviaturas ambiguas

### Sufijos recomendados

- `Repository`
- `Service`
- `Policy`
- `Request`
- `View`
- `Error`

## Imports y organizacion de archivos

### Orden recomendado de imports

1. `crate::...`
2. std
3. dependencias externas

### Reglas de estilo

- agrupar imports por origen
- evitar imports innecesarios
- usar `crate::modulo::*` solo cuando el modulo ya expose una API interna clara y pequena

## Validacion

- la validacion de input HTTP debe ocurrir en DTOs y body validators
- la validacion de negocio debe ocurrir en services o policies
- no confiar en validaciones solo del frontend

## Persistencia de quizzes e intentos

### Quizzes

- el `owner_id` es obligatorio
- los `collaborators` se modelan en tabla separada
- las preguntas deben tener id estable
- las preguntas no deben depender del indice del arreglo

### Intentos

- el `attempt` es una entidad real del backend
- las respuestas se guardan por `question_id`
- el flujo de intento usa snapshot del backend
- el backend define `started_at`, `expires_at` y `status`

## Estilo de API interna

- preferir firmas explicitas a builders complejos cuando el caso de uso es pequeno
- usar `Option<T>` y `Result<T, E>` de forma idiomatica
- usar `let Some(x) = ... else { ... };` cuando mejore claridad
- evitar anidar `match` o `if` innecesariamente

## Testing

### Objetivo

- testear reglas de negocio y permisos antes que detalles de framework

### Prioridad de tests

1. policies
2. services/use cases
3. repositories complejos
4. integracion HTTP

### Reglas

- cada policy importante debe tener tests directos
- cada endpoint critico de permisos debe tener al menos un test de acceso permitido y uno denegado
- evitar tests excesivamente acoplados a strings de logs o implementaciones internas triviales

## Checklist para nuevas implementaciones

Antes de añadir una funcionalidad nueva, revisar:

- [ ] existe un modulo de dominio correcto para esta funcionalidad
- [ ] el controller solo hace trabajo HTTP
- [ ] la logica de negocio vive en service o use case
- [ ] la autorizacion esta separada entre guard y policy
- [ ] el repository encapsula acceso a datos
- [ ] el request esta en `dtos.rs`
- [ ] la respuesta publica esta en `views.rs`
- [ ] no se expone la entity directamente
- [ ] hay errores de dominio claros
- [ ] hay tests para permisos y reglas centrales

## Decisiones de estilo fijadas aqui

- Arquitectura modular por dominio.
- Controllers finos y services pequenos.
- Entities, DTOs y Views separadas.
- Repositories como unica capa de acceso a datos.
- Errores por modulo mas error global compartido.
- Autorizacion separada entre permisos globales y policies por recurso.
- Preguntas identificadas por id estable, no por indice.
