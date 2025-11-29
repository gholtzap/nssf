import { getCollection, handleDatabaseError } from '../db/mongodb';
import { SnssaiMapping } from '../types/db-types';
import { Snssai, PlmnId, Tai } from '../types/common-types';
import { MappingOfSnssai } from '../types/nnssf-nsselection-types';

const snssaiMatches = (s1: Snssai, s2: Snssai): boolean => {
  return s1.sst === s2.sst && s1.sd === s2.sd;
};

const plmnMatches = (p1: PlmnId, p2: PlmnId): boolean => {
  return p1.mcc === p2.mcc && p1.mnc === p2.mnc;
};

const taiMatches = (t1: Tai, t2: Tai): boolean => {
  return plmnMatches(t1.plmnId, t2.plmnId) && t1.tac === t2.tac;
};

const isMappingValidInTai = (mapping: SnssaiMapping, tai?: Tai): boolean => {
  if (!tai || !mapping.validityArea || mapping.validityArea.length === 0) {
    return true;
  }

  return mapping.validityArea.some(validTai => taiMatches(validTai, tai));
};

export const getMappingForSnssai = async (
  servingSnssai: Snssai,
  servingPlmnId: PlmnId,
  homePlmnId: PlmnId,
  tai?: Tai
): Promise<Snssai | null> => {
  try {
    const mappingCollection = getCollection<SnssaiMapping>('snssai-mappings');

    const mappings = await mappingCollection.find({
      'servingSnssai.sst': servingSnssai.sst,
      'servingSnssai.sd': servingSnssai.sd,
      'servingPlmnId.mcc': servingPlmnId.mcc,
      'servingPlmnId.mnc': servingPlmnId.mnc,
      'homePlmnId.mcc': homePlmnId.mcc,
      'homePlmnId.mnc': homePlmnId.mnc
    }).toArray();

    for (const mapping of mappings) {
      if (isMappingValidInTai(mapping, tai)) {
        return mapping.homeSnssai;
      }
    }

    return null;
  } catch (error) {
    console.error('Error getting mapping for S-NSSAI:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const getReverseMappingForSnssai = async (
  homeSnssai: Snssai,
  servingPlmnId: PlmnId,
  homePlmnId: PlmnId,
  tai?: Tai
): Promise<Snssai | null> => {
  try {
    const mappingCollection = getCollection<SnssaiMapping>('snssai-mappings');

    const mappings = await mappingCollection.find({
      'homeSnssai.sst': homeSnssai.sst,
      'homeSnssai.sd': homeSnssai.sd,
      'servingPlmnId.mcc': servingPlmnId.mcc,
      'servingPlmnId.mnc': servingPlmnId.mnc,
      'homePlmnId.mcc': homePlmnId.mcc,
      'homePlmnId.mnc': homePlmnId.mnc
    }).toArray();

    for (const mapping of mappings) {
      if (isMappingValidInTai(mapping, tai)) {
        return mapping.servingSnssai;
      }
    }

    return null;
  } catch (error) {
    console.error('Error getting reverse mapping for S-NSSAI:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const processMappingRequests = async (
  snssaisForMapping: Snssai[],
  servingPlmnId: PlmnId,
  homePlmnId: PlmnId,
  tai?: Tai
): Promise<MappingOfSnssai[]> => {
  const mappings: MappingOfSnssai[] = [];

  for (const servingSnssai of snssaisForMapping) {
    const homeSnssai = await getMappingForSnssai(
      servingSnssai,
      servingPlmnId,
      homePlmnId,
      tai
    );

    if (homeSnssai) {
      mappings.push({
        servingSnssai,
        homeSnssai
      });
    }
  }

  return mappings;
};

export const createSnssaiMapping = async (mapping: Omit<SnssaiMapping, 'mappingId'>): Promise<SnssaiMapping> => {
  try {
    const mappingCollection = getCollection<SnssaiMapping>('snssai-mappings');

    const existingMapping = await mappingCollection.findOne({
      'servingSnssai.sst': mapping.servingSnssai.sst,
      'servingSnssai.sd': mapping.servingSnssai.sd,
      'servingPlmnId.mcc': mapping.servingPlmnId.mcc,
      'servingPlmnId.mnc': mapping.servingPlmnId.mnc,
      'homePlmnId.mcc': mapping.homePlmnId.mcc,
      'homePlmnId.mnc': mapping.homePlmnId.mnc
    });

    if (existingMapping) {
      throw new Error('S-NSSAI mapping already exists');
    }

    const mappingId = `mapping-${mapping.servingPlmnId.mcc}${mapping.servingPlmnId.mnc}-${mapping.homePlmnId.mcc}${mapping.homePlmnId.mnc}-${mapping.servingSnssai.sst}${mapping.servingSnssai.sd || ''}`;

    const newMapping: SnssaiMapping = {
      ...mapping,
      mappingId
    };

    await mappingCollection.insertOne(newMapping as any);
    return newMapping;
  } catch (error) {
    console.error('Error creating S-NSSAI mapping:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const getAllSnssaiMappings = async (): Promise<SnssaiMapping[]> => {
  try {
    const mappingCollection = getCollection<SnssaiMapping>('snssai-mappings');
    return await mappingCollection.find({}).toArray();
  } catch (error) {
    console.error('Error getting all S-NSSAI mappings:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const getSnssaiMappingById = async (mappingId: string): Promise<SnssaiMapping | null> => {
  try {
    const mappingCollection = getCollection<SnssaiMapping>('snssai-mappings');
    return await mappingCollection.findOne({ mappingId });
  } catch (error) {
    console.error('Error getting S-NSSAI mapping by ID:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const updateSnssaiMapping = async (
  mappingId: string,
  updates: Partial<Omit<SnssaiMapping, 'mappingId'>>
): Promise<SnssaiMapping | null> => {
  try {
    const mappingCollection = getCollection<SnssaiMapping>('snssai-mappings');

    const result = await mappingCollection.findOneAndUpdate(
      { mappingId },
      { $set: updates },
      { returnDocument: 'after' }
    );

    return result || null;
  } catch (error) {
    console.error('Error updating S-NSSAI mapping:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const deleteSnssaiMapping = async (mappingId: string): Promise<boolean> => {
  try {
    const mappingCollection = getCollection<SnssaiMapping>('snssai-mappings');

    const result = await mappingCollection.deleteOne({ mappingId });
    return result.deletedCount > 0;
  } catch (error) {
    console.error('Error deleting S-NSSAI mapping:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};
