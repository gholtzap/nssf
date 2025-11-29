import { Snssai } from '../types/common-types';
import { InvalidParam } from '../types/problem-details-types';
import { validateSnssai } from '../utils/validation';

const MAX_REQUESTED_NSSAI = 8;

type RequestedNssaiValidationResult = {
  isValid: boolean;
  invalidParams: InvalidParam[];
  validatedNssai?: Snssai[];
};

const snssaiMatches = (s1: Snssai, s2: Snssai): boolean => {
  return s1.sst === s2.sst && s1.sd === s2.sd;
};

const hasDuplicates = (nssaiList: Snssai[]): boolean => {
  for (let i = 0; i < nssaiList.length; i++) {
    for (let j = i + 1; j < nssaiList.length; j++) {
      if (snssaiMatches(nssaiList[i], nssaiList[j])) {
        return true;
      }
    }
  }
  return false;
};

const removeDuplicates = (nssaiList: Snssai[]): Snssai[] => {
  const uniqueNssai: Snssai[] = [];

  for (const snssai of nssaiList) {
    const isDuplicate = uniqueNssai.some(existing => snssaiMatches(existing, snssai));
    if (!isDuplicate) {
      uniqueNssai.push(snssai);
    }
  }

  return uniqueNssai;
};

export const validateRequestedNssai = (
  requestedNssai: Snssai[] | undefined,
  paramPrefix: string = 'requestedNssai'
): RequestedNssaiValidationResult => {
  const invalidParams: InvalidParam[] = [];

  if (!requestedNssai) {
    return { isValid: true, invalidParams: [], validatedNssai: [] };
  }

  if (!Array.isArray(requestedNssai)) {
    invalidParams.push({
      param: paramPrefix,
      reason: 'must be an array'
    });
    return { isValid: false, invalidParams };
  }

  if (requestedNssai.length === 0) {
    return { isValid: true, invalidParams: [], validatedNssai: [] };
  }

  if (requestedNssai.length > MAX_REQUESTED_NSSAI) {
    invalidParams.push({
      param: paramPrefix,
      reason: `cannot exceed ${MAX_REQUESTED_NSSAI} S-NSSAI entries (per 3GPP TS 24.501)`
    });
    return { isValid: false, invalidParams };
  }

  for (let i = 0; i < requestedNssai.length; i++) {
    const snssaiError = validateSnssai(requestedNssai[i], `${paramPrefix}[${i}]`);
    if (snssaiError) {
      invalidParams.push(snssaiError);
    }
  }

  if (invalidParams.length > 0) {
    return { isValid: false, invalidParams };
  }

  if (hasDuplicates(requestedNssai)) {
    invalidParams.push({
      param: paramPrefix,
      reason: 'contains duplicate S-NSSAI entries'
    });
    return { isValid: false, invalidParams };
  }

  return {
    isValid: true,
    invalidParams: [],
    validatedNssai: requestedNssai
  };
};

export const processRequestedNssai = (
  requestedNssai: Snssai[] | undefined
): Snssai[] => {
  if (!requestedNssai || !Array.isArray(requestedNssai)) {
    return [];
  }

  const deduplicatedNssai = removeDuplicates(requestedNssai);

  return deduplicatedNssai.slice(0, MAX_REQUESTED_NSSAI);
};

export const filterRequestedNssaiBySubscription = (
  requestedNssai: Snssai[],
  subscribedNssai: Snssai[]
): { subscribed: Snssai[]; notSubscribed: Snssai[] } => {
  const subscribed: Snssai[] = [];
  const notSubscribed: Snssai[] = [];

  for (const requested of requestedNssai) {
    const isSubscribed = subscribedNssai.some(subscribed =>
      snssaiMatches(subscribed, requested)
    );

    if (isSubscribed) {
      subscribed.push(requested);
    } else {
      notSubscribed.push(requested);
    }
  }

  return { subscribed, notSubscribed };
};

export const prioritizeRequestedNssai = (
  requestedNssai: Snssai[],
  subscribedNssai: Array<{ subscribedSnssai: Snssai; defaultIndication?: boolean }>
): Snssai[] => {
  const prioritized: Snssai[] = [];
  const defaultSnssais: Snssai[] = [];
  const nonDefaultSnssais: Snssai[] = [];

  for (const requested of requestedNssai) {
    const subscription = subscribedNssai.find(sub =>
      snssaiMatches(sub.subscribedSnssai, requested)
    );

    if (subscription?.defaultIndication) {
      defaultSnssais.push(requested);
    } else {
      nonDefaultSnssais.push(requested);
    }
  }

  return [...defaultSnssais, ...nonDefaultSnssais];
};
