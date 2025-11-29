import { NfInstanceId, Uri, Snssai, Guami, PlmnId } from './common-types';

export type AmfSetId = string;

export type AmfServiceSetId = string;

export type AmfCandidate = {
  nfInstanceId: NfInstanceId;
  amfSetId?: AmfSetId;
  amfServiceSetId?: AmfServiceSetId;
  guami?: Guami;
};

export type AmfSetConfig = {
  amfSetId: AmfSetId;
  plmnId: PlmnId;
  supportedSnssais: Snssai[];
  nrfId: Uri;
  nrfNfMgtUri?: Uri;
  nrfAccessTokenUri?: Uri;
  nrfOauth2Required?: Record<string, boolean>;
  priority?: number;
  capacity?: number;
};

export type AmfServiceSetConfig = {
  amfServiceSetId: AmfServiceSetId;
  amfSetId: AmfSetId;
  plmnId: PlmnId;
  supportedSnssais: Snssai[];
  nrfId: Uri;
  priority?: number;
};

export type AmfInstanceConfig = {
  nfInstanceId: NfInstanceId;
  amfSetId: AmfSetId;
  amfServiceSetId?: AmfServiceSetId;
  plmnId: PlmnId;
  supportedSnssais: Snssai[];
  guami?: Guami;
  capacity?: number;
  loadLevel?: number;
};
