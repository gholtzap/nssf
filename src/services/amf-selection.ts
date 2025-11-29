import { getCollection } from '../db/mongodb';
import { Snssai, PlmnId, Tai, Uri, NfInstanceId } from '../types/common-types';
import {
  AmfSetId,
  AmfServiceSetId,
  AmfCandidate,
  AmfSetConfig,
  AmfServiceSetConfig,
  AmfInstanceConfig
} from '../types/amf-selection-types';

type AmfSelectionInput = {
  targetSnssais: Snssai[];
  plmnId: PlmnId;
  tai?: Tai;
};

type AmfSelectionResult = {
  targetAmfSet?: AmfSetId;
  targetAmfServiceSet?: AmfServiceSetId;
  candidateAmfList?: AmfCandidate[];
  nrfAmfSet?: Uri;
  nrfAmfSetNfMgtUri?: Uri;
  nrfAmfSetAccessTokenUri?: Uri;
  nrfOauth2Required?: Record<string, boolean>;
};

const snssaiMatches = (s1: Snssai, s2: Snssai): boolean => {
  return s1.sst === s2.sst && s1.sd === s2.sd;
};

const plmnMatches = (p1: PlmnId, p2: PlmnId): boolean => {
  return p1.mcc === p2.mcc && p1.mnc === p2.mnc;
};

const supportsAllSnssais = (
  supportedSnssais: Snssai[],
  requiredSnssais: Snssai[]
): boolean => {
  return requiredSnssais.every(required =>
    supportedSnssais.some(supported => snssaiMatches(supported, required))
  );
};

export const selectTargetAmfSet = async (
  input: AmfSelectionInput
): Promise<AmfSelectionResult | null> => {
  const { targetSnssais, plmnId } = input;

  const amfSetCollection = getCollection<AmfSetConfig>('amf_sets');

  const amfSets = await amfSetCollection.find({
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  }).toArray();

  const compatibleSets = amfSets.filter(set =>
    supportsAllSnssais(set.supportedSnssais, targetSnssais)
  );

  if (compatibleSets.length === 0) {
    return null;
  }

  compatibleSets.sort((a, b) => {
    if (a.priority !== b.priority) {
      return (b.priority || 0) - (a.priority || 0);
    }
    return (a.capacity || 0) - (b.capacity || 0);
  });

  const selectedSet = compatibleSets[0];

  return {
    targetAmfSet: selectedSet.amfSetId,
    nrfAmfSet: selectedSet.nrfId,
    nrfAmfSetNfMgtUri: selectedSet.nrfNfMgtUri,
    nrfAmfSetAccessTokenUri: selectedSet.nrfAccessTokenUri,
    nrfOauth2Required: selectedSet.nrfOauth2Required
  };
};

export const selectTargetAmfServiceSet = async (
  input: AmfSelectionInput & { amfSetId: AmfSetId }
): Promise<AmfServiceSetId | null> => {
  const { targetSnssais, plmnId, amfSetId } = input;

  const amfServiceSetCollection = getCollection<AmfServiceSetConfig>('amf_service_sets');

  const serviceSets = await amfServiceSetCollection.find({
    amfSetId: amfSetId,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  }).toArray();

  const compatibleServiceSets = serviceSets.filter(serviceSet =>
    supportsAllSnssais(serviceSet.supportedSnssais, targetSnssais)
  );

  if (compatibleServiceSets.length === 0) {
    return null;
  }

  compatibleServiceSets.sort((a, b) => {
    return (b.priority || 0) - (a.priority || 0);
  });

  return compatibleServiceSets[0].amfServiceSetId;
};

export const generateCandidateAmfList = async (
  input: AmfSelectionInput & { amfSetId: AmfSetId; amfServiceSetId?: AmfServiceSetId }
): Promise<AmfCandidate[]> => {
  const { targetSnssais, plmnId, amfSetId, amfServiceSetId } = input;

  const amfInstanceCollection = getCollection<AmfInstanceConfig>('amf_instances');

  const query: any = {
    amfSetId: amfSetId,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  };

  if (amfServiceSetId) {
    query.amfServiceSetId = amfServiceSetId;
  }

  const instances = await amfInstanceCollection.find(query).toArray();

  const compatibleInstances = instances.filter(instance =>
    supportsAllSnssais(instance.supportedSnssais, targetSnssais)
  );

  compatibleInstances.sort((a, b) => {
    if (a.capacity !== b.capacity) {
      return (b.capacity || 0) - (a.capacity || 0);
    }
    return (a.loadLevel || 0) - (b.loadLevel || 0);
  });

  return compatibleInstances.map(instance => ({
    nfInstanceId: instance.nfInstanceId,
    amfSetId: instance.amfSetId,
    amfServiceSetId: instance.amfServiceSetId,
    guami: instance.guami
  }));
};

export const performAmfSelection = async (
  input: AmfSelectionInput
): Promise<AmfSelectionResult | null> => {
  const amfSetResult = await selectTargetAmfSet(input);

  if (!amfSetResult || !amfSetResult.targetAmfSet) {
    return null;
  }

  const amfServiceSetId = await selectTargetAmfServiceSet({
    ...input,
    amfSetId: amfSetResult.targetAmfSet
  });

  const candidateAmfList = await generateCandidateAmfList({
    ...input,
    amfSetId: amfSetResult.targetAmfSet,
    amfServiceSetId: amfServiceSetId || undefined
  });

  return {
    ...amfSetResult,
    targetAmfServiceSet: amfServiceSetId || undefined,
    candidateAmfList: candidateAmfList.length > 0 ? candidateAmfList : undefined
  };
};
