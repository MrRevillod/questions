# Reglas de Dominio

## Proposito

Definir las reglas funcionales del sistema antes de refactorizar el backend, la API REST y el modelo de autorizacion.

Este documento es la fuente de verdad para el nuevo modelo de dominio.

## Conceptos Base

- `Quiz`: evaluacion creada y gestionada por un profesor o ayudante.
- `Owner`: usuario que creo el quiz y es su responsable principal.
- `Collaborator`: ayudante o profesor autorizado explicitamente para gestionar un quiz.
- `Attempt`: ejecucion de un quiz por parte de un estudiante.

## Roles Globales

El sistema tiene exactamente tres roles globales:

- `student`
- `assistant`
- `func`

Significado de cada rol:

- `student`: puede participar en quizzes.
- `assistant`: puede gestionar quizzes, pero solo aquellos donde es owner o collaborator.
- `func`: rol de profesor. Hereda todas las capacidades de `assistant` y ademas puede asignar o quitar el rol de ayudante a estudiantes.

## Modelo de Permisos

Los permisos se evalúan en dos capas:

1. Permiso global por rol.
2. Autorizacion especifica sobre el recurso.

Ejemplos:

- Un `assistant` puede tener la capacidad global de editar quizzes, pero solo puede editar quizzes donde sea `owner` o `collaborator`.
- Un `student` puede tener la capacidad global de entregar intentos, pero solo puede entregar sus propios intentos.

## Ownership y Colaboracion en Quizzes

### Ownership

- Todo quiz tiene exactamente un `owner_id`.
- El owner siempre es un `func` o un `assistant`.
- El owner puede leer y actualizar el quiz.
- El owner puede gestionar los colaboradores del quiz.

### Collaborators

- Un collaborator es un `func` o `assistant` asociado a un quiz.
- Un collaborator puede leer y actualizar el quiz.
- Un collaborator no puede gestionar colaboradores en el quiz.

## Reglas de Acceso para Estudiantes

### Unirse por codigo

- Un `student` se une a un quiz solo mediante su codigo.
- El `join_code` del quiz es unico a nivel global.
- Si el backend genera un codigo repetido, debe regenerarlo antes de guardar el quiz.

### Orden aleatorio de preguntas

- El orden de las preguntas se presenta de forma aleatoria al estudiante.
- Cada pregunta debe tener un identificador estable propio.
- Las preguntas no deben identificarse por su posicion dentro del arreglo.
- Las respuestas enviadas por el estudiante deben referenciar `question_id`, no `question_index`.
- El backend debe poder reconstruir y validar un intento aunque el orden mostrado al estudiante sea distinto del orden de almacenamiento.

### Ciclo de vida del intento

- Un estudiante solo puede tener un intento total por quiz.
- Si existe un intento en progreso para ese quiz, el sistema debe reanudarlo y no crear uno nuevo.
- Si el intento ya fue entregado, el estudiante no puede volver a iniciar el quiz.
- Un estudiante no puede iniciar el intento antes de `start_time`.
- Un estudiante solo puede entregar su propio intento en progreso.
- Un intento entregado no puede enviarse nuevamente.
- El intento real se crea cuando el estudiante confirma el inicio desde la pantalla de instrucciones.
- El quiz debe mostrarse al estudiante usando un snapshot del intento generado por el backend.

## Reglas de Acceso para Profesores y Ayudantes

### Assistant

Un `assistant` puede:

- crear quizzes
- listar quizzes de los que es owner
- listar quizzes donde es collaborator
- leer el detalle completo de gestion de quizzes donde es owner o collaborator
- actualizar quizzes donde es owner o collaborator

### Func (Profesor)

Un `func` puede hacer todo lo que puede hacer un `assistant`, y ademas:

- asignar el rol `assistant` a un `student`
- quitar el rol `assistant` a un `assistant`, dejandolo nuevamente como `student`

Un `func` no puede:

- promover a otro usuario a `func`

## Visibilidad de Datos del Quiz

La API debe exponer distintas views segun el caso de uso.

### View de gestion

Visible para owner y collaborators:

- metadata del quiz
- owner del quiz
- configuracion temporal
- configuracion de certeza
- conjunto completo de preguntas
- identificadores estables de cada pregunta
- respuestas correctas
- collaborators

### View de participante

Visible para estudiantes durante los flujos de union e intento:

- metadata necesaria del quiz para renderizar el intento
- preguntas
- identificador estable de cada pregunta
- alternativas/opciones
- imagenes
- informacion de certeza solo si es necesaria para responder

Nunca debe incluir:

- respuestas correctas
- detalles internos de correccion que no sean necesarios para responder
- metadata exclusiva de gestion

## Reglas de Intentos

- El backend es responsable del estado del intento.
- La entrega del intento nunca debe depender solo del cliente.
- La clave de respuestas debe permanecer del lado del servidor en los flujos orientados a estudiantes.
- Las respuestas del intento deben almacenarse vinculadas al identificador estable de cada pregunta.
- El cliente debe persistir localmente el snapshot del intento para tolerar perdida de conexion.
- Las respuestas deben guardarse de forma incremental, una por una, y no solo al final.
- El snapshot local debe eliminarse cuando el intento se entregue correctamente.
- El backend debe impedir iniciar un intento antes de `start_time`.

## Modelo de Dominio Objetivo

El refactor debe mover el sistema hacia estos conceptos:

- `quizzes`
  - includes `owner_id`
  - contiene preguntas con identificadores estables
- `quiz_collaborators`
- `quiz_attempts`
- `quiz_answers`

El campo actual `authorized_user_ids` se considera ambiguo y no debe seguir siendo el mecanismo principal de autorizacion.

## Fuera de Alcance para Esta Fase

- no dar soporte a roles personalizados fuera de `student`, `assistant`, `func`
- no dar soporte a promover usuarios a `func`
- no dar soporte a edicion abierta de permisos en la UI
- no dar soporte a correccion y notas en esta etapa del refactor

## Decisiones Fijadas Aqui

- `func` hereda todas las capacidades de `assistant`.
- El ownership es explicito mediante `owner_id`.
- El ownership del quiz es fijo y no existe una operacion para transferirlo.
- La colaboracion es explicita mediante una tabla relacional, no mediante un array.
- La participacion de estudiantes ocurre solo a traves de join-by-code.
- Las preguntas se identifican por id estable y no por indice.
