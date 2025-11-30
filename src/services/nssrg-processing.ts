import { getAllNssrgConfigurations } from './nssrg-configuration';
import { NssrgConfiguration } from '../types/db-types';
import { Snssai, Tai, PlmnId, NsSrg } from '../types/common-types';

type NssrgAssignmentResult = {
  assigned: boolean;
  nssrgId?: NsSrg;
  reason?: string;
};

const snssaiMatches = (s1: Snssai, s2: Snssai): boolean => {
  return s1.sst === s2.sst && s1.sd === s2.sd;
};

const plmnMatches = (p1: PlmnId, p2: PlmnId): boolean => {
  return p1.mcc === p2.mcc && p1.mnc === p2.mnc;
};

const taiMatches = (tai: Tai, nssrg: NssrgConfiguration): boolean => {
  if (!nssrg.taiList && !nssrg.taiRangeList) {
    return true;
  }

  if (nssrg.taiList) {
    for (const nssrgTai of nssrg.taiList) {
      if (
        plmnMatches(tai.plmnId, nssrgTai.plmnId) &&
        tai.tac === nssrgTai.tac
      ) {
        return true;
      }
    }
  }

  if (nssrg.taiRangeList) {
    for (const taiRange of nssrg.taiRangeList) {
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

export const assignNssrg = async (
  snssai: Snssai,
  plmnId: PlmnId,
  tai?: Tai
): Promise<NssrgAssignmentResult> => {
  const nssrgs = await getAllNssrgConfigurations();

  const matchingNssrgs = nssrgs.filter(nssrg => {
    if (!nssrg.enabled) {
      return false;
    }

    if (!plmnMatches(nssrg.plmnId, plmnId)) {
      return false;
    }

    const hasMatchingSnssai = nssrg.snssaiList.some(s => snssaiMatches(s, snssai));
    if (!hasMatchingSnssai) {
      return false;
    }

    if (tai && !taiMatches(tai, nssrg)) {
      return false;
    }

    return true;
  });

  if (matchingNssrgs.length === 0) {
    return {
      assigned: false,
      reason: 'No matching NSSRG found'
    };
  }

  const sortedNssrgs = matchingNssrgs.sort((a, b) => {
    return (b.priority || 0) - (a.priority || 0);
  });

  for (const nssrg of sortedNssrgs) {
    if (nssrg.maxUeCount === undefined) {
      return {
        assigned: true,
        nssrgId: nssrg.nssrgId
      };
    }

    const currentCount = nssrg.currentUeCount || 0;
    if (currentCount < nssrg.maxUeCount) {
      return {
        assigned: true,
        nssrgId: nssrg.nssrgId
      };
    }
  }

  return {
    assigned: false,
    reason: 'NSSRG capacity exceeded'
  };
};

export const getNssrgForSnssai = async (
  snssai: Snssai,
  plmnId: PlmnId,
  tai?: Tai
): Promise<NssrgConfiguration | null> => {
  const nssrgs = await getAllNssrgConfigurations();

  const matchingNssrgs = nssrgs.filter(nssrg => {
    if (!nssrg.enabled) {
      return false;
    }

    if (!plmnMatches(nssrg.plmnId, plmnId)) {
      return false;
    }

    const hasMatchingSnssai = nssrg.snssaiList.some(s => snssaiMatches(s, snssai));
    if (!hasMatchingSnssai) {
      return false;
    }

    if (tai && !taiMatches(tai, nssrg)) {
      return false;
    }

    return true;
  });

  if (matchingNssrgs.length === 0) {
    return null;
  }

  const sortedNssrgs = matchingNssrgs.sort((a, b) => {
    return (b.priority || 0) - (a.priority || 0);
  });

  return sortedNssrgs[0];
};
