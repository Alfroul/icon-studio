const isDev = import.meta.env.DEV;

export function logError(context: string, error: unknown) {
  if (isDev) {
    console.error(`[${context}]`, error);
  }
}

export function logWarn(context: string, error: unknown) {
  if (isDev) {
    console.warn(`[${context}]`, error);
  }
}
