import { AI_PROVIDER_SETTINGS_PATH } from '$lib/config/ai-nav';

export interface AIErrorInfo {
  message: string;
  type: 'configuration' | 'provider_unavailable' | 'network' | 'model_not_found' | 'unknown';
  actionable: boolean;
  settingsPath?: string;
}

export const defaultParseError = (err: unknown): AIErrorInfo => {
  const errorMessage =
    err instanceof Error
      ? err.message
      : typeof err === 'string'
        ? err
        : 'An error occurred during AI generation';

  // Check for configuration issues
  if (
    errorMessage.includes('No default provider') ||
    errorMessage.includes('No default provider set') ||
    errorMessage.includes('Configuration incomplete') ||
    errorMessage.includes('provider configuration') ||
    errorMessage.includes('Missing fields') ||
    errorMessage.includes('not configured') ||
    errorMessage.includes('Provider not found') ||
    errorMessage.includes('Provider not registered')
  ) {
    return {
      message:
        'AI provider is not configured. Please set up Agent Platform in Settings → AI Providers.',
      type: 'configuration',
      actionable: true,
      settingsPath: AI_PROVIDER_SETTINGS_PATH,
    };
  }

  // Check for model not found errors
  if (
    errorMessage.includes('model') &&
    (errorMessage.includes('not found') ||
      errorMessage.includes('404') ||
      errorMessage.includes('does not exist') ||
      errorMessage.includes('not installed'))
  ) {
    let modelMatch = errorMessage.match(/model\s+['"]([^'"]+)['"]/);
    if (!modelMatch) {
      modelMatch = errorMessage.match(/model\s+['"]([^'"]+?)['"]/);
    }
    if (!modelMatch) {
      modelMatch = errorMessage.match(/"error":\s*"model\s+['"]([^'"]+?)['"]/);
    }
    if (!modelMatch) {
      modelMatch = errorMessage.match(/\{"error":\s*"model\s+['"]([^'"]+?)['"]/);
    }

    const modelName = modelMatch ? modelMatch[1] : 'the specified model';

    return {
      message: `Model "${modelName}" is not available on agent-platform. Check the model alias in Settings → AI Providers.`,
      type: 'model_not_found',
      actionable: true,
      settingsPath: AI_PROVIDER_SETTINGS_PATH,
    };
  }

  // Check for provider unavailable
  if (
    errorMessage.includes('Provider not available') ||
    errorMessage.includes('service is not running') ||
    errorMessage.includes('failed to connect')
  ) {
    return {
      message:
        'AI provider is not available. Please check that your AI service is running and properly configured.',
      type: 'provider_unavailable',
      actionable: true,
      settingsPath: AI_PROVIDER_SETTINGS_PATH,
    };
  }

  // Check for network issues
  if (
    errorMessage.includes('Network error') ||
    errorMessage.includes('timeout') ||
    errorMessage.includes('connection')
  ) {
    return {
      message:
        'Network error connecting to AI provider. Please check your connection and try again.',
      type: 'network',
      actionable: false,
    };
  }

  // Default unknown error
  return {
    message: errorMessage,
    type: 'unknown',
    actionable: false,
  };
};
