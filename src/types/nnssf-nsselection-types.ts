import { Snssai, NfInstanceId, Uri, SupportedFeatures, NfServiceSetId, NsSrg, NsagId, Tai, AccessType } from './common-types';

export type NsiId = string;

export enum RoamingIndication {
  NON_ROAMING = 'NON_ROAMING',
  LOCAL_BREAKOUT = 'LOCAL_BREAKOUT',
  HOME_ROUTED_ROAMING = 'HOME_ROUTED_ROAMING'
}

export type NsiInformation = {
  nrfId: Uri;
  nsiId?: NsiId;
  nrfNfMgtUri?: Uri;
  nrfAccessTokenUri?: Uri;
  nrfOauth2Required?: Record<string, boolean>;
};

export type AllowedSnssai = {
  allowedSnssai: Snssai;
  nsiInformationList?: NsiInformation[];
  mappedHomeSnssai?: Snssai;
};

export type AllowedNssai = {
  allowedSnssaiList: AllowedSnssai[];
  accessType: AccessType;
};

export type ConfiguredSnssai = {
  configuredSnssai: Snssai;
  mappedHomeSnssai?: Snssai;
};

export type NsagInfo = {
  nsagIds: NsagId[];
  snssaiList: Snssai[];
  taiList?: Tai[];
  taiRangeList?: TaiRange[];
};

export type TaiRange = {
  plmnId: {
    mcc: string;
    mnc: string;
  };
  tacRangeList: {
    start: string;
    end?: string;
  }[];
};

export type AuthorizedNetworkSliceInfo = {
  allowedNssaiList?: AllowedNssai[];
  configuredNssai?: ConfiguredSnssai[];
  targetAmfSet?: string;
  candidateAmfList?: NfInstanceId[];
  rejectedNssaiInPlmn?: Snssai[];
  rejectedNssaiInTa?: Snssai[];
  nsiInformation?: NsiInformation;
  supportedFeatures?: SupportedFeatures;
  nrfAmfSet?: Uri;
  nrfAmfSetNfMgtUri?: Uri;
  nrfAmfSetAccessTokenUri?: Uri;
  nrfOauth2Required?: Record<string, boolean>;
  targetAmfServiceSet?: NfServiceSetId;
  targetNssai?: Snssai[];
  nsagInfos?: NsagInfo[];
};

export type SubscribedSnssai = {
  subscribedSnssai: Snssai;
  defaultIndication?: boolean;
  subscribedNsSrgList?: NsSrg[];
};

export type MappingOfSnssai = {
  servingSnssai: Snssai;
  homeSnssai: Snssai;
};

export type SliceInfoForRegistration = {
  subscribedNssai?: SubscribedSnssai[];
  allowedNssaiCurrentAccess?: AllowedNssai;
  allowedNssaiOtherAccess?: AllowedNssai;
  sNssaiForMapping?: Snssai[];
  requestedNssai?: Snssai[];
  defaultConfiguredSnssaiInd?: boolean;
  mappingOfNssai?: MappingOfSnssai[];
  requestMapping?: boolean;
  ueSupNssrgInd?: boolean;
  suppressNssrgInd?: boolean;
  nsagSupported?: boolean;
};

export type SliceInfoForPDUSession = {
  sNssai: Snssai;
  roamingIndication: RoamingIndication;
  homeSnssai?: Snssai;
};

export type SliceInfoForUEConfigurationUpdate = {
  subscribedNssai?: SubscribedSnssai[];
  allowedNssaiCurrentAccess?: AllowedNssai;
  allowedNssaiOtherAccess?: AllowedNssai;
  defaultConfiguredSnssaiInd?: boolean;
  requestedNssai?: Snssai[];
  mappingOfNssai?: MappingOfSnssai[];
  ueSupNssrgInd?: boolean;
  suppressNssrgInd?: boolean;
  rejectedNssaiRa?: Snssai[];
  nsagSupported?: boolean;
};
