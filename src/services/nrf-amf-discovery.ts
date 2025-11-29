import { discoverAmfInstances, getNfProfile } from './nrf-client';
import { NFProfile, NfStatus } from '../types/nrf-types';
import { Snssai, PlmnId, Tai, Uri, NfInstanceId } from '../types/common-types';
import { AmfCandidate } from '../types/amf-selection-types';

type NrfAmfDiscoveryInput = {
  nrfId: Uri;
  nrfNfMgtUri?: Uri;
  nrfAccessTokenUri?: Uri;
  nrfOauth2Required?: Record<string, boolean>;
  nfInstanceId: NfInstanceId;
  targetSnssais: Snssai[];
  plmnId: PlmnId;
  tai?: Tai;
  amfSetId?: string;
  amfRegionId?: string;
  targetNsiList?: string[];
};

type NrfAmfDiscoveryResult = {
  candidateAmfList: AmfCandidate[];
  discoveredProfiles: NFProfile[];
};

const isOauth2RequiredForService = (
  oauth2Config?: Record<string, boolean>,
  serviceName: string = 'nnrf-disc'
): boolean => {
  if (!oauth2Config) {
    return false;
  }
  return oauth2Config[serviceName] === true;
};

const nfProfileToAmfCandidate = (profile: NFProfile): AmfCandidate | null => {
  if (!profile.amfInfo || !profile.amfInfo.amfSetId) {
    return null;
  }

  const guami = profile.amfInfo.guamiList && profile.amfInfo.guamiList.length > 0
    ? profile.amfInfo.guamiList[0]
    : undefined;

  return {
    nfInstanceId: profile.nfInstanceId,
    amfSetId: profile.amfInfo.amfSetId,
    guami: guami
  };
};

const filterActiveAmfProfiles = (profiles: NFProfile[]): NFProfile[] => {
  return profiles.filter(profile =>
    profile.nfStatus === NfStatus.REGISTERED &&
    profile.amfInfo !== undefined
  );
};

const rankAmfProfiles = (profiles: NFProfile[]): NFProfile[] => {
  const rankedProfiles = [...profiles];

  rankedProfiles.sort((a, b) => {
    if (a.priority !== b.priority) {
      return (b.priority || 0) - (a.priority || 0);
    }

    if (a.capacity !== b.capacity) {
      return (b.capacity || 0) - (a.capacity || 0);
    }

    return (a.load || 0) - (b.load || 0);
  });

  return rankedProfiles;
};

export const discoverAmfViaNrf = async (
  input: NrfAmfDiscoveryInput
): Promise<NrfAmfDiscoveryResult | null> => {
  const {
    nrfId,
    nrfNfMgtUri,
    nrfAccessTokenUri,
    nrfOauth2Required,
    nfInstanceId,
    targetSnssais,
    plmnId,
    tai,
    amfSetId,
    amfRegionId,
    targetNsiList
  } = input;

  if (!nrfNfMgtUri) {
    return null;
  }

  const oauth2Required = isOauth2RequiredForService(nrfOauth2Required, 'nnrf-disc');

  const discoveredProfiles = await discoverAmfInstances(
    {
      nrfId,
      nrfNfMgtUri,
      nrfAccessTokenUri,
      nfInstanceId
    },
    {
      targetPlmnList: [plmnId],
      targetSnssaiList: targetSnssais,
      taiList: tai ? [tai] : undefined,
      amfSetId,
      amfRegionId,
      targetNsiList
    },
    oauth2Required
  );

  const activeProfiles = filterActiveAmfProfiles(discoveredProfiles);

  if (activeProfiles.length === 0) {
    return null;
  }

  const rankedProfiles = rankAmfProfiles(activeProfiles);

  const candidateAmfList: AmfCandidate[] = rankedProfiles
    .map(nfProfileToAmfCandidate)
    .filter((candidate): candidate is AmfCandidate => candidate !== null);

  return {
    candidateAmfList,
    discoveredProfiles: rankedProfiles
  };
};

export const getAmfProfileFromNrf = async (
  nrfId: Uri,
  nrfNfMgtUri: Uri | undefined,
  nrfAccessTokenUri: Uri | undefined,
  nrfOauth2Required: Record<string, boolean> | undefined,
  nfInstanceId: NfInstanceId,
  targetNfInstanceId: NfInstanceId
): Promise<NFProfile | null> => {
  if (!nrfNfMgtUri) {
    return null;
  }

  const oauth2Required = isOauth2RequiredForService(nrfOauth2Required, 'nnrf-nfm');

  return await getNfProfile(
    {
      nrfId,
      nrfNfMgtUri,
      nrfAccessTokenUri,
      nfInstanceId
    },
    targetNfInstanceId,
    oauth2Required
  );
};
