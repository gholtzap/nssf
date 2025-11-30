import { getAllNsagConfigurations } from './nsag-configuration';
import { NsagConfiguration } from '../types/db-types';
import { Snssai, Tai, PlmnId, NsagId } from '../types/common-types';

type AdmissionResult = {
  admitted: boolean;
  nsagId?: NsagId;
  reason?: string;
};

const snssaiMatches = (s1: Snssai, s2: Snssai): boolean => {
  return s1.sst === s2.sst && s1.sd === s2.sd;
};

const plmnMatches = (p1: PlmnId, p2: PlmnId): boolean => {
  return p1.mcc === p2.mcc && p1.mnc === p2.mnc;
};

const taiMatches = (tai: Tai, nsag: NsagConfiguration): boolean => {
  if (!nsag.taiList && !nsag.taiRangeList) {
    return true;
  }

  if (nsag.taiList) {
    for (const nsagTai of nsag.taiList) {
      if (
        plmnMatches(tai.plmnId, nsagTai.plmnId) &&
        tai.tac === nsagTai.tac
      ) {
        return true;
      }
    }
  }

  if (nsag.taiRangeList) {
    for (const taiRange of nsag.taiRangeList) {
      if (
        plmnMatches(tai.plmnId, taiRange.plmnId) &&
        tai.tac >= taiRange.start &&
        tai.tac <= taiRange.end
      ) {
        return true;
      }
    }
  }

  return false;
};

export const checkNsagAdmission = async (
  snssai: Snssai,
  plmnId: PlmnId,
  tai?: Tai
): Promise<AdmissionResult> => {
  const nsags = await getAllNsagConfigurations();

  const matchingNsags = nsags.filter(nsag => {
    if (!nsag.enabled) {
      return false;
    }

    if (!plmnMatches(nsag.plmnId, plmnId)) {
      return false;
    }

    const hasMatchingSnssai = nsag.snssaiList.some(s => snssaiMatches(s, snssai));
    if (!hasMatchingSnssai) {
      return false;
    }

    if (tai && !taiMatches(tai, nsag)) {
      return false;
    }

    return true;
  });

  if (matchingNsags.length === 0) {
    return {
      admitted: true
    };
  }

  const sortedNsags = matchingNsags.sort((a, b) => {
    return (b.priority || 0) - (a.priority || 0);
  });

  for (const nsag of sortedNsags) {
    if (nsag.maxUeCount === undefined) {
      return {
        admitted: true,
        nsagId: nsag.nsagId
      };
    }

    const currentCount = nsag.currentUeCount || 0;
    if (currentCount < nsag.maxUeCount) {
      return {
        admitted: true,
        nsagId: nsag.nsagId
      };
    }
  }

  return {
    admitted: false,
    reason: 'NSAG capacity exceeded'
  };
};

export const getNsagForSnssai = async (
  snssai: Snssai,
  plmnId: PlmnId,
  tai?: Tai
): Promise<NsagConfiguration | null> => {
  const nsags = await getAllNsagConfigurations();

  const matchingNsags = nsags.filter(nsag => {
    if (!nsag.enabled) {
      return false;
    }

    if (!plmnMatches(nsag.plmnId, plmnId)) {
      return false;
    }

    const hasMatchingSnssai = nsag.snssaiList.some(s => snssaiMatches(s, snssai));
    if (!hasMatchingSnssai) {
      return false;
    }

    if (tai && !taiMatches(tai, nsag)) {
      return false;
    }

    return true;
  });

  if (matchingNsags.length === 0) {
    return null;
  }

  const sortedNsags = matchingNsags.sort((a, b) => {
    return (b.priority || 0) - (a.priority || 0);
  });

  return sortedNsags[0];
};
