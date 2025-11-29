import { getCollection } from '../db/mongodb';
import { SliceConfiguration } from '../types/db-types';
import { Snssai, PlmnId } from '../types/common-types';

export const getAllSliceConfigurations = async (): Promise<SliceConfiguration[]> => {
  const slicesCollection = getCollection<SliceConfiguration>('slices');
  return await slicesCollection.find({}).toArray();
};

export const getSliceConfiguration = async (
  snssai: Snssai,
  plmnId: PlmnId
): Promise<SliceConfiguration | null> => {
  const slicesCollection = getCollection<SliceConfiguration>('slices');

  const slice = await slicesCollection.findOne({
    'snssai.sst': snssai.sst,
    'snssai.sd': snssai.sd,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  });

  return slice;
};

export const createSliceConfiguration = async (
  sliceConfig: SliceConfiguration
): Promise<void> => {
  const slicesCollection = getCollection<SliceConfiguration>('slices');

  const existing = await getSliceConfiguration(sliceConfig.snssai, sliceConfig.plmnId);
  if (existing) {
    throw new Error('Slice configuration already exists for this S-NSSAI and PLMN');
  }

  await slicesCollection.insertOne(sliceConfig);
};

export const updateSliceConfiguration = async (
  snssai: Snssai,
  plmnId: PlmnId,
  updates: Partial<SliceConfiguration>
): Promise<void> => {
  const slicesCollection = getCollection<SliceConfiguration>('slices');

  const result = await slicesCollection.updateOne(
    {
      'snssai.sst': snssai.sst,
      'snssai.sd': snssai.sd,
      'plmnId.mcc': plmnId.mcc,
      'plmnId.mnc': plmnId.mnc
    },
    { $set: updates }
  );

  if (result.matchedCount === 0) {
    throw new Error('Slice configuration not found');
  }
};

export const deleteSliceConfiguration = async (
  snssai: Snssai,
  plmnId: PlmnId
): Promise<void> => {
  const slicesCollection = getCollection<SliceConfiguration>('slices');

  const result = await slicesCollection.deleteOne({
    'snssai.sst': snssai.sst,
    'snssai.sd': snssai.sd,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  });

  if (result.deletedCount === 0) {
    throw new Error('Slice configuration not found');
  }
};
