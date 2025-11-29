import { SupportedFeatures } from '../types/common-types';

export enum NssfFeature {
  NSSRG = 0,
  NSAG = 1,
  ENHANCED_ROAMING = 2,
  SLICE_PRIORITY = 3,
  DYNAMIC_MAPPING = 4
}

type FeatureSupport = {
  [key in NssfFeature]?: boolean;
};

const NSSF_SUPPORTED_FEATURES: FeatureSupport = {
  [NssfFeature.ENHANCED_ROAMING]: true,
  [NssfFeature.SLICE_PRIORITY]: true,
  [NssfFeature.DYNAMIC_MAPPING]: true,
  [NssfFeature.NSSRG]: false,
  [NssfFeature.NSAG]: false
};

const hexCharToBits = (hexChar: string): boolean[] => {
  const value = parseInt(hexChar, 16);
  return [
    (value & 0x1) !== 0,
    (value & 0x2) !== 0,
    (value & 0x4) !== 0,
    (value & 0x8) !== 0
  ];
};

const bitsToHexChar = (bits: boolean[]): string => {
  let value = 0;
  if (bits[0]) value |= 0x1;
  if (bits[1]) value |= 0x2;
  if (bits[2]) value |= 0x4;
  if (bits[3]) value |= 0x8;
  return value.toString(16);
};

const parseFeatureString = (features: string): boolean[] => {
  const bits: boolean[] = [];

  for (let i = features.length - 1; i >= 0; i--) {
    const hexBits = hexCharToBits(features[i]);
    bits.push(...hexBits);
  }

  return bits;
};

const createFeatureString = (bits: boolean[]): string => {
  let result = '';

  for (let i = 0; i < bits.length; i += 4) {
    const chunk = [
      bits[i] || false,
      bits[i + 1] || false,
      bits[i + 2] || false,
      bits[i + 3] || false
    ];
    result = bitsToHexChar(chunk) + result;
  }

  return result || '0';
};

export const negotiateFeatures = (
  consumerFeatures: SupportedFeatures | undefined
): SupportedFeatures | undefined => {
  if (!consumerFeatures || consumerFeatures.length === 0) {
    return undefined;
  }

  if (!/^[0-9a-fA-F]+$/.test(consumerFeatures)) {
    return undefined;
  }

  const consumerBits = parseFeatureString(consumerFeatures.toLowerCase());

  const negotiatedBits: boolean[] = [];

  for (let i = 0; i < consumerBits.length; i++) {
    const featureSupported = NSSF_SUPPORTED_FEATURES[i as NssfFeature] ?? false;
    negotiatedBits[i] = consumerBits[i] && featureSupported;
  }

  while (negotiatedBits.length > 0 && !negotiatedBits[negotiatedBits.length - 1]) {
    negotiatedBits.pop();
  }

  if (negotiatedBits.length === 0) {
    return undefined;
  }

  return createFeatureString(negotiatedBits);
};

export const isFeatureSupported = (
  negotiatedFeatures: SupportedFeatures | undefined,
  feature: NssfFeature
): boolean => {
  if (!negotiatedFeatures) {
    return false;
  }

  const bits = parseFeatureString(negotiatedFeatures.toLowerCase());

  return bits[feature] === true;
};

export const getRequiredFeaturesForNsi = (): SupportedFeatures | undefined => {
  return undefined;
};

export const validateRequiredFeatures = (
  negotiatedFeatures: SupportedFeatures | undefined,
  requiredFeatures: SupportedFeatures | undefined
): boolean => {
  if (!requiredFeatures) {
    return true;
  }

  if (!negotiatedFeatures) {
    return false;
  }

  const negotiatedBits = parseFeatureString(negotiatedFeatures.toLowerCase());
  const requiredBits = parseFeatureString(requiredFeatures.toLowerCase());

  for (let i = 0; i < requiredBits.length; i++) {
    if (requiredBits[i] && !negotiatedBits[i]) {
      return false;
    }
  }

  return true;
};
