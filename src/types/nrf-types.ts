import { PlmnId, Snssai, Tai, Uri, NfInstanceId } from './common-types';

export enum NfType {
  NRF = 'NRF',
  UDM = 'UDM',
  AMF = 'AMF',
  SMF = 'SMF',
  AUSF = 'AUSF',
  NEF = 'NEF',
  PCF = 'PCF',
  SMSF = 'SMSF',
  NSSF = 'NSSF',
  UDR = 'UDR',
  LMF = 'LMF',
  GMLC = 'GMLC',
  FIVE_G_EIR = '5G_EIR',
  SEPP = 'SEPP',
  UPF = 'UPF',
  N3IWF = 'N3IWF',
  AF = 'AF',
  UDSF = 'UDSF',
  BSF = 'BSF',
  CHF = 'CHF',
  NWDAF = 'NWDAF'
}

export enum NfStatus {
  REGISTERED = 'REGISTERED',
  SUSPENDED = 'SUSPENDED',
  UNDISCOVERABLE = 'UNDISCOVERABLE'
}

export type NFProfile = {
  nfInstanceId: NfInstanceId;
  nfType: NfType;
  nfStatus: NfStatus;
  plmnList?: PlmnId[];
  sNssais?: Snssai[];
  nsiList?: string[];
  fqdn?: string;
  ipv4Addresses?: string[];
  ipv6Addresses?: string[];
  priority?: number;
  capacity?: number;
  load?: number;
  locality?: string;
  nfServices?: NFService[];
  amfInfo?: AmfInfo;
};

export type NFService = {
  serviceInstanceId: string;
  serviceName: string;
  versions: NFServiceVersion[];
  scheme: 'http' | 'https';
  nfServiceStatus: NfStatus;
  fqdn?: string;
  ipEndPoints?: IpEndPoint[];
  apiPrefix?: string;
  allowedPlmns?: PlmnId[];
  allowedNssais?: Snssai[];
  priority?: number;
  capacity?: number;
  load?: number;
};

export type NFServiceVersion = {
  apiVersionInUri: string;
  apiFullVersion: string;
};

export type IpEndPoint = {
  ipv4Address?: string;
  ipv6Address?: string;
  transport?: 'TCP' | 'UDP' | 'SCTP';
  port?: number;
};

export type AmfInfo = {
  amfSetId: string;
  amfRegionId: string;
  guamiList?: Guami[];
  taiList?: Tai[];
  n2InterfaceAmfInfo?: N2InterfaceAmfInfo;
};

export type Guami = {
  plmnId: PlmnId;
  amfId: string;
};

export type N2InterfaceAmfInfo = {
  ipv4EndpointAddress?: string[];
  ipv6EndpointAddress?: string[];
  amfName?: string;
};

export type SearchResult = {
  nfInstances: NFProfile[];
  validityPeriod?: number;
  nrfSupportedFeatures?: string;
};

export type AccessTokenRequest = {
  grant_type: 'client_credentials';
  nfInstanceId: NfInstanceId;
  scope: string;
  targetNfType?: NfType;
  targetNfInstanceId?: NfInstanceId;
};

export type AccessTokenResponse = {
  access_token: string;
  token_type: string;
  expires_in: number;
  scope?: string;
};

export type AccessTokenCacheEntry = {
  token: string;
  expiresAt: number;
  scope: string;
};
