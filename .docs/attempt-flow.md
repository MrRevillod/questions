# Flujo de Intento

## Proposito

Definir el flujo tecnico y funcional del intento de un estudiante, incluyendo inicio, obtencion de preguntas, persistencia local, tolerancia a perdida de conexion y entrega final.

Este documento fija las decisiones actuales para el refactor del sistema de intentos.

## Objetivos del flujo

- permitir que el estudiante entre al quiz por codigo
- mostrar una pantalla de instrucciones antes de iniciar formalmente el intento
- crear un `attempt` real en backend antes de mostrar la prueba
- entregar las preguntas ya mezcladas desde el servidor
- tolerar perdida de conexion durante la rendicion
- evitar depender de `question_index`
- permitir guardar progreso incremental del estudiante

## Flujo funcional acordado

### 1. Ingreso del codigo

- El estudiante ingresa el codigo del quiz.
- El frontend no inicia inmediatamente el intento real.
- Primero se debe consultar una vista previa del quiz.

### 2. Pantalla de instrucciones

- El backend devuelve informacion suficiente para mostrar una pantalla de instrucciones.
- Esta pantalla debe incluir, como minimo:
  - titulo del quiz
  - tipo de quiz
  - cantidad de preguntas
  - duracion del intento
  - instrucciones basicas
- En esta etapa todavia no se crea el intento definitivo.

### 3. Comienzo real del intento

- Cuando el estudiante pulsa `Comenzar`, el frontend llama al endpoint de inicio de intento.
- El backend debe rechazar el inicio si el quiz aun no llega a `startTime`.
- El estudiante solo puede tener un intento total por quiz.
- Si ya existe un intento en progreso, el backend debe devolver ese intento en vez de crear uno nuevo.
- Si ya existe un intento entregado, el backend debe rechazar un nuevo inicio.
- En este punto el backend crea un `attempt` real.
- El backend responde con un snapshot completo del intento.

## Snapshot del intento

El snapshot del intento debe incluir, como minimo:

- `attemptId`
- `quizId`
- `startedAt`
- `expiresAt`
- preguntas del quiz en orden aleatorio
- `questionId` estable para cada pregunta

Ejemplo conceptual:

```json
{
  "attemptId": "uuid",
  "quizId": "uuid",
  "startedAt": "2026-03-27T20:00:00Z",
  "expiresAt": "2026-03-27T20:30:00Z",
  "questions": [
    {
      "questionId": "uuid",
      "question": "...",
      "options": ["A", "B", "C"],
      "images": []
    }
  ]
}
```

## Orden aleatorio de preguntas

- El backend entrega las preguntas ya mezcladas.
- El frontend no debe depender del indice de posicion para identificar una pregunta.
- Cada pregunta se identifica siempre por `questionId`.
- Las respuestas del intento se guardan usando `questionId`.

## Persistencia local durante la rendicion

### Decision actual

- El snapshot del intento se guarda localmente en el navegador.
- Cuando el intento se entrega correctamente, ese almacenamiento local se elimina.

### Recomendacion tecnica

- Para un MVP pequeno, `localStorage` puede servir.
- Para una solucion mas robusta y profesional, se recomienda `IndexedDB`.
- La recomendacion para este sistema es preferir `IndexedDB` para snapshots e intentos en curso.

### Que debe guardarse localmente

- `attemptId`
- `quizId`
- `startedAt`
- `expiresAt`
- `questions` ya mezcladas
- respuestas actuales del estudiante
- estado de sincronizacion de cada respuesta, si se implementa autosave remoto

### Que no debe guardarse localmente

- respuestas correctas
- logica de correccion
- cualquier dato sensible que no sea necesario para reanudar el intento

## Estrategia ante perdida de conexion

### Regla principal

- El estudiante debe poder seguir respondiendo aunque se pierda la conexion, siempre que ya tenga el snapshot del intento cargado en el navegador.

### Implicaciones

- No se deben pedir preguntas una por una durante la rendicion.
- Las preguntas deben descargarse completas al iniciar el intento.
- El frontend debe poder seguir mostrando el intento con los datos persistidos localmente.

## Guardado de respuestas

### Decision actual

- Las respuestas no se acumulan solo para el final.
- Cada vez que el estudiante responde o modifica una respuesta, esa respuesta debe guardarse individualmente.

### Orden recomendado de guardado

1. Guardar la respuesta en almacenamiento local.
2. Intentar sincronizarla con el backend.
3. Si falla la red, dejarla pendiente de sincronizacion y reintentar despues.

### Forma de la respuesta

Cada respuesta debe asociarse a una sola pregunta:

```json
{
  "questionId": "uuid",
  "answerIndex": 2,
  "certaintyLevel": "medium"
}
```

Notas:

- `certaintyLevel` solo aplica si el quiz lo requiere.
- La identidad de la respuesta es `questionId`, no la posicion visual.

## Entrega final

- La entrega final del intento ocurre en una operacion separada.
- El `submit` no debe ser el unico momento en que el servidor conoce las respuestas.
- El `submit` marca el intento como `submitted`.
- Si el `submit` es exitoso, el frontend debe limpiar el almacenamiento local del intento.

## Responsabilidad del servidor

- El servidor es la fuente de verdad para:
  - `startedAt`
  - `expiresAt`
  - `status` del intento
  - validacion de si el intento sigue vigente
-  - validacion de si el quiz ya comenzo
- El cliente solo representa ese estado y persiste progreso para tolerancia a fallos.

## Endpoints objetivo relacionados con intentos

Superficie recomendada para este flujo:

- `POST /quizzes/join-by-code`
  - devuelve vista previa para la pantalla de instrucciones
- `POST /quizzes/{quizId}/attempts`
  - crea el intento y devuelve el snapshot completo
- `GET /quizzes/{quizId}/attempts/me`
  - permite recuperar el intento activo del estudiante si existe
- `PUT /attempts/{attemptId}/answers/{questionId}` o `PATCH /attempts/{attemptId}/answers`
  - guarda una respuesta individual
- `POST /attempts/{attemptId}/submit`
  - entrega formalmente el intento

## Decision de API recomendada

Para el guardado individual de respuestas, la opcion preferida es:

- `PUT /attempts/{attemptId}/answers/{questionId}`

Porque expresa mejor que cada respuesta es un recurso individual asociado al intento.

## Decisiones fijadas aqui

- El intento real se crea solo cuando el estudiante pulsa `Comenzar`.
- Antes del intento existe una pantalla de instrucciones.
- El servidor devuelve las preguntas en orden aleatorio.
- Las preguntas se identifican por `questionId` estable.
- El `join_code` es unico global y se reintenta generar antes de guardar si colisiona.
- Un estudiante solo puede tener un intento total por quiz.
- Si el intento ya existe y sigue en progreso, el backend lo reanuda.
- No se puede iniciar un intento antes de `startTime`.
- El cliente guarda localmente el snapshot del intento.
- Las respuestas se guardan una por una, no solo al final.
- El `submit` final es una operacion separada del guardado incremental.
- Al finalizar correctamente, el snapshot local se elimina.
