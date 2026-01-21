# Prompt de Desarrollo y Arquitectura - Rust AI Agent Template

Este documento sirve como contexto maestro para el desarrollo de sistemas basados en Agentes de IA en este proyecto. Define la arquitectura, el flujo de datos y las reglas para la implementación de nuevos agentes y herramientas.

## 1. Arquitectura Agéntica

El sistema sigue un patrón de **Orquestación y Especialización**. A diferencia de un MVC tradicional, aquí la lógica de negocio reside en la interacción entre modelos de lenguaje (LLMs) y herramientas ejecutables.

### Mapa del Territorio

- **`src/agents`**: **El Cerebro y las Manos.**
  - **`orchestrator`**: El agente principal que recibe la intención del usuario.
  - **`specialized`**: Agentes expertos en tareas únicas (ej. `address`, `damage`).
  - **`tools`**: **Las Manos.** Funciones deterministas que los agentes pueden ejecutar (ej. `geocoding`, `calculators`).
    - _Regla_: Las herramientas deben ser puras o manejar sus propios side-effects de forma aislada.
  - _Regla General_: Todo lo relacionado con la inteligencia o ejecución de tareas vive aquí.

- **`src/infra`**: **El Sistema Nervioso.** Conexiones a servicios externos.
  - **`telemetry.rs`**: Observabilidad.

- **`src/api`**: **Los Sentidos.** La interfaz HTTP.
  - Recibe peticiones externas y se las pasa al `Orchestrator`. No contiene lógica de negocio, solo transformación de DTOs.

- **`src/state.rs`**: **Memoria a Corto Plazo.**
  - Mantiene el estado compartido de la aplicación (referencias al orquestador).

## 2. Stack Tecnológico & Estándares

- **Framework de Agentes**: `rig` (Rust Intelligent Graph).
- **Inyección de Modelos**:
  - **Especialistas**: Deben ser genéricos (`struct MyAgent<M: CompletionModel...>`) para aceptar cualquier modelo inyectado.
  - **Orquestador**: Decide qué modelo concreto usa cada especialista.

### Reglas de Implementación

1.  **Prompts como Código**: Los System Prompts deben estar en archivos `.md` separados (ej. `system_prompt.md`) y cargados con `include_str!`. Esto facilita la lectura y edición sin recompilar cadenas gigantes en Rust.
2.  **Tipado Fuerte**: Usa structs de Rust para definir los inputs y outputs de las Tools. Aprovecha el sistema de tipos para validar antes de que el LLM "alucine".
3.  **Modularidad**: Un agente no debe saber de la existencia de la API HTTP. La API importa al agente, nunca al revés.

## 3. Flujo de Trabajo para Nuevas Features

Para añadir una nueva capacidad al sistema:

1.  **Definir la Tool (Opcional)**: Si el agente necesita hacer algo físico (buscar en DB, calcular envío), crea la herramienta en `src/tools`.
2.  **Crear el Especialista**:
    - Crea `src/agents/specialized/NUEVO_AGENTE.rs`.
    - Define su `system_prompt.md`.
    - Configura qué modelo usará (generalmente uno más rápido/barato si la tarea es simple).
3.  **Registrar en el Orquestador**:
    - Añade el nuevo especialista como una `.tool()` en `src/agents/orchestrator/mod.rs`.
    - Actualiza el prompt del orquestador si es necesario para que sepa cuándo usarlo.
4.  **Exponer (Si aplica)**: Si el agente requiere un endpoint directo (raro, usualmente es vía orquestador), añádelo en `src/api`.

---

**Nota para IA:** Al generar código, recuerda que estamos usando `rig`. Verifica `src/agents/orchestrator/mod.rs` para ver qué modelos están disponibles y pre-configurados. Prioriza el uso de `Gemini Flash` para tareas de alta velocidad y `Claude Sonnet` o `GPT-4o` para razonamiento complejo.
