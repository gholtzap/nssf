import { getCollection } from '../db/mongodb';
import { NssrgConfiguration } from '../types/db-types';
import { NsSrg } from '../types/common-types';

export const getAllNssrgConfigurations = async (): Promise<NssrgConfiguration[]> => {
  const nssrgCollection = getCollection<NssrgConfiguration>('nssrgs');
  return await nssrgCollection.find({}).toArray();
};

export const getNssrgConfiguration = async (nssrgId: NsSrg): Promise<NssrgConfiguration | null> => {
  const nssrgCollection = getCollection<NssrgConfiguration>('nssrgs');
  return await nssrgCollection.findOne({ nssrgId });
};

export const createNssrgConfiguration = async (nssrgConfig: NssrgConfiguration): Promise<void> => {
  const nssrgCollection = getCollection<NssrgConfiguration>('nssrgs');

  const existing = await getNssrgConfiguration(nssrgConfig.nssrgId);
  if (existing) {
    throw new Error('NSSRG configuration already exists');
  }

  await nssrgCollection.insertOne(nssrgConfig);
};

export const updateNssrgConfiguration = async (
  nssrgId: NsSrg,
  updates: Partial<NssrgConfiguration>
): Promise<void> => {
  const nssrgCollection = getCollection<NssrgConfiguration>('nssrgs');

  const result = await nssrgCollection.updateOne(
    { nssrgId },
    { $set: updates }
  );

  if (result.matchedCount === 0) {
    throw new Error('NSSRG configuration not found');
  }
};

export const deleteNssrgConfiguration = async (nssrgId: NsSrg): Promise<void> => {
  const nssrgCollection = getCollection<NssrgConfiguration>('nssrgs');

  const result = await nssrgCollection.deleteOne({ nssrgId });

  if (result.deletedCount === 0) {
    throw new Error('NSSRG configuration not found');
  }
};

export const incrementNssrgUeCount = async (nssrgId: NsSrg): Promise<void> => {
  const nssrgCollection = getCollection<NssrgConfiguration>('nssrgs');

  await nssrgCollection.updateOne(
    { nssrgId },
    { $inc: { currentUeCount: 1 } }
  );
};

export const decrementNssrgUeCount = async (nssrgId: NsSrg): Promise<void> => {
  const nssrgCollection = getCollection<NssrgConfiguration>('nssrgs');

  await nssrgCollection.updateOne(
    { nssrgId },
    {
      $inc: { currentUeCount: -1 },
      $max: { currentUeCount: 0 }
    }
  );
};
