import { Snssai, Tai, PlmnId, Uri, NfInstanceId, SupportedFeatures } from './common-types';

export type NssaiAvailabilitySubscription = {
  subscriptionId: string;
  nfInstanceId: NfInstanceId;
  subscriptionData: NssaiAvailabilitySubscriptionData;
  notificationUri: Uri;
  supportedFeatures?: SupportedFeatures;
  expiryTime?: string;
  createdAt: Date;
  updatedAt: Date;
};

export type NssaiAvailabilitySubscriptionData = {
  tai: Tai;
  supportedSnssaiList?: Snssai[];
};

export type NssaiAvailabilitySubscriptionCreateRequest = {
  nfInstanceId: NfInstanceId;
  subscriptionData: NssaiAvailabilitySubscriptionData;
  notificationUri: Uri;
  supportedFeatures?: SupportedFeatures;
  expiryTime?: string;
};

export type NssaiAvailabilitySubscriptionUpdateRequest = {
  subscriptionData: NssaiAvailabilitySubscriptionData;
  supportedFeatures?: SupportedFeatures;
  expiryTime?: string;
};

export type NssaiAvailabilityNotification = {
  subscriptionId: string;
  authorizedNssaiAvailabilityData: AuthorizedNssaiAvailabilityData[];
};

export type AuthorizedNssaiAvailabilityData = {
  tai: Tai;
  supportedSnssaiList: Snssai[];
  restrictedSnssaiList?: RestrictedSnssai[];
};

export type RestrictedSnssai = {
  snssai: Snssai;
  restrictionType?: RestrictionType;
};

export enum RestrictionType {
  NOT_ALLOWED = 'NOT_ALLOWED',
  RESTRICTED_IN_TAI = 'RESTRICTED_IN_TAI'
}
