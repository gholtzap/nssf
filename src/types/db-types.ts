import { Snssai, PlmnId, Tai, AccessType } from './common-types';
import { SubscribedSnssai } from './nnssf-nsselection-types';

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
