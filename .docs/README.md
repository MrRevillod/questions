# Documentacion

Este directorio contiene la documentacion funcional y tecnica que define la arquitectura objetivo del sistema.

## Archivos

- `domain-rules.md`: fuente de verdad para roles, permisos, ownership de quizzes, intentos y reglas de visibilidad.
- `mvp-scope.md`: alcance funcional minimo que debe completarse antes de extender el sistema.
- `attempt-flow.md`: flujo tecnico del intento, persistencia local, orden aleatorio de preguntas y guardado incremental.
- `rust-backend-style.md`: guia de estilo, patrones y arquitectura para desarrollar el backend en Rust con Sword.

## Uso

- Actualiza estos documentos antes de hacer cambios estructurales en el backend.
- Trata estos archivos como el contrato para los refactors de base de datos, API REST, views y autorizacion.
