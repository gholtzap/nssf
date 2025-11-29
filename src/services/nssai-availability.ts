import { getCollection } from '../db/mongodb';
import {
  NssaiAvailabilitySubscription,
  NssaiAvailabilitySubscriptionCreateRequest,
  NssaiAvailabilitySubscriptionUpdateRequest,
  NssaiAvailabilityNotification,
  AuthorizedNssaiAvailabilityData,
  RestrictedSnssai,
  RestrictionType
} from '../types/nnssf-nssaiavailability-types';
import { SliceConfiguration } from '../types/db-types';
import { Snssai, Tai, PlmnId } from '../types/common-types';
import { createId } from '@paralleldrive/cuid2';
import axios from 'axios';

const snssaiMatches = (s1: Snssai, s2: Snssai): boolean => {
  return s1.sst === s2.sst && s1.sd === s2.sd;
};

const plmnMatches = (p1: PlmnId, p2: PlmnId): boolean => {
  return p1.mcc === p2.mcc && p1.mnc === p2.mnc;
};

const taiMatches = (t1: Tai, t2: Tai): boolean => {
  return plmnMatches(t1.plmnId, t2.plmnId) && t1.tac === t2.tac;
};

export const createNssaiAvailabilitySubscription = async (
  request: NssaiAvailabilitySubscriptionCreateRequest
): Promise<NssaiAvailabilitySubscription> => {
  const subscriptionsCollection = getCollection<NssaiAvailabilitySubscription>('nssai_availability_subscriptions');

  const subscriptionId = createId();
  const now = new Date();

  const subscription: NssaiAvailabilitySubscription = {
    subscriptionId,
    nfInstanceId: request.nfInstanceId,
    subscriptionData: request.subscriptionData,
    notificationUri: request.notificationUri,
    supportedFeatures: request.supportedFeatures,
    expiryTime: request.expiryTime,
    createdAt: now,
    updatedAt: now
  };

  await subscriptionsCollection.insertOne(subscription);

  return subscription;
};

export const getNssaiAvailabilitySubscription = async (
  subscriptionId: string
): Promise<NssaiAvailabilitySubscription | null> => {
  const subscriptionsCollection = getCollection<NssaiAvailabilitySubscription>('nssai_availability_subscriptions');

  const subscription = await subscriptionsCollection.findOne({ subscriptionId });

  return subscription;
};

export const updateNssaiAvailabilitySubscription = async (
  subscriptionId: string,
  update: NssaiAvailabilitySubscriptionUpdateRequest
): Promise<NssaiAvailabilitySubscription | null> => {
  const subscriptionsCollection = getCollection<NssaiAvailabilitySubscription>('nssai_availability_subscriptions');

  const updateDoc = {
    $set: {
      subscriptionData: update.subscriptionData,
      supportedFeatures: update.supportedFeatures,
      expiryTime: update.expiryTime,
      updatedAt: new Date()
    }
  };

  const result = await subscriptionsCollection.findOneAndUpdate(
    { subscriptionId },
    updateDoc,
    { returnDocument: 'after' }
  );

  return result || null;
};

export const deleteNssaiAvailabilitySubscription = async (
  subscriptionId: string
): Promise<boolean> => {
  const subscriptionsCollection = getCollection<NssaiAvailabilitySubscription>('nssai_availability_subscriptions');

  const result = await subscriptionsCollection.deleteOne({ subscriptionId });

  return result.deletedCount > 0;
};

export const getAuthorizedNssaiAvailabilityData = async (
  tai: Tai,
  supportedSnssaiList?: Snssai[]
): Promise<AuthorizedNssaiAvailabilityData> => {
  const slicesCollection = getCollection<SliceConfiguration>('slices');

  const slices = await slicesCollection.find({
    'plmnId.mcc': tai.plmnId.mcc,
    'plmnId.mnc': tai.plmnId.mnc
  }).toArray();

  const availableSnssais: Snssai[] = [];
  const restrictedSnssais: RestrictedSnssai[] = [];

  const snssaisToCheck = supportedSnssaiList || slices.map(s => s.snssai);

  for (const snssai of snssaisToCheck) {
    const slice = slices.find(s => snssaiMatches(s.snssai, snssai));

    if (!slice) {
      restrictedSnssais.push({
        snssai,
        restrictionType: RestrictionType.NOT_ALLOWED
      });
      continue;
    }

    if (slice.taiList && slice.taiList.length > 0) {
      const isAvailableInTai = slice.taiList.some(sliceTai => taiMatches(sliceTai, tai));

      if (!isAvailableInTai) {
        restrictedSnssais.push({
          snssai,
          restrictionType: RestrictionType.RESTRICTED_IN_TAI
        });
        continue;
      }
    }

    availableSnssais.push(snssai);
  }

  return {
    tai,
    supportedSnssaiList: availableSnssais,
    restrictedSnssaiList: restrictedSnssais.length > 0 ? restrictedSnssais : undefined
  };
};

export const notifyNssaiAvailabilityChange = async (
  tai: Tai,
  supportedSnssaiList?: Snssai[]
): Promise<void> => {
  const subscriptionsCollection = getCollection<NssaiAvailabilitySubscription>('nssai_availability_subscriptions');

  const subscriptions = await subscriptionsCollection.find({
    'subscriptionData.tai.plmnId.mcc': tai.plmnId.mcc,
    'subscriptionData.tai.plmnId.mnc': tai.plmnId.mnc,
    'subscriptionData.tai.tac': tai.tac
  }).toArray();

  const authorizedData = await getAuthorizedNssaiAvailabilityData(tai, supportedSnssaiList);

  for (const subscription of subscriptions) {
    const notification: NssaiAvailabilityNotification = {
      subscriptionId: subscription.subscriptionId,
      authorizedNssaiAvailabilityData: [authorizedData]
    };

    try {
      await axios.post(subscription.notificationUri, notification, {
        headers: {
          'Content-Type': 'application/json'
        },
        timeout: 5000
      });
    } catch (error) {
      console.error(`Failed to send NSSAI availability notification to ${subscription.notificationUri}:`, error);
    }
  }
};
