/**
 * Proxy module that keeps legacy imports pointing at the shared
 * @noa-ark/server prompt controller implementation.
 */
export {
  handlePromptRequest,
  renderPromptTemplate,
  validatePromptPayload,
} from "@noa-ark/server/ai/controllers/prompt";

export type {
  AiRequestLogEntry,
  PromptControllerDependencies,
  PromptControllerResult,
  PromptPayload,
} from "@noa-ark/server/ai/controllers/prompt";
