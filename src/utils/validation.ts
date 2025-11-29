import { PlmnId, Snssai, Tai } from '../types/common-types';
import { InvalidParam } from '../types/problem-details-types';

export type ValidationResult = {
  isValid: boolean;
  invalidParams: InvalidParam[];
};

export const validatePlmnId = (plmnId: any, paramName: string): InvalidParam | null => {
  if (!plmnId || typeof plmnId !== 'object') {
    return { param: paramName, reason: 'must be an object' };
  }

  if (!plmnId.mcc || typeof plmnId.mcc !== 'string' || plmnId.mcc.length !== 3) {
    return { param: `${paramName}.mcc`, reason: 'must be a 3-digit string' };
  }

  if (!plmnId.mnc || typeof plmnId.mnc !== 'string' || (plmnId.mnc.length !== 2 && plmnId.mnc.length !== 3)) {
    return { param: `${paramName}.mnc`, reason: 'must be a 2 or 3-digit string' };
  }

  return null;
};

export const validateSnssai = (snssai: any, paramName: string): InvalidParam | null => {
  if (!snssai || typeof snssai !== 'object') {
    return { param: paramName, reason: 'must be an object' };
  }

  if (snssai.sst === undefined || snssai.sst === null) {
    return { param: `${paramName}.sst`, reason: 'is required' };
  }

  if (typeof snssai.sst !== 'number' || snssai.sst < 0 || snssai.sst > 255) {
    return { param: `${paramName}.sst`, reason: 'must be a number between 0 and 255' };
  }

  if (snssai.sd !== undefined && snssai.sd !== null) {
    if (typeof snssai.sd !== 'string' || !/^[0-9A-Fa-f]{6}$/.test(snssai.sd)) {
      return { param: `${paramName}.sd`, reason: 'must be a 6-digit hexadecimal string' };
    }
  }

  return null;
};

export const validateTai = (tai: any, paramName: string): InvalidParam | null => {
  if (!tai || typeof tai !== 'object') {
    return { param: paramName, reason: 'must be an object' };
  }

  const plmnError = validatePlmnId(tai.plmnId, `${paramName}.plmnId`);
  if (plmnError) {
    return plmnError;
  }

  if (!tai.tac || typeof tai.tac !== 'string' || !/^[0-9A-Fa-f]{4,6}$/.test(tai.tac)) {
    return { param: `${paramName}.tac`, reason: 'must be a 4-6 digit hexadecimal string' };
  }

  return null;
};

export const validateSupi = (supi: any): InvalidParam | null => {
  if (!supi || typeof supi !== 'string') {
    return { param: 'supi', reason: 'must be a non-empty string' };
  }

  if (!/^(imsi-|nai-)[0-9a-zA-Z@\.\-]+$/.test(supi)) {
    return { param: 'supi', reason: 'must be a valid SUPI format (imsi-* or nai-*)' };
  }

  return null;
};

export const parseJsonParam = <T>(
  paramValue: string | undefined,
  paramName: string
): { value: T | null; error: InvalidParam | null } => {
  if (!paramValue) {
    return { value: null, error: null };
  }

  try {
    const parsed = JSON.parse(paramValue);
    return { value: parsed, error: null };
  } catch (error) {
    return {
      value: null,
      error: { param: paramName, reason: 'must be valid JSON' }
    };
  }
};

export const validateRequiredParam = (
  value: any,
  paramName: string
): InvalidParam | null => {
  if (value === undefined || value === null || value === '') {
    return { param: paramName, reason: 'is required' };
  }
  return null;
};
