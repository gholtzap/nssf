import { getCollection, handleDatabaseError } from '../db/mongodb';
import { SlicePolicy } from '../types/db-types';
import { Snssai, PlmnId } from '../types/common-types';

export const getAllPolicies = async (): Promise<SlicePolicy[]> => {
  try {
    const policyCollection = getCollection<SlicePolicy>('policies');
    return await policyCollection.find({}).toArray();
  } catch (error) {
    console.error('Error getting all policies:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const getPoliciesBySnssai = async (snssai: Snssai, plmnId: PlmnId): Promise<SlicePolicy[]> => {
  try {
    const policyCollection = getCollection<SlicePolicy>('policies');
    return await policyCollection.find({
      'snssai.sst': snssai.sst,
      'snssai.sd': snssai.sd,
      'plmnId.mcc': plmnId.mcc,
      'plmnId.mnc': plmnId.mnc
    }).toArray();
  } catch (error) {
    console.error('Error getting policies by S-NSSAI:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const getPolicyById = async (policyId: string): Promise<SlicePolicy | null> => {
  try {
    const policyCollection = getCollection<SlicePolicy>('policies');
    return await policyCollection.findOne({ policyId });
  } catch (error) {
    console.error('Error getting policy by ID:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const createPolicy = async (policy: SlicePolicy): Promise<void> => {
  try {
    const policyCollection = getCollection<SlicePolicy>('policies');

    const existing = await policyCollection.findOne({ policyId: policy.policyId });
    if (existing) {
      throw new Error(`Policy with ID ${policy.policyId} already exists`);
    }

    await policyCollection.insertOne(policy);
  } catch (error) {
    console.error('Error creating policy:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const updatePolicy = async (policyId: string, policy: Partial<SlicePolicy>): Promise<void> => {
  try {
    const policyCollection = getCollection<SlicePolicy>('policies');

    const existing = await policyCollection.findOne({ policyId });
    if (!existing) {
      throw new Error(`Policy with ID ${policyId} not found`);
    }

    await policyCollection.updateOne(
      { policyId },
      { $set: policy }
    );
  } catch (error) {
    console.error('Error updating policy:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};

export const deletePolicy = async (policyId: string): Promise<void> => {
  try {
    const policyCollection = getCollection<SlicePolicy>('policies');

    const result = await policyCollection.deleteOne({ policyId });
    if (result.deletedCount === 0) {
      throw new Error(`Policy with ID ${policyId} not found`);
    }
  } catch (error) {
    console.error('Error deleting policy:', error);
    const dbError = handleDatabaseError(error);
    throw dbError;
  }
};
