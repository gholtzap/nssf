import { getCollection } from '../db/mongodb';
import { NsiConfiguration } from '../types/db-types';
import { Snssai, PlmnId } from '../types/common-types';
import { NsiId } from '../types/nnssf-nsselection-types';

export const getAllNsiConfigurations = async (): Promise<NsiConfiguration[]> => {
  const nsiCollection = getCollection<NsiConfiguration>('nsi');
  return await nsiCollection.find({}).toArray();
};

export const getNsiConfiguration = async (nsiId: NsiId): Promise<NsiConfiguration | null> => {
  const nsiCollection = getCollection<NsiConfiguration>('nsi');
  return await nsiCollection.findOne({ nsiId });
};

export const getNsiConfigurationsBySnssai = async (
  snssai: Snssai,
  plmnId: PlmnId
): Promise<NsiConfiguration[]> => {
  const nsiCollection = getCollection<NsiConfiguration>('nsi');

  return await nsiCollection.find({
    'snssai.sst': snssai.sst,
    'snssai.sd': snssai.sd,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  }).toArray();
};

export const createNsiConfiguration = async (
  nsiConfig: NsiConfiguration
): Promise<void> => {
  const nsiCollection = getCollection<NsiConfiguration>('nsi');

  const existing = await getNsiConfiguration(nsiConfig.nsiId);
  if (existing) {
    throw new Error('NSI configuration already exists for this NSI ID');
  }

  await nsiCollection.insertOne(nsiConfig);
};

export const updateNsiConfiguration = async (
  nsiId: NsiId,
  updates: Partial<NsiConfiguration>
): Promise<void> => {
  const nsiCollection = getCollection<NsiConfiguration>('nsi');

  const result = await nsiCollection.updateOne(
    { nsiId },
    { $set: updates }
  );

  if (result.matchedCount === 0) {
    throw new Error('NSI configuration not found');
  }
};

export const deleteNsiConfiguration = async (nsiId: NsiId): Promise<void> => {
  const nsiCollection = getCollection<NsiConfiguration>('nsi');

  const result = await nsiCollection.deleteOne({ nsiId });

  if (result.deletedCount === 0) {
    throw new Error('NSI configuration not found');
  }
};
