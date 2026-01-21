# 🤖 Rust AI Agent Template

Esta plantilla proporciona una base de nivel empresarial para construir sistemas de **Agentes de IA** en Rust. Utiliza el framework **Rig** para la lógica agéntica y **Axum** para la interfaz HTTP, diseñado con una arquitectura modular y escalable.

---

## ✨ Características Principales

- **Arquitectura Orquestador-Especialistas**: Patrón jerárquico donde un agente principal delega tareas complejas a sub-agentes expertos.
- **Memoria Persistente (Redis)**: Historial de chat y estado de sesión mantenido en Redis, permitiendo conversaciones continuas y escalabilidad horizontal.
- **Herramientas Tipadas**: Sistema robusto de `Tools` definidas con structs de Rust, minimizando alucinaciones y errores de ejecución.
- **Observabilidad**: Telemetría integrada con `tracing` para monitorear el flujo de decisiones de los agentes.
- **API RESTful**: Interfaz HTTP lista para producción con Axum.

---

## 🚀 Inicio Rápido

### 1. Prerrequisitos

- **Rust** (versión estable reciente)
- **Redis** (ejecutándose localmente o accesible vía URL)
- **API Keys** de proveedores LLM (OpenAI, Anthropic o Google Gemini)

### 2. Configuración

1. Clona el repositorio.
2. Copia el archivo de ejemplo:
   ```bash
   cp .env.example .env
   ```
3. Configura tus variables en `.env`. A continuación se detallan las más importantes:

   | Variable | Descripción | Valor por Defecto |
   | :--- | :--- | :--- |
   | `PORT` | Puerto del servidor HTTP | `8080` |
   | `REDIS_URL` | Conexión a Redis | `redis://default@localhost:6379` |
   | `SESSION_TTL` | Tiempo de vida de la sesión (segundos) | `86400` (24h) |
   | `OPENAI_API_KEY` | Key para GPT-4o, etc. | - |
   | `GEMINI_API_KEY` | Key para modelos Gemini | - |
   | `ANTHROPIC_API_KEY`| Key para Claude 3.5 Sonnet | - |
   | `DEBUG_LEVEL` | Nivel de logs (INFO, DEBUG, TRACE) | `INFO` |

### 3. Ejecutar

```bash
cargo run
```

El servidor iniciará en `http://0.0.0.0:8080`.

---

## 🔌 API Reference

### Chat (`POST /chat`)

Interactúa con el orquestador. El sistema mantendrá el contexto basado en el `session_id` si se provee (header o body, según implementación de cliente).

**Request:**
```json
{
  "prompt": "Hola, necesito validar una dirección en Madrid",
  "session_id": "user-123" 
}
```

**Ejemplo cURL:**
```bash
curl -X POST http://localhost:8080/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "¿Cuál es el estatus del envío #99?", "session_id": "test-1"}'
```

---

## 🧠 Arquitectura del Sistema

El sistema emula una organización inteligente:

### 1. 🎼 Orquestador (`agents/orchestrator`)
Es el gerente general. Recibe todas las peticiones, mantiene la "Big Picture" y decide a quién asignar el trabajo.
- **Responsabilidad**: Entender la intención, mantener la conversación y delegar.
- **Memoria**: Recupera el historial de chat desde Redis antes de cada interacción.

### 2. 🕵️ Especialistas (`agents/specialized`)
Son expertos de dominio (ej. `AddressSpecialist`, `DamageEvaluator`).
- Se inyectan al orquestador como **Herramientas Avanzadas**.
- Tienen su propio System Prompt y pueden usar sus propias herramientas (ej. consultar base de datos, API externa).
- **Ventaja**: Permite usar modelos más pequeños/rápidos para tareas específicas, o prompts muy detallados sin "contaminar" el contexto principal.

### 3. 🛠️ Herramientas (`agents/tools`)
Funciones puras o deterministas que ejecutan acciones concretas.
- **Ejemplos**: `GeoCoding`, `CostCalculator`, `TextReverser`.
- No usan LLM, son código Rust estándar.

### 4. 💾 Estado y Memoria (`infra/redis.rs`)
- **RedisProvider**: Abstracción sobre `redis-rs`.
- Almacena el historial de mensajes serializado en JSON.
- Permite que el agente "recuerde" lo dicho 5 turnos atrás.

---

## 🛠️ Guía de Desarrollo: Crear un Nuevo Especialista

Pasos para añadir una nueva capacidad (ej. `FinancialAnalyst`) al sistema.

### Paso 1: Crear el Módulo
Duplica la carpeta `src/agents/specialized/dummy` y renómbrala a `analyst`.

### Paso 2: Definir el Prompt
Edita `analyst/system_prompt.md`. Define claramente qué hace y qué NO hace este agente.
> "Eres un experto financiero. Tu trabajo es analizar riesgos..."

### Paso 3: Implementar la Lógica
En `analyst/mod.rs`:
1. Renombra los structs (`AnalystArgs`, `AnalystOutput`).
2. Define los argumentos que necesita recibir (ej. `amount`, `currency`).
3. Implementa el trait `Tool`. La función `definition()` es crucial: describe al LLM orquestador **cuándo** usar esta herramienta.

### Paso 4: Registrar
1. En `src/agents/specialized/mod.rs`: `pub mod analyst;`
2. En `src/agents/orchestrator/mod.rs`:
   - Instancia el agente: `let analyst = AnalystSpecialist::new(model.clone());`
   - Añádelo al builder: `.tool(analyst)`

¡Listo! El orquestador ahora tiene un experto financiero en su equipo.

---

## 📚 Recursos Adicionales

- **Documentación del crate `rig`**: Para una comprensión profunda de los componentes y la API de Rig, consulta la documentación oficial en [docs.rs](https://docs.rs/rig/latest/rig/).