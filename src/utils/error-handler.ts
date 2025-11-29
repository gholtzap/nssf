import { Response } from 'express';
import { DatabaseError } from '../db/mongodb';
import { NrfError } from './errors';
import { createProblemDetails } from '../types/problem-details-types';

export const handleError = (error: unknown, res: Response, context: string): void => {
  console.error(`Error in ${context}:`, error);

  if (error instanceof DatabaseError) {
    const status = error.isConnectionError ? 503 : 500;
    const problemDetails = createProblemDetails(
      status,
      error.isConnectionError ? 'Service Unavailable' : 'Internal Server Error',
      error.isConnectionError
        ? 'Unable to connect to database. Please try again later.'
        : 'A database error occurred while processing the request',
      error.message
    );
    res.status(status).json(problemDetails);
    return;
  }

  if (error instanceof NrfError) {
    const status = error.isTimeout ? 504 : 503;
    const problemDetails = createProblemDetails(
      status,
      error.isTimeout ? 'Gateway Timeout' : 'Service Unavailable',
      error.isTimeout
        ? 'NRF service timed out'
        : 'Unable to communicate with NRF service',
      error.message
    );
    res.status(status).json(problemDetails);
    return;
  }

  const problemDetails = createProblemDetails(
    500,
    'Internal Server Error',
    `An unexpected error occurred in ${context}`,
    error instanceof Error ? error.message : 'Unknown error'
  );
  res.status(500).json(problemDetails);
};
