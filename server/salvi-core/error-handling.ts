/**
 * Error Handling & Retry Logic
 * Standard Error Responses and Error Code Registry
 * 
 * @author Capomastro Holdings Ltd.
 * @license Proprietary - All Rights Reserved
 */

/**
 * Backoff Strategy
 */
export type BackoffStrategy = 'exponential' | 'linear' | 'constant';

/**
 * Error Category
 */
export type ErrorCategory = 'PAY' | 'SFK' | 'BC' | 'TIM';

/**
 * Error Details
 */
export interface ErrorDetails {
  payment_id?: string;
  operation_id?: string;
  failure_reason: string;
  suggested_action: string;
  additional_context?: Record<string, unknown>;
}

/**
 * Retry Information
 */
export interface RetryInfo {
  retryable: boolean;
  retry_after_seconds: number;
  max_retries: number;
  backoff_strategy: BackoffStrategy;
}

/**
 * Error Response Structure
 */
export interface SalviErrorResponse {
  error: {
    code: string;
    message: string;
    details: ErrorDetails;
    timestamp: string;
    trace_id: string;
    documentation_url: string;
  };
  retry_info: RetryInfo;
}

/**
 * Error Definition
 */
export interface ErrorDefinition {
  message: string;
  http: number;
  retryable: boolean;
  retry_after_seconds: number;
  max_retries: number;
}

/**
 * Error Code Registry
 */
export const ERROR_CODES: Record<string, ErrorDefinition> = {
  // Payment Errors (PAY_xxx)
  PAY_001: { 
    message: 'Invalid webhook signature', 
    http: 401, 
    retryable: false, 
    retry_after_seconds: 0, 
    max_retries: 0 
  },
  PAY_002: { 
    message: 'Payment already processed', 
    http: 409, 
    retryable: false, 
    retry_after_seconds: 0, 
    max_retries: 0 
  },
  PAY_003: { 
    message: 'Insufficient payment amount', 
    http: 402, 
    retryable: false, 
    retry_after_seconds: 0, 
    max_retries: 0 
  },
  PAY_004: { 
    message: 'Payment gateway unavailable', 
    http: 503, 
    retryable: true, 
    retry_after_seconds: 30, 
    max_retries: 5 
  },
  PAY_005: { 
    message: 'Payment timeout', 
    http: 504, 
    retryable: true, 
    retry_after_seconds: 60, 
    max_retries: 3 
  },

  // SFK Errors (SFK_xxx)
  SFK_001: { 
    message: 'Operation queue full', 
    http: 503, 
    retryable: true, 
    retry_after_seconds: 30, 
    max_retries: 5 
  },
  SFK_002: { 
    message: 'Invalid operation parameters', 
    http: 400, 
    retryable: false, 
    retry_after_seconds: 0, 
    max_retries: 0 
  },
  SFK_003: { 
    message: 'Timing synchronization failed', 
    http: 500, 
    retryable: true, 
    retry_after_seconds: 10, 
    max_retries: 3 
  },
  SFK_004: { 
    message: 'Operation not found', 
    http: 404, 
    retryable: false, 
    retry_after_seconds: 0, 
    max_retries: 0 
  },
  SFK_005: { 
    message: 'Security mode not supported', 
    http: 400, 
    retryable: false, 
    retry_after_seconds: 0, 
    max_retries: 0 
  },

  // Blockchain Errors (BC_xxx)
  BC_001: { 
    message: 'Hedera HCS submission failed', 
    http: 502, 
    retryable: true, 
    retry_after_seconds: 5, 
    max_retries: 3 
  },
  BC_002: { 
    message: 'XRPL payment failed', 
    http: 502, 
    retryable: true, 
    retry_after_seconds: 10, 
    max_retries: 3 
  },
  BC_003: { 
    message: 'Algorand smart contract call failed', 
    http: 502, 
    retryable: true, 
    retry_after_seconds: 5, 
    max_retries: 3 
  },
  BC_004: { 
    message: 'Insufficient blockchain balance', 
    http: 402, 
    retryable: false, 
    retry_after_seconds: 0, 
    max_retries: 0 
  },
  BC_005: { 
    message: 'Blockchain network congestion', 
    http: 503, 
    retryable: true, 
    retry_after_seconds: 60, 
    max_retries: 5 
  },

  // Timing Errors (TIM_xxx)
  TIM_001: { 
    message: 'Clock synchronization out of bounds', 
    http: 500, 
    retryable: true, 
    retry_after_seconds: 5, 
    max_retries: 3 
  },
  TIM_002: { 
    message: 'Femtosecond accuracy not met', 
    http: 500, 
    retryable: true, 
    retry_after_seconds: 10, 
    max_retries: 2 
  },
  TIM_003: { 
    message: 'Timing certification failed', 
    http: 500, 
    retryable: true, 
    retry_after_seconds: 30, 
    max_retries: 2 
  },
  TIM_004: { 
    message: 'Clock source unavailable', 
    http: 503, 
    retryable: true, 
    retry_after_seconds: 60, 
    max_retries: 5 
  }
} as const;

