import { Snssai, PlmnId, Tai, AccessType, Uri, NsagId, NsSrg } from './common-types';
import { SubscribedSnssai, NsiId } from './nnssf-nsselection-types';

export type SliceConfiguration = {
  snssai: Snssai;
  plmnId: PlmnId;
  accessType: AccessType;
  taiList?: Tai[];
  isDefault?: boolean;
  priority?: number;
  maxUeSupport?: number;
};

export type UeSubscription = {
  supi: string;
  plmnId: PlmnId;
  subscribedSnssais: SubscribedSnssai[];
  defaultSnssai?: Snssai;
};

export type NsiConfiguration = {
  nsiId: NsiId;
  snssai: Snssai;
  plmnId: PlmnId;
  nrfId: Uri;
  nrfNfMgtUri?: Uri;
  nrfAccessTokenUri?: Uri;
  nrfOauth2Required?: Record<string, boolean>;
  taiList?: Tai[];
  priority?: number;
  loadLevel?: number;
};

export type TimeWindow = {
  startTime: string;
  endTime: string;
  daysOfWeek?: number[];
};

export type SlicePolicy = {
  policyId: string;
  snssai: Snssai;
  plmnId: PlmnId;
  maxUesPerSlice?: number;
  maxSessionsPerUe?: number;
  priorityLevel?: number;
  allowedTimeWindows?: TimeWindow[];
  deniedTimeWindows?: TimeWindow[];
  minPriorityLevel?: number;
  maxLoadLevel?: number;
  requiredSubscriptionTier?: string;
  allowedTaiList?: Tai[];
  deniedTaiList?: Tai[];
  enabled: boolean;
};

export type SnssaiMapping = {
  mappingId: string;
  servingPlmnId: PlmnId;
  homePlmnId: PlmnId;
  servingSnssai: Snssai;
  homeSnssai: Snssai;
  validityArea?: Tai[];
};

export type TaiRange = {
  start: string;
  end: string;
  plmnId: PlmnId;
};

export type NsagConfiguration = {
  nsagId: NsagId;
  snssaiList: Snssai[];
  plmnId: PlmnId;
  taiList?: Tai[];
  taiRangeList?: TaiRange[];
  maxUeCount?: number;
  currentUeCount?: number;
  priority?: number;
  enabled: boolean;
};

export type NssrgConfiguration = {
  nssrgId: NsSrg;
  snssaiList: Snssai[];
  plmnId: PlmnId;
  taiList?: Tai[];
  taiRangeList?: TaiRange[];
  maxUeCount?: number;
  currentUeCount?: number;
  priority?: number;
  enabled: boolean;
};
