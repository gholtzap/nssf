import { getCollection } from '../db/mongodb';
import { UeSubscription } from '../types/db-types';
import { PlmnId } from '../types/common-types';

export const getSubscriptionBySupi = async (
  supi: string,
  plmnId: PlmnId
): Promise<UeSubscription | null> => {
  const subscriptionsCollection = getCollection<UeSubscription>('subscriptions');

  const subscription = await subscriptionsCollection.findOne({
    supi,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  });

  return subscription;
};

export const createSubscription = async (
  subscription: UeSubscription
): Promise<void> => {
  const subscriptionsCollection = getCollection<UeSubscription>('subscriptions');

  await subscriptionsCollection.insertOne(subscription);
};

export const updateSubscription = async (
  supi: string,
  plmnId: PlmnId,
  subscription: Partial<UeSubscription>
): Promise<void> => {
  const subscriptionsCollection = getCollection<UeSubscription>('subscriptions');

  await subscriptionsCollection.updateOne(
    {
      supi,
      'plmnId.mcc': plmnId.mcc,
      'plmnId.mnc': plmnId.mnc
    },
    { $set: subscription }
  );
};

export const deleteSubscription = async (
  supi: string,
  plmnId: PlmnId
): Promise<void> => {
  const subscriptionsCollection = getCollection<UeSubscription>('subscriptions');

  await subscriptionsCollection.deleteOne({
    supi,
    'plmnId.mcc': plmnId.mcc,
    'plmnId.mnc': plmnId.mnc
  });
};
