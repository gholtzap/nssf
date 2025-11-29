import { getCollection, handleDatabaseError, DatabaseError } from '../db/mongodb';
import {
  AuthorizedNetworkSliceInfo,
  AllowedNssai,
  AllowedSnssai,
  ConfiguredSnssai,
  SliceInfoForRegistration,
  SliceInfoForPDUSession,
  SliceInfoForUEConfigurationUpdate,
  RoamingIndication,
  NsiInformation,
  MappingOfSnssai
} from '../types/nnssf-nsselection-types';
import { Snssai, PlmnId, Tai, AccessType } from '../types/common-types';
import { SliceConfiguration, NsiConfiguration } from '../types/db-types';
import { getSubscriptionBySupi } from './subscription';
import { performAmfSelection } from './amf-selection';
import { determineAllowedNssai } from './allowed-nssai';
import { processMappingRequests, getMappingForSnssai, getReverseMappingForSnssai } from './snssai-mapping';

type NetworkSliceSelectionInput = {
  sliceInfoForRegistration: SliceInfoForRegistration;
  homePlmnId: PlmnId;
  supi: string;
  tai?: Tai;
};

type PduSessionSelectionInput = {
  sliceInfoForPDUSession: SliceInfoForPDUSession;
  homePlmnId: PlmnId;
  supi: string;
  tai?: Tai;
};

type UeConfigurationUpdateInput = {
  sliceInfoForUEConfigurationUpdate: SliceInfoForUEConfigurationUpdate;
  homePlmnId: PlmnId;
  supi: string;
  tai?: Tai;
};

const snssaiMatches = (s1: Snssai, s2: Snssai): boolean => {
  return s1.sst === s2.sst && s1.sd === s2.sd;
};

const plmnMatches = (p1: PlmnId, p2: PlmnId): boolean => {
  return p1.mcc === p2.mcc && p1.mnc === p2.mnc;
};

const isSliceAvailableInTai = (slice: SliceConfiguration, tai?: Tai): boolean => {
  if (!tai || !slice.taiList || slice.taiList.length === 0) {
    return true;
  }

  return slice.taiList.some(sliceTai =>
    plmnMatches(sliceTai.plmnId, tai.plmnId) && sliceTai.tac === tai.tac
  );
};

const isNsiAvailableInTai = (nsi: NsiConfiguration, tai?: Tai): boolean => {
  if (!tai || !nsi.taiList || nsi.taiList.length === 0) {
    return true;
  }

  return nsi.taiList.some(nsiTai =>
    plmnMatches(nsiTai.plmnId, tai.plmnId) && nsiTai.tac === tai.tac
  );
};

