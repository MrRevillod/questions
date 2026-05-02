
# Instrucciones de frontend page

## Resumen

Ramtun es una plataforma de tests de certeza academica. En este documento te definiré los requerimientos de la web.

La plataforma cuenta con cursos, quizzes, question banks y otros, sin embargo, los cursos no almacenan estudiantes, si no que es curso está pensado como un contenedor de quizzes y bancos de preguntas, para que el  profesor y ayudantes puedan organizar su material. Un estudiante no se une a un curso, sino que se une a un quiz, y el quiz pertenece a un curso.

## Usuarios

La plataforma tiene 4 tipos de usuarios, que se  traducen a roles, "student", "func", "assistant" y "admin".

### Student

Un student debe poder:
- Iniciar sesión
- Unirse a un quiz (con un código de quiz)
- Enviar un quiz (finalizar intento) 
- Ver resultados de un quiz (con un código de quiz)


### Assistant

Un assistant, es un student que ha sido promovido a "ayudante" en un curso X. Un assistant debe poder:

- Hacer todo lo que un student puede hacer
- Crear quizzes para el curso X
- Finalizar quizzes para el curso X
- Ver resultados de los quizzes del curso X
- Administrar members del curso, añadir o remover.
- Crear un curso.
- Crear bancos de preguntas para el curso X
- Eliminar bancos de preguntas para el curso X
- Eliminar quizzes para el curso X

### Func
Un func es un profesor. Un func debe poder:
- Hacer todo lo que un assistant puede hacer
- Promover a un student a assistant (globalmente)


### Admin
Un admin es un super usuario, que tiene acceso a todo. Un admin debe poder:
- Hacer todo lo que un func puede hacer

## Diseño e interfaz de la página web

Actualmente hay componentes en la página web, sin embargo, cumplia las necesidades básicas del MVP inicial. La siguiente meta es mejorar el diseño y la interfaz de la web. Por ejemplo, haciendo que se parezca más a una plataforma de educación, tipo Educa Blackboard.

Mantendriamos la paleta de colores sobria en escala de grises, pero mejorariamos en general la experiencia de usuarios, haciendo que sea más fácil navegar por la página, y que sea más agradable a la vista.

La web utiliza tailwind y fuente Palatino.
