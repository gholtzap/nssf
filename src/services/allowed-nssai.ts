import { getCollection, handleDatabaseError } from '../db/mongodb';
import { SlicePolicy, UeSubscription, SliceConfiguration } from '../types/db-types';
import { Snssai, PlmnId, Tai } from '../types/common-types';

type PolicyEvaluationContext = {
  snssai: Snssai;
  plmnId: PlmnId;
  tai?: Tai;
  subscription?: UeSubscription;
  slice?: SliceConfiguration;
  currentTime?: Date;
};

type PolicyEvaluationResult = {
  allowed: boolean;
  reason?: string;
  policyId?: string;
};

type AllowedNssaiResult = {
  allowed: boolean;
  reasons: string[];
};

const snssaiMatches = (s1: Snssai, s2: Snssai): boolean => {
  return s1.sst === s2.sst && s1.sd === s2.sd;
};

const plmnMatches = (p1: PlmnId, p2: PlmnId): boolean => {
  return p1.mcc === p2.mcc && p1.mnc === p2.mnc;
};

const taiMatches = (t1: Tai, t2: Tai): boolean => {
  return plmnMatches(t1.plmnId, t2.plmnId) && t1.tac === t2.tac;
};

const isWithinTimeWindow = (currentTime: Date, startTime: string, endTime: string, daysOfWeek?: number[]): boolean => {
  if (daysOfWeek && daysOfWeek.length > 0) {
    const currentDay = currentTime.getDay();
    if (!daysOfWeek.includes(currentDay)) {
      return false;
    }
  }

  const currentTimeStr = currentTime.toTimeString().substring(0, 5);
  return currentTimeStr >= startTime && currentTimeStr <= endTime;
};

const evaluateTimePolicy = (policy: SlicePolicy, currentTime: Date): PolicyEvaluationResult => {
  if (policy.allowedTimeWindows && policy.allowedTimeWindows.length > 0) {
    const isInAllowedWindow = policy.allowedTimeWindows.some(window =>
      isWithinTimeWindow(currentTime, window.startTime, window.endTime, window.daysOfWeek)
    );

    if (!isInAllowedWindow) {
      return {
        allowed: false,
        reason: 'Outside allowed time window',
        policyId: policy.policyId
      };
    }
  }

  if (policy.deniedTimeWindows && policy.deniedTimeWindows.length > 0) {
    const isInDeniedWindow = policy.deniedTimeWindows.some(window =>
      isWithinTimeWindow(currentTime, window.startTime, window.endTime, window.daysOfWeek)
    );

    if (isInDeniedWindow) {
      return {
        allowed: false,
        reason: 'Within denied time window',
        policyId: policy.policyId
      };
    }
  }

  return { allowed: true };
};

const evaluateTaiPolicy = (policy: SlicePolicy, tai?: Tai): PolicyEvaluationResult => {
  if (!tai) {
    return { allowed: true };
  }

  if (policy.deniedTaiList && policy.deniedTaiList.length > 0) {
    const isInDeniedTai = policy.deniedTaiList.some(deniedTai => taiMatches(tai, deniedTai));
    if (isInDeniedTai) {
      return {
        allowed: false,
        reason: 'TAI is in denied list',
        policyId: policy.policyId
      };
    }
  }

  if (policy.allowedTaiList && policy.allowedTaiList.length > 0) {
    const isInAllowedTai = policy.allowedTaiList.some(allowedTai => taiMatches(tai, allowedTai));
    if (!isInAllowedTai) {
      return {
        allowed: false,
        reason: 'TAI is not in allowed list',
        policyId: policy.policyId
      };
    }
  }

  return { allowed: true };
};

const evaluateLoadPolicy = (policy: SlicePolicy, slice?: SliceConfiguration): PolicyEvaluationResult => {
  if (!slice) {
    return { allowed: true };
  }

  if (policy.maxLoadLevel !== undefined) {
    const currentLoadLevel = 0;

    if (currentLoadLevel > policy.maxLoadLevel) {
      return {
        allowed: false,
        reason: `Slice load (${currentLoadLevel}) exceeds maximum allowed (${policy.maxLoadLevel})`,
        policyId: policy.policyId
      };
    }
  }

  return { allowed: true };
};

const evaluateSubscriptionPolicy = (policy: SlicePolicy, subscription?: UeSubscription): PolicyEvaluationResult => {
  if (!subscription) {
    return { allowed: true };
  }

  return { allowed: true };
};

const evaluatePolicy = (policy: SlicePolicy, context: PolicyEvaluationContext): PolicyEvaluationResult => {
  if (!policy.enabled) {
    return { allowed: true };
  }

  const currentTime = context.currentTime || new Date();

  const timeResult = evaluateTimePolicy(policy, currentTime);
  if (!timeResult.allowed) {
    return timeResult;
  }

  const taiResult = evaluateTaiPolicy(policy, context.tai);
  if (!taiResult.allowed) {
    return taiResult;
  }

  const loadResult = evaluateLoadPolicy(policy, context.slice);
  if (!loadResult.allowed) {
    return loadResult;
  }

  const subscriptionResult = evaluateSubscriptionPolicy(policy, context.subscription);
  if (!subscriptionResult.allowed) {
    return subscriptionResult;
  }

  return { allowed: true };
};

export const determineAllowedNssai = async (context: PolicyEvaluationContext): Promise<AllowedNssaiResult> => {
  try {
    const policyCollection = getCollection<SlicePolicy>('policies');

    const policies = await policyCollection.find({
      'snssai.sst': context.snssai.sst,
      'snssai.sd': context.snssai.sd,
      'plmnId.mcc': context.plmnId.mcc,
      'plmnId.mnc': context.plmnId.mnc,
      enabled: true
    }).toArray();

    if (policies.length === 0) {
      return { allowed: true, reasons: [] };
    }

    const denialReasons: string[] = [];

    for (const policy of policies) {
      const result = evaluatePolicy(policy, context);
      if (!result.allowed && result.reason) {
        denialReasons.push(`Policy ${result.policyId}: ${result.reason}`);
      }
    }

    if (denialReasons.length > 0) {
      return {
        allowed: false,
        reasons: denialReasons
      };
    }

    return { allowed: true, reasons: [] };
  } catch (error) {
    console.error('Error determining allowed NSSAI:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const determineAllowedNssaiForMultiple = async (
  snssais: Snssai[],
  plmnId: PlmnId,
  tai?: Tai,
  subscription?: UeSubscription,
  slices?: Map<string, SliceConfiguration>
): Promise<Map<string, AllowedNssaiResult>> => {
  const results = new Map<string, AllowedNssaiResult>();

  for (const snssai of snssais) {
    const sliceKey = `${snssai.sst}-${snssai.sd || ''}`;
    const slice = slices?.get(sliceKey);

    const result = await determineAllowedNssai({
      snssai,
      plmnId,
      tai,
      subscription,
      slice
    });

    results.set(sliceKey, result);
  }

  return results;
};