const selectNsiForSnssai = async (
  snssai: Snssai,
  plmnId: PlmnId,
  tai?: Tai
): Promise<NsiInformation[]> => {
  try {
    const nsiCollection = getCollection<NsiConfiguration>('nsi');

    const nsiConfigs = await nsiCollection.find({
      'snssai.sst': snssai.sst,
      'snssai.sd': snssai.sd,
      'plmnId.mcc': plmnId.mcc,
      'plmnId.mnc': plmnId.mnc
    }).toArray();

    const availableNsis = nsiConfigs.filter(nsi => isNsiAvailableInTai(nsi, tai));

    availableNsis.sort((a, b) => {
      if (a.priority !== b.priority) {
        return (b.priority || 0) - (a.priority || 0);
      }
      return (a.loadLevel || 0) - (b.loadLevel || 0);
    });

    return availableNsis.map(nsi => ({
      nrfId: nsi.nrfId,
      nsiId: nsi.nsiId,
      nrfNfMgtUri: nsi.nrfNfMgtUri,
      nrfAccessTokenUri: nsi.nrfAccessTokenUri,
      nrfOauth2Required: nsi.nrfOauth2Required
    }));
  } catch (error) {
    console.error('Error selecting NSI for S-NSSAI:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const selectNetworkSlicesForRegistration = async (
  input: NetworkSliceSelectionInput
): Promise<AuthorizedNetworkSliceInfo> => {
  const { sliceInfoForRegistration, homePlmnId, supi, tai } = input;

  try {
    const slicesCollection = getCollection<SliceConfiguration>('slices');

    const allowedNssaiList: AllowedNssai[] = [];
    const configuredNssai: ConfiguredSnssai[] = [];
    const rejectedNssaiInPlmn: Snssai[] = [];
    const rejectedNssaiInTa: Snssai[] = [];
    let mappingOfNssai: MappingOfSnssai[] | undefined;

    const subscription = await getSubscriptionBySupi(supi, homePlmnId);

    if (!subscription) {
      const requestedNssais = sliceInfoForRegistration.requestedNssai || [];
      return {
        rejectedNssaiInPlmn: requestedNssais.length > 0 ? requestedNssais : undefined
      };
    }

    const servingPlmnId = subscription.plmnId;
    const roamingIndication = sliceInfoForRegistration.roamingIndication || RoamingIndication.NON_ROAMING;
    const isRoaming = !plmnMatches(servingPlmnId, homePlmnId);
    const targetPlmnId = roamingIndication === RoamingIndication.LOCAL_BREAKOUT && isRoaming ? servingPlmnId : homePlmnId;

    if (sliceInfoForRegistration.requestMapping && sliceInfoForRegistration.sNssaiForMapping) {
      mappingOfNssai = await processMappingRequests(
        sliceInfoForRegistration.sNssaiForMapping,
        servingPlmnId,
        homePlmnId,
        tai
      );
    }

    const subscribedSnssais = subscription.subscribedSnssais;
    const requestedNssais = sliceInfoForRegistration.requestedNssai || [];
    const defaultConfiguredSnssaiInd = sliceInfoForRegistration.defaultConfiguredSnssaiInd;

    const availableSlices = await slicesCollection.find({}).toArray();

  const processedSnssais: Snssai[] = [];

  let snssaisToCheck: Snssai[];
  if (defaultConfiguredSnssaiInd && requestedNssais.length === 0) {
    snssaisToCheck = subscribedSnssais
      .filter(s => s.defaultIndication === true)
      .map(s => s.subscribedSnssai);
  } else if (requestedNssais.length > 0) {
    snssaisToCheck = requestedNssais;
  } else {
    snssaisToCheck = subscribedSnssais.map(s => s.subscribedSnssai);
  }

  for (const snssai of snssaisToCheck) {
    const isSubscribed = subscribedSnssais.some(s => snssaiMatches(s.subscribedSnssai, snssai));

    if (!isSubscribed && requestedNssais.length > 0) {
      rejectedNssaiInPlmn.push(snssai);
      continue;
    }

    const availableSlice = availableSlices.find((slice: SliceConfiguration) =>
      snssaiMatches(slice.snssai, snssai) && plmnMatches(slice.plmnId, targetPlmnId)
    );

    if (!availableSlice) {
      rejectedNssaiInPlmn.push(snssai);
      continue;
    }

    if (!isSliceAvailableInTai(availableSlice, tai)) {
      rejectedNssaiInTa.push(snssai);
      continue;
    }

    const policyResult = await determineAllowedNssai({
      snssai,
      plmnId: targetPlmnId,
      tai,
      subscription,
      slice: availableSlice
    });

    if (!policyResult.allowed) {
      rejectedNssaiInPlmn.push(snssai);
      continue;
    }

    processedSnssais.push(snssai);
  }

  if (processedSnssais.length > 0) {
    const allowedSnssaiList: AllowedSnssai[] = [];

    for (const snssai of processedSnssais) {
      const nsiInformationList = await selectNsiForSnssai(snssai, targetPlmnId, tai);

      const allowedSnssaiEntry: AllowedSnssai = {
        allowedSnssai: snssai,
        nsiInformationList: nsiInformationList.length > 0 ? nsiInformationList : undefined
      };

      if (roamingIndication === RoamingIndication.HOME_ROUTED_ROAMING && isRoaming) {
        const homeSnssai = await getMappingForSnssai(snssai, servingPlmnId, homePlmnId, tai);
        if (homeSnssai) {
          allowedSnssaiEntry.mappedHomeSnssai = homeSnssai;
        }
      }

      allowedSnssaiList.push(allowedSnssaiEntry);
    }

    allowedNssaiList.push({
      allowedSnssaiList,
      accessType: AccessType.THREE_GPP_ACCESS
    });
  }

  const snssaisForConfigured = defaultConfiguredSnssaiInd
    ? subscribedSnssais.filter(s => s.defaultIndication === true)
    : subscribedSnssais;

  for (const subscribedSnssai of snssaisForConfigured) {
    const slice = availableSlices.find((s: SliceConfiguration) =>
      snssaiMatches(s.snssai, subscribedSnssai.subscribedSnssai) && plmnMatches(s.plmnId, targetPlmnId)
    );

    if (slice && isSliceAvailableInTai(slice, tai)) {
      const configuredEntry: ConfiguredSnssai = {
        configuredSnssai: subscribedSnssai.subscribedSnssai
      };

      if (roamingIndication === RoamingIndication.HOME_ROUTED_ROAMING && isRoaming) {
        const homeSnssai = await getMappingForSnssai(
          subscribedSnssai.subscribedSnssai,
          servingPlmnId,
          homePlmnId,
          tai
        );
        if (homeSnssai) {
          configuredEntry.mappedHomeSnssai = homeSnssai;
        }
      } else if (roamingIndication === RoamingIndication.LOCAL_BREAKOUT && isRoaming) {
        configuredEntry.mappedHomeSnssai = subscribedSnssai.subscribedSnssai;
      }

      configuredNssai.push(configuredEntry);
    }
  }

    const result: AuthorizedNetworkSliceInfo = {
      allowedNssaiList: allowedNssaiList.length > 0 ? allowedNssaiList : undefined,
      configuredNssai: configuredNssai.length > 0 ? configuredNssai : undefined,
      rejectedNssaiInPlmn: rejectedNssaiInPlmn.length > 0 ? rejectedNssaiInPlmn : undefined,
      rejectedNssaiInTa: rejectedNssaiInTa.length > 0 ? rejectedNssaiInTa : undefined
    };

    if (mappingOfNssai && mappingOfNssai.length > 0) {
      result.mappingOfNssai = mappingOfNssai;
    }

    if (processedSnssais.length === 0 && requestedNssais.length > 0) {
      const amfSelectionResult = await performAmfSelection({
        targetSnssais: requestedNssais,
        plmnId: targetPlmnId,
        tai
      });

      if (amfSelectionResult) {
        result.targetAmfSet = amfSelectionResult.targetAmfSet;
        result.targetAmfServiceSet = amfSelectionResult.targetAmfServiceSet;
        result.candidateAmfList = amfSelectionResult.candidateAmfList?.map(c => c.nfInstanceId);
        result.nrfAmfSet = amfSelectionResult.nrfAmfSet;
        result.nrfAmfSetNfMgtUri = amfSelectionResult.nrfAmfSetNfMgtUri;
        result.nrfAmfSetAccessTokenUri = amfSelectionResult.nrfAmfSetAccessTokenUri;
        result.nrfOauth2Required = amfSelectionResult.nrfOauth2Required;
      }
    }

    return result;
  } catch (error) {
    console.error('Error in selectNetworkSlicesForRegistration:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const selectNetworkSlicesForPDUSession = async (
  input: PduSessionSelectionInput
): Promise<AuthorizedNetworkSliceInfo> => {
  const { sliceInfoForPDUSession, homePlmnId, supi, tai } = input;

  try {
    const slicesCollection = getCollection<SliceConfiguration>('slices');

    const requestedSnssai = sliceInfoForPDUSession.sNssai;
    const roamingIndication = sliceInfoForPDUSession.roamingIndication || RoamingIndication.NON_ROAMING;
    const homeSnssai = sliceInfoForPDUSession.homeSnssai;

    const subscription = await getSubscriptionBySupi(supi, homePlmnId);

  if (!subscription) {
    return {
      rejectedNssaiInPlmn: [requestedSnssai]
    };
  }

  const servingPlmnId = subscription.plmnId;
  const isRoaming = !plmnMatches(servingPlmnId, homePlmnId);

  let targetSnssai = requestedSnssai;
  let targetPlmnId = homePlmnId;

  if (roamingIndication === RoamingIndication.HOME_ROUTED_ROAMING && homeSnssai) {
    targetSnssai = homeSnssai;
    targetPlmnId = homePlmnId;
  } else if (roamingIndication === RoamingIndication.LOCAL_BREAKOUT && isRoaming) {
    targetPlmnId = servingPlmnId;
  }

  const isSubscribed = subscription.subscribedSnssais.some(s =>
    snssaiMatches(s.subscribedSnssai, targetSnssai)
  );

  if (!isSubscribed) {
    return {
      rejectedNssaiInPlmn: [requestedSnssai]
    };
  }

  const availableSlices = await slicesCollection.find({}).toArray();

  const availableSlice = availableSlices.find((slice: SliceConfiguration) =>
    snssaiMatches(slice.snssai, targetSnssai) && plmnMatches(slice.plmnId, targetPlmnId)
  );

  if (!availableSlice) {
    return {
      rejectedNssaiInPlmn: [requestedSnssai]
    };
  }

  if (!isSliceAvailableInTai(availableSlice, tai)) {
    return {
      rejectedNssaiInTa: [requestedSnssai]
    };
  }

  const policyResult = await determineAllowedNssai({
    snssai: targetSnssai,
    plmnId: targetPlmnId,
    tai,
    subscription,
    slice: availableSlice
  });

  if (!policyResult.allowed) {
    return {
      rejectedNssaiInPlmn: [requestedSnssai]
    };
  }

  const nsiInformationList = await selectNsiForSnssai(requestedSnssai, targetPlmnId, tai);

  const allowedSnssai: AllowedSnssai = {
    allowedSnssai: requestedSnssai,
    nsiInformationList: nsiInformationList.length > 0 ? nsiInformationList : undefined
  };

  if (roamingIndication === RoamingIndication.HOME_ROUTED_ROAMING && homeSnssai) {
    allowedSnssai.mappedHomeSnssai = homeSnssai;
  } else if (roamingIndication === RoamingIndication.LOCAL_BREAKOUT && isRoaming) {
    const homeSnssaiMapping = await getMappingForSnssai(requestedSnssai, servingPlmnId, homePlmnId, tai);
    if (homeSnssaiMapping) {
      allowedSnssai.mappedHomeSnssai = homeSnssaiMapping;
    }
  }

    const allowedNssai: AllowedNssai = {
      allowedSnssaiList: [allowedSnssai],
      accessType: AccessType.THREE_GPP_ACCESS
    };

    return {
      allowedNssaiList: [allowedNssai]
    };
  } catch (error) {
    console.error('Error in selectNetworkSlicesForPDUSession:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const selectNetworkSlicesForUEConfigurationUpdate = async (
  input: UeConfigurationUpdateInput
): Promise<AuthorizedNetworkSliceInfo> => {
  const { sliceInfoForUEConfigurationUpdate, homePlmnId, supi, tai } = input;

  try {
    const slicesCollection = getCollection<SliceConfiguration>('slices');

    const allowedNssaiList: AllowedNssai[] = [];
    const configuredNssai: ConfiguredSnssai[] = [];
    const rejectedNssaiInPlmn: Snssai[] = [];
    const rejectedNssaiInTa: Snssai[] = [];
    let mappingOfNssai: MappingOfSnssai[] | undefined;

  const subscription = await getSubscriptionBySupi(supi, homePlmnId);

  if (!subscription) {
    const requestedNssais = sliceInfoForUEConfigurationUpdate.requestedNssai || [];
    return {
      rejectedNssaiInPlmn: requestedNssais.length > 0 ? requestedNssais : undefined
    };
  }

  const servingPlmnId = subscription.plmnId;
  const roamingIndication = sliceInfoForUEConfigurationUpdate.roamingIndication || RoamingIndication.NON_ROAMING;
  const isRoaming = !plmnMatches(servingPlmnId, homePlmnId);
  const targetPlmnId = roamingIndication === RoamingIndication.LOCAL_BREAKOUT && isRoaming ? servingPlmnId : homePlmnId;

  const subscribedSnssais = subscription.subscribedSnssais;
  const requestedNssais = sliceInfoForUEConfigurationUpdate.requestedNssai || [];
  const rejectedNssaiRa = sliceInfoForUEConfigurationUpdate.rejectedNssaiRa || [];
  const defaultConfiguredSnssaiInd = sliceInfoForUEConfigurationUpdate.defaultConfiguredSnssaiInd;

  const availableSlices = await slicesCollection.find({}).toArray();

  const processedSnssais: Snssai[] = [];

  let snssaisToCheck: Snssai[];
  if (defaultConfiguredSnssaiInd && requestedNssais.length === 0) {
    snssaisToCheck = subscribedSnssais
      .filter(s => s.defaultIndication === true)
      .map(s => s.subscribedSnssai);
  } else if (requestedNssais.length > 0) {
    snssaisToCheck = requestedNssais;
  } else {
    snssaisToCheck = subscribedSnssais.map(s => s.subscribedSnssai);
  }

  for (const snssai of snssaisToCheck) {
    if (rejectedNssaiRa.some(rejected => snssaiMatches(rejected, snssai))) {
      rejectedNssaiInPlmn.push(snssai);
      continue;
    }

    const isSubscribed = subscribedSnssais.some(s => snssaiMatches(s.subscribedSnssai, snssai));

    if (!isSubscribed && requestedNssais.length > 0) {
      rejectedNssaiInPlmn.push(snssai);
      continue;
    }

    const availableSlice = availableSlices.find((slice: SliceConfiguration) =>
      snssaiMatches(slice.snssai, snssai) && plmnMatches(slice.plmnId, targetPlmnId)
    );

    if (!availableSlice) {
      rejectedNssaiInPlmn.push(snssai);
      continue;
    }

    if (!isSliceAvailableInTai(availableSlice, tai)) {
      rejectedNssaiInTa.push(snssai);
      continue;
    }

    const policyResult = await determineAllowedNssai({
      snssai,
      plmnId: targetPlmnId,
      tai,
      subscription,
      slice: availableSlice
    });

    if (!policyResult.allowed) {
      rejectedNssaiInPlmn.push(snssai);
      continue;
    }

    processedSnssais.push(snssai);
  }

  if (processedSnssais.length > 0) {
    const allowedSnssaiList: AllowedSnssai[] = [];

    for (const snssai of processedSnssais) {
      const nsiInformationList = await selectNsiForSnssai(snssai, targetPlmnId, tai);

      const allowedSnssaiEntry: AllowedSnssai = {
        allowedSnssai: snssai,
        nsiInformationList: nsiInformationList.length > 0 ? nsiInformationList : undefined
      };

      if (roamingIndication === RoamingIndication.HOME_ROUTED_ROAMING && isRoaming) {
        const homeSnssai = await getMappingForSnssai(snssai, servingPlmnId, homePlmnId, tai);
        if (homeSnssai) {
          allowedSnssaiEntry.mappedHomeSnssai = homeSnssai;
        }
      }

      allowedSnssaiList.push(allowedSnssaiEntry);
    }

    allowedNssaiList.push({
      allowedSnssaiList,
      accessType: AccessType.THREE_GPP_ACCESS
    });
  }

  const snssaisForConfigured = defaultConfiguredSnssaiInd
    ? subscribedSnssais.filter(s => s.defaultIndication === true)
    : subscribedSnssais;

  for (const subscribedSnssai of snssaisForConfigured) {
    const slice = availableSlices.find((s: SliceConfiguration) =>
      snssaiMatches(s.snssai, subscribedSnssai.subscribedSnssai) && plmnMatches(s.plmnId, targetPlmnId)
    );

    if (slice && isSliceAvailableInTai(slice, tai)) {
      const configuredEntry: ConfiguredSnssai = {
        configuredSnssai: subscribedSnssai.subscribedSnssai
      };

      if (roamingIndication === RoamingIndication.HOME_ROUTED_ROAMING && isRoaming) {
        const homeSnssai = await getMappingForSnssai(
          subscribedSnssai.subscribedSnssai,
          servingPlmnId,
          homePlmnId,
          tai
        );
        if (homeSnssai) {
          configuredEntry.mappedHomeSnssai = homeSnssai;
        }
      } else if (roamingIndication === RoamingIndication.LOCAL_BREAKOUT && isRoaming) {
        configuredEntry.mappedHomeSnssai = subscribedSnssai.subscribedSnssai;
      }

      configuredNssai.push(configuredEntry);
    }
  }

  if (sliceInfoForUEConfigurationUpdate.mappingOfNssai) {
    mappingOfNssai = sliceInfoForUEConfigurationUpdate.mappingOfNssai;
  }

    const result: AuthorizedNetworkSliceInfo = {
      allowedNssaiList: allowedNssaiList.length > 0 ? allowedNssaiList : undefined,
      configuredNssai: configuredNssai.length > 0 ? configuredNssai : undefined,
      rejectedNssaiInPlmn: rejectedNssaiInPlmn.length > 0 ? rejectedNssaiInPlmn : undefined,
      rejectedNssaiInTa: rejectedNssaiInTa.length > 0 ? rejectedNssaiInTa : undefined
    };

    if (mappingOfNssai && mappingOfNssai.length > 0) {
      result.mappingOfNssai = mappingOfNssai;
    }

    return result;
  } catch (error) {
    console.error('Error in selectNetworkSlicesForUEConfigurationUpdate:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};
