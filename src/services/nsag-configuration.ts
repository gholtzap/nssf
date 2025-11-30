import { getCollection } from '../db/mongodb';
import { NsagConfiguration } from '../types/db-types';
import { NsagId } from '../types/common-types';

export const getAllNsagConfigurations = async (): Promise<NsagConfiguration[]> => {
  const nsagCollection = getCollection<NsagConfiguration>('nsags');
  return await nsagCollection.find({}).toArray();
};

export const getNsagConfiguration = async (nsagId: NsagId): Promise<NsagConfiguration | null> => {
  const nsagCollection = getCollection<NsagConfiguration>('nsags');
  return await nsagCollection.findOne({ nsagId });
};

export const createNsagConfiguration = async (nsagConfig: NsagConfiguration): Promise<void> => {
  const nsagCollection = getCollection<NsagConfiguration>('nsags');

  const existing = await getNsagConfiguration(nsagConfig.nsagId);
  if (existing) {
    throw new Error('NSAG configuration already exists');
  }

  await nsagCollection.insertOne(nsagConfig);
};

export const updateNsagConfiguration = async (
  nsagId: NsagId,
  updates: Partial<NsagConfiguration>
): Promise<void> => {
  const nsagCollection = getCollection<NsagConfiguration>('nsags');

  const result = await nsagCollection.updateOne(
    { nsagId },
    { $set: updates }
  );

  if (result.matchedCount === 0) {
    throw new Error('NSAG configuration not found');
  }
};

export const deleteNsagConfiguration = async (nsagId: NsagId): Promise<void> => {
  const nsagCollection = getCollection<NsagConfiguration>('nsags');

  const result = await nsagCollection.deleteOne({ nsagId });

  if (result.deletedCount === 0) {
    throw new Error('NSAG configuration not found');
  }
};

export const incrementNsagUeCount = async (nsagId: NsagId): Promise<void> => {
  const nsagCollection = getCollection<NsagConfiguration>('nsags');

  await nsagCollection.updateOne(
    { nsagId },
    { $inc: { currentUeCount: 1 } }
  );
};

export const decrementNsagUeCount = async (nsagId: NsagId): Promise<void> => {
  const nsagCollection = getCollection<NsagConfiguration>('nsags');

  await nsagCollection.updateOne(
    { nsagId },
    {
      $inc: { currentUeCount: -1 },
      $max: { currentUeCount: 0 }
    }
  );
};
