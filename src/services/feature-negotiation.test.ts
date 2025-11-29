import { expect } from 'chai';
import { negotiateFeatures, isFeatureSupported, validateRequiredFeatures, NssfFeature } from './feature-negotiation';

describe('Feature Negotiation', () => {
  describe('negotiateFeatures', () => {
    it('should return undefined when consumerFeatures is undefined', () => {
      const result = negotiateFeatures(undefined);
      expect(result).to.be.undefined;
    });

    it('should return undefined when consumerFeatures is empty string', () => {
      const result = negotiateFeatures('');
      expect(result).to.be.undefined;
    });

    it('should return undefined for invalid hex string', () => {
      const result = negotiateFeatures('xyz');
      expect(result).to.be.undefined;
    });

    it('should negotiate features correctly when consumer supports all NSSF features', () => {
      const result = negotiateFeatures('1f');
      expect(result).to.equal('1c');
    });

    it('should negotiate features correctly when consumer supports subset', () => {
      const result = negotiateFeatures('7');
      expect(result).to.equal('4');
    });

    it('should negotiate features with enhanced roaming (feature 2)', () => {
      const result = negotiateFeatures('4');
      expect(result).to.equal('4');
    });

    it('should negotiate features with slice priority (feature 3)', () => {
      const result = negotiateFeatures('8');
      expect(result).to.equal('8');
    });

    it('should negotiate features with dynamic mapping (feature 4)', () => {
      const result = negotiateFeatures('10');
      expect(result).to.equal('10');
    });

    it('should reject NSSRG feature (feature 0)', () => {
      const result = negotiateFeatures('1');
      expect(result).to.be.undefined;
    });

    it('should reject NSAG feature (feature 1)', () => {
      const result = negotiateFeatures('2');
      expect(result).to.be.undefined;
    });

    it('should negotiate multiple supported features', () => {
      const result = negotiateFeatures('1c');
      expect(result).to.equal('1c');
    });

    it('should handle uppercase hex characters', () => {
      const result = negotiateFeatures('1C');
      expect(result).to.equal('1c');
    });

    it('should handle longer feature strings', () => {
      const result = negotiateFeatures('00001c');
      expect(result).to.equal('1c');
    });

    it('should return undefined when no features are negotiated', () => {
      const result = negotiateFeatures('3');
      expect(result).to.be.undefined;
    });
  });

  describe('isFeatureSupported', () => {
    it('should return false when negotiatedFeatures is undefined', () => {
      const result = isFeatureSupported(undefined, NssfFeature.ENHANCED_ROAMING);
      expect(result).to.be.false;
    });

    it('should return true for supported ENHANCED_ROAMING feature', () => {
      const result = isFeatureSupported('4', NssfFeature.ENHANCED_ROAMING);
      expect(result).to.be.true;
    });

    it('should return false for unsupported NSSRG feature', () => {
      const result = isFeatureSupported('4', NssfFeature.NSSRG);
      expect(result).to.be.false;
    });

    it('should return true for supported SLICE_PRIORITY feature', () => {
      const result = isFeatureSupported('8', NssfFeature.SLICE_PRIORITY);
      expect(result).to.be.true;
    });

    it('should return true for supported DYNAMIC_MAPPING feature', () => {
      const result = isFeatureSupported('10', NssfFeature.DYNAMIC_MAPPING);
      expect(result).to.be.true;
    });

    it('should return false when feature is not in negotiated string', () => {
      const result = isFeatureSupported('4', NssfFeature.SLICE_PRIORITY);
      expect(result).to.be.false;
    });

    it('should handle multiple features', () => {
      expect(isFeatureSupported('1c', NssfFeature.ENHANCED_ROAMING)).to.be.true;
      expect(isFeatureSupported('1c', NssfFeature.SLICE_PRIORITY)).to.be.true;
      expect(isFeatureSupported('1c', NssfFeature.DYNAMIC_MAPPING)).to.be.true;
      expect(isFeatureSupported('1c', NssfFeature.NSSRG)).to.be.false;
      expect(isFeatureSupported('1c', NssfFeature.NSAG)).to.be.false;
    });
  });

  describe('validateRequiredFeatures', () => {
    it('should return true when requiredFeatures is undefined', () => {
      const result = validateRequiredFeatures('4', undefined);
      expect(result).to.be.true;
    });

    it('should return false when negotiatedFeatures is undefined but required features exist', () => {
      const result = validateRequiredFeatures(undefined, '4');
      expect(result).to.be.false;
    });

    it('should return true when all required features are negotiated', () => {
      const result = validateRequiredFeatures('1c', '4');
      expect(result).to.be.true;
    });

    it('should return false when required feature is missing', () => {
      const result = validateRequiredFeatures('4', '8');
      expect(result).to.be.false;
    });

    it('should return true when negotiated features exceed required features', () => {
      const result = validateRequiredFeatures('1c', '8');
      expect(result).to.be.true;
    });

    it('should handle multiple required features', () => {
      const result = validateRequiredFeatures('1c', 'c');
      expect(result).to.be.true;
    });

    it('should return false when any required feature is missing', () => {
      const result = validateRequiredFeatures('4', 'c');
      expect(result).to.be.false;
    });

    it('should return true when both are undefined', () => {
      const result = validateRequiredFeatures(undefined, undefined);
      expect(result).to.be.true;
    });
  });

  describe('Integration tests', () => {
    it('should negotiate and validate features correctly', () => {
      const consumerFeatures = '1f';
      const negotiated = negotiateFeatures(consumerFeatures);

      expect(negotiated).to.equal('1c');
      expect(isFeatureSupported(negotiated, NssfFeature.ENHANCED_ROAMING)).to.be.true;
      expect(isFeatureSupported(negotiated, NssfFeature.SLICE_PRIORITY)).to.be.true;
      expect(isFeatureSupported(negotiated, NssfFeature.DYNAMIC_MAPPING)).to.be.true;
    });

    it('should handle case where consumer has no supported features', () => {
      const consumerFeatures = '3';
      const negotiated = negotiateFeatures(consumerFeatures);

      expect(negotiated).to.be.undefined;
      expect(isFeatureSupported(negotiated, NssfFeature.ENHANCED_ROAMING)).to.be.false;
    });

    it('should validate required features against negotiated features', () => {
      const consumerFeatures = '1f';
      const negotiated = negotiateFeatures(consumerFeatures);
      const requiredFeatures = '4';

      expect(validateRequiredFeatures(negotiated, requiredFeatures)).to.be.true;
    });
  });
});
