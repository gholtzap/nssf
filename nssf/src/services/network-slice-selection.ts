import { getCollection } from '../db/mongodb';
import {
  AuthorizedNetworkSliceInfo,
  AllowedNssai,
  AllowedSnssai,
  ConfiguredSnssai,
  SliceInfoForRegistration
} from '../types/nnssf-nsselection-types';
import { Snssai, PlmnId, Tai, AccessType } from '../types/common-types';
import { SliceConfiguration, UeSubscription } from '../types/db-types';

type NetworkSliceSelectionInput = {
  sliceInfoForRegistration: SliceInfoForRegistration;
  homePlmnId?: PlmnId;
  tai?: Tai;
};

const snssaiMatches = (s1: Snssai, s2: Snssai): boolean => {
  return s1.sst === s2.sst && s1.sd === s2.sd;
};

const isSnssaiInList = (snssai: Snssai, list: Snssai[]): boolean => {
  return list.some(s => snssaiMatches(s, snssai));
};

const plmnMatches = (p1: PlmnId, p2: PlmnId): boolean => {
  return p1.mcc === p2.mcc && p1.mnc === p2.mnc;
};

const isSliceAvailableInTai = (slice: SliceConfiguration, tai?: Tai): boolean => {
  if (!tai || !slice.taiList || slice.taiList.length === 0) {
    return true;
  }

  return slice.taiList.some(sliceTai =>
    plmnMatches(sliceTai.plmnId, tai.plmnId) && sliceTai.tac === tai.tac
  );
};

export const selectNetworkSlicesForRegistration = async (
  input: NetworkSliceSelectionInput
): Promise<AuthorizedNetworkSliceInfo> => {
  const { sliceInfoForRegistration, homePlmnId, tai } = input;

  const slicesCollection = getCollection<SliceConfiguration>('slices');
  const subscriptionsCollection = getCollection<UeSubscription>('subscriptions');

  const allowedNssaiList: AllowedNssai[] = [];
  const configuredNssai: ConfiguredSnssai[] = [];
  const rejectedNssaiInPlmn: Snssai[] = [];
  const rejectedNssaiInTa: Snssai[] = [];

  const subscribedSnssais = sliceInfoForRegistration.subscribedNssai || [];
  const requestedNssais = sliceInfoForRegistration.requestedNssai || [];

  const availableSlices = await slicesCollection.find({}).toArray();

  const processedSnssais: Snssai[] = [];

  const snssaisToCheck = requestedNssais.length > 0 ? requestedNssais :
                         subscribedSnssais.map(s => s.subscribedSnssai);

  for (const snssai of snssaisToCheck) {
    const isSubscribed = subscribedSnssais.some(s => snssaiMatches(s.subscribedSnssai, snssai));

    if (!isSubscribed && requestedNssais.length > 0) {
      rejectedNssaiInPlmn.push(snssai);
      continue;
    }

    const availableSlice = availableSlices.find((slice: SliceConfiguration) =>
      snssaiMatches(slice.snssai, snssai)
    );

    if (!availableSlice) {
      rejectedNssaiInPlmn.push(snssai);
      continue;
    }

    if (homePlmnId && !plmnMatches(availableSlice.plmnId, homePlmnId)) {
      rejectedNssaiInPlmn.push(snssai);
      continue;
    }

    if (!isSliceAvailableInTai(availableSlice, tai)) {
      rejectedNssaiInTa.push(snssai);
      continue;
    }

    processedSnssais.push(snssai);
  }

  if (processedSnssais.length > 0) {
    const allowedSnssaiList: AllowedSnssai[] = processedSnssais.map(snssai => ({
      allowedSnssai: snssai
    }));

    allowedNssaiList.push({
      allowedSnssaiList,
      accessType: AccessType.THREE_GPP_ACCESS
    });
  }

  for (const subscribedSnssai of subscribedSnssais) {
    const slice = availableSlices.find((s: SliceConfiguration) =>
      snssaiMatches(s.snssai, subscribedSnssai.subscribedSnssai)
    );

    if (slice && isSliceAvailableInTai(slice, tai)) {
      configuredNssai.push({
        configuredSnssai: subscribedSnssai.subscribedSnssai
      });
    }
  }

  return {
    allowedNssaiList: allowedNssaiList.length > 0 ? allowedNssaiList : undefined,
    configuredNssai: configuredNssai.length > 0 ? configuredNssai : undefined,
    rejectedNssaiInPlmn: rejectedNssaiInPlmn.length > 0 ? rejectedNssaiInPlmn : undefined,
    rejectedNssaiInTa: rejectedNssaiInTa.length > 0 ? rejectedNssaiInTa : undefined
  };
};
