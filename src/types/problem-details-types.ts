import { Uri } from './common-types';

export type ProblemDetails = {
  type?: Uri;
  title?: string;
  status?: number;
  detail?: string;
  instance?: Uri;
  cause?: string;
  invalidParams?: InvalidParam[];
  supportedFeatures?: string;
};

export type InvalidParam = {
  param: string;
  reason?: string;
};

export enum ProblemType {
  INVALID_REQUEST = 'https://tools.ietf.org/html/rfc7231#section-6.5.1',
  NOT_FOUND = 'https://tools.ietf.org/html/rfc7231#section-6.5.4',
  INTERNAL_ERROR = 'https://tools.ietf.org/html/rfc7231#section-6.6.1',
  SERVICE_UNAVAILABLE = 'https://tools.ietf.org/html/rfc7231#section-6.6.3',
  GATEWAY_TIMEOUT = 'https://tools.ietf.org/html/rfc7231#section-6.6.5'
}

export const createProblemDetails = (
  status: number,
  title: string,
  detail?: string,
  cause?: string,
  invalidParams?: InvalidParam[]
): ProblemDetails => {
  const problemType = getProblemType(status);

  return {
    type: problemType,
    title,
    status,
    detail,
    cause,
    invalidParams: invalidParams && invalidParams.length > 0 ? invalidParams : undefined
  };
};

const getProblemType = (status: number): Uri => {
  switch (status) {
    case 400:
      return ProblemType.INVALID_REQUEST;
    case 404:
      return ProblemType.NOT_FOUND;
    case 500:
      return ProblemType.INTERNAL_ERROR;
    case 503:
      return ProblemType.SERVICE_UNAVAILABLE;
    case 504:
      return ProblemType.GATEWAY_TIMEOUT;
    default:
      return ProblemType.INTERNAL_ERROR;
  }
};
