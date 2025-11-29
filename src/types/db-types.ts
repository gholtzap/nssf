import { Snssai, PlmnId, Tai, AccessType, Uri } from './common-types';
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