/**
 * Generate unique trace ID
 */
export function generateTraceId(): string {
  const timestamp = Date.now().toString(36);
  const random = Math.random().toString(36).substring(2, 10);
  return `trace_${timestamp}${random}`;
}

/**
 * Create error response
 */
export function createErrorResponse(
  errorCode: string,
  details: Partial<ErrorDetails>
): SalviErrorResponse {
  const errorDef = ERROR_CODES[errorCode];
  
  if (!errorDef) {
    throw new Error(`Unknown error code: ${errorCode}`);
  }

  return {
    error: {
      code: errorCode,
      message: errorDef.message,
      details: {
        failure_reason: details.failure_reason || errorDef.message,
        suggested_action: details.suggested_action || 'contact_support',
        payment_id: details.payment_id,
        operation_id: details.operation_id,
        additional_context: details.additional_context
      },
      timestamp: new Date().toISOString(),
      trace_id: generateTraceId(),
      documentation_url: `https://docs.salvi-framework.com/errors/${errorCode}`
    },
    retry_info: {
      retryable: errorDef.retryable,
      retry_after_seconds: errorDef.retry_after_seconds,
      max_retries: errorDef.max_retries,
      backoff_strategy: 'exponential'
    }
  };
}

/**
 * Calculate retry delay with exponential backoff
 */
export function calculateRetryDelay(
  baseDelay: number,
  attempt: number,
  strategy: BackoffStrategy = 'exponential'
): number {
  switch (strategy) {
    case 'exponential':
      return baseDelay * Math.pow(2, attempt - 1);
    case 'linear':
      return baseDelay * attempt;
    case 'constant':
    default:
      return baseDelay;
  }
}

/**
 * Check if error is retryable
 */
export function isRetryable(errorCode: string): boolean {
  const errorDef = ERROR_CODES[errorCode];
  return errorDef?.retryable ?? false;
}

/**
 * Get HTTP status code for error
 */
export function getHttpStatus(errorCode: string): number {
  const errorDef = ERROR_CODES[errorCode];
  return errorDef?.http ?? 500;
}

/**
 * Retry with backoff utility
 */
export async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  errorCode: string,
  onRetry?: (attempt: number, delay: number) => void
): Promise<T> {
  const errorDef = ERROR_CODES[errorCode];
  
  if (!errorDef || !errorDef.retryable) {
    return fn();
  }

  let lastError: Error | null = null;
  
  for (let attempt = 1; attempt <= errorDef.max_retries + 1; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error as Error;
      
      if (attempt <= errorDef.max_retries) {
        const delay = calculateRetryDelay(
          errorDef.retry_after_seconds * 1000,
          attempt,
          'exponential'
        );
        
        onRetry?.(attempt, delay);
        
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }

  throw lastError;
}

/**
 * Error handler middleware type
 */
export type ErrorHandler = (
  error: Error,
  req: unknown,
  res: unknown,
  next: () => void
) => void;

/**
 * Create Express error handler middleware
 */
export function createErrorHandler(): ErrorHandler {
  return (error, _req, res: any, _next) => {
    const errorCode = (error as any).code || 'SFK_002';
    const response = createErrorResponse(errorCode, {
      failure_reason: error.message
    });
    
    res.status(getHttpStatus(errorCode)).json(response);
  };
}
