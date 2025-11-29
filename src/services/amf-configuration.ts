import { getCollection } from '../db/mongodb';
import {
  AmfSetConfig,
  AmfServiceSetConfig,
  AmfInstanceConfig,
  AmfSetId,
  AmfServiceSetId
} from '../types/amf-selection-types';
import { NfInstanceId, PlmnId } from '../types/common-types';

export const getAllAmfSets = async (): Promise<AmfSetConfig[]> => {
  const amfSetCollection = getCollection<AmfSetConfig>('amf_sets');
  return await amfSetCollection.find({}).toArray();
};

export const getAmfSet = async (
  amfSetId: AmfSetId,
  plmnId: PlmnId
): Promise<AmfSetConfig | null> => {
  const amfSetCollection = getCollection<AmfSetConfig>('amf_sets');

  return await amfSetCollection.findOne({
    amfSetId,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  });
};

export const createAmfSet = async (amfSetConfig: AmfSetConfig): Promise<void> => {
  const amfSetCollection = getCollection<AmfSetConfig>('amf_sets');

  const existing = await getAmfSet(amfSetConfig.amfSetId, amfSetConfig.plmnId);
  if (existing) {
    throw new Error('AMF Set already exists for this AMF Set ID and PLMN');
  }

  await amfSetCollection.insertOne(amfSetConfig);
};

export const updateAmfSet = async (
  amfSetId: AmfSetId,
  plmnId: PlmnId,
  updates: Partial<AmfSetConfig>
): Promise<void> => {
  const amfSetCollection = getCollection<AmfSetConfig>('amf_sets');

  const result = await amfSetCollection.updateOne(
    {
      amfSetId,
      'plmnId.mcc': plmnId.mcc,
      'plmnId.mnc': plmnId.mnc
    },
    { $set: updates }
  );

  if (result.matchedCount === 0) {
    throw new Error('AMF Set not found');
  }
};

export const deleteAmfSet = async (amfSetId: AmfSetId, plmnId: PlmnId): Promise<void> => {
  const amfSetCollection = getCollection<AmfSetConfig>('amf_sets');

  const result = await amfSetCollection.deleteOne({
    amfSetId,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  });

  if (result.deletedCount === 0) {
    throw new Error('AMF Set not found');
  }
};

export const getAllAmfServiceSets = async (): Promise<AmfServiceSetConfig[]> => {
  const amfServiceSetCollection = getCollection<AmfServiceSetConfig>('amf_service_sets');
  return await amfServiceSetCollection.find({}).toArray();
};

export const getAmfServiceSet = async (
  amfServiceSetId: AmfServiceSetId,
  amfSetId: AmfSetId,
  plmnId: PlmnId
): Promise<AmfServiceSetConfig | null> => {
  const amfServiceSetCollection = getCollection<AmfServiceSetConfig>('amf_service_sets');

  return await amfServiceSetCollection.findOne({
    amfServiceSetId,
    amfSetId,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  });
};

export const createAmfServiceSet = async (
  amfServiceSetConfig: AmfServiceSetConfig
): Promise<void> => {
  const amfServiceSetCollection = getCollection<AmfServiceSetConfig>('amf_service_sets');

  const existing = await getAmfServiceSet(
    amfServiceSetConfig.amfServiceSetId,
    amfServiceSetConfig.amfSetId,
    amfServiceSetConfig.plmnId
  );
  if (existing) {
    throw new Error('AMF Service Set already exists');
  }

  await amfServiceSetCollection.insertOne(amfServiceSetConfig);
};

export const updateAmfServiceSet = async (
  amfServiceSetId: AmfServiceSetId,
  amfSetId: AmfSetId,
  plmnId: PlmnId,
  updates: Partial<AmfServiceSetConfig>
): Promise<void> => {
  const amfServiceSetCollection = getCollection<AmfServiceSetConfig>('amf_service_sets');

  const result = await amfServiceSetCollection.updateOne(
    {
      amfServiceSetId,
      amfSetId,
      'plmnId.mcc': plmnId.mcc,
      'plmnId.mnc': plmnId.mnc
    },
    { $set: updates }
  );

  if (result.matchedCount === 0) {
    throw new Error('AMF Service Set not found');
  }
};

export const deleteAmfServiceSet = async (
  amfServiceSetId: AmfServiceSetId,
  amfSetId: AmfSetId,
  plmnId: PlmnId
): Promise<void> => {
  const amfServiceSetCollection = getCollection<AmfServiceSetConfig>('amf_service_sets');

  const result = await amfServiceSetCollection.deleteOne({
    amfServiceSetId,
    amfSetId,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  });

  if (result.deletedCount === 0) {
    throw new Error('AMF Service Set not found');
  }
};

export const getAllAmfInstances = async (): Promise<AmfInstanceConfig[]> => {
  const amfInstanceCollection = getCollection<AmfInstanceConfig>('amf_instances');
  return await amfInstanceCollection.find({}).toArray();
};

export const getAmfInstance = async (
  nfInstanceId: NfInstanceId
): Promise<AmfInstanceConfig | null> => {
  const amfInstanceCollection = getCollection<AmfInstanceConfig>('amf_instances');
  return await amfInstanceCollection.findOne({ nfInstanceId });
};

export const createAmfInstance = async (
  amfInstanceConfig: AmfInstanceConfig
): Promise<void> => {
  const amfInstanceCollection = getCollection<AmfInstanceConfig>('amf_instances');

  const existing = await getAmfInstance(amfInstanceConfig.nfInstanceId);
  if (existing) {
    throw new Error('AMF Instance already exists for this NF Instance ID');
  }

  await amfInstanceCollection.insertOne(amfInstanceConfig);
};

export const updateAmfInstance = async (
  nfInstanceId: NfInstanceId,
  updates: Partial<AmfInstanceConfig>
): Promise<void> => {
  const amfInstanceCollection = getCollection<AmfInstanceConfig>('amf_instances');

  const result = await amfInstanceCollection.updateOne(
    { nfInstanceId },
    { $set: updates }
  );

  if (result.matchedCount === 0) {
    throw new Error('AMF Instance not found');
  }
};

export const deleteAmfInstance = async (nfInstanceId: NfInstanceId): Promise<void> => {
  const amfInstanceCollection = getCollection<AmfInstanceConfig>('amf_instances');

  const result = await amfInstanceCollection.deleteOne({ nfInstanceId });

  if (result.deletedCount === 0) {
    throw new Error('AMF Instance not found');
  }
};
