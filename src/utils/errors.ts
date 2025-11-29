export class NrfError extends Error {
  constructor(
    message: string,
    public readonly originalError?: Error,
    public readonly isTimeout: boolean = false,
    public readonly nrfUri?: string
  ) {
    super(message);
    this.name = 'NrfError';
  }
}

export const handleNrfError = (error: unknown, nrfUri?: string): NrfError => {
  if (error instanceof NrfError) {
    return error;
  }

  if (error instanceof Error) {
    const isTimeout = error.message.includes('timeout') || error.message.includes('ETIMEDOUT');

    return new NrfError(
      `NRF communication failed: ${error.message}`,
      error,
      isTimeout,
      nrfUri
    );
  }

  return new NrfError('An unknown NRF error occurred', undefined, false, nrfUri);
};
