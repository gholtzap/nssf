import { expect } from 'chai';
import {
  validateRequestedNssai,
  processRequestedNssai,
  filterRequestedNssaiBySubscription,
  prioritizeRequestedNssai
} from './requested-nssai-validation';
import { Snssai } from '../types/common-types';

describe('Requested NSSAI Validation Service', () => {
  describe('validateRequestedNssai', () => {
    it('should return valid result for undefined requested NSSAI', () => {
      const result = validateRequestedNssai(undefined);
      expect(result.isValid).to.be.true;
      expect(result.invalidParams).to.be.empty;
      expect(result.validatedNssai).to.be.empty;
    });

    it('should return valid result for empty array', () => {
      const result = validateRequestedNssai([]);
      expect(result.isValid).to.be.true;
      expect(result.invalidParams).to.be.empty;
      expect(result.validatedNssai).to.be.empty;
    });

    it('should validate a single valid S-NSSAI', () => {
      const requestedNssai: Snssai[] = [{ sst: 1, sd: 'ABCDEF' }];
      const result = validateRequestedNssai(requestedNssai);
      expect(result.isValid).to.be.true;
      expect(result.invalidParams).to.be.empty;
      expect(result.validatedNssai).to.deep.equal(requestedNssai);
    });

    it('should validate multiple valid S-NSSAI entries', () => {
      const requestedNssai: Snssai[] = [
        { sst: 1, sd: 'ABCDEF' },
        { sst: 2, sd: '123456' },
        { sst: 3 }
      ];
      const result = validateRequestedNssai(requestedNssai);
      expect(result.isValid).to.be.true;
      expect(result.invalidParams).to.be.empty;
      expect(result.validatedNssai).to.deep.equal(requestedNssai);
    });

    it('should reject if requested NSSAI is not an array', () => {
      const result = validateRequestedNssai('invalid' as any);
      expect(result.isValid).to.be.false;
      expect(result.invalidParams).to.have.lengthOf(1);
      expect(result.invalidParams[0].param).to.equal('requestedNssai');
      expect(result.invalidParams[0].reason).to.equal('must be an array');
    });

    it('should reject if requested NSSAI exceeds maximum of 8 entries', () => {
      const requestedNssai: Snssai[] = [
        { sst: 1 },
        { sst: 2 },
        { sst: 3 },
        { sst: 4 },
        { sst: 5 },
        { sst: 6 },
        { sst: 7 },
        { sst: 8 },
        { sst: 9 }
      ];
      const result = validateRequestedNssai(requestedNssai);
      expect(result.isValid).to.be.false;
      expect(result.invalidParams).to.have.lengthOf(1);
      expect(result.invalidParams[0].reason).to.include('cannot exceed 8');
    });

    it('should reject if S-NSSAI has invalid SST', () => {
      const requestedNssai: Snssai[] = [{ sst: 256, sd: 'ABCDEF' }];
      const result = validateRequestedNssai(requestedNssai);
      expect(result.isValid).to.be.false;
      expect(result.invalidParams).to.have.lengthOf(1);
      expect(result.invalidParams[0].param).to.include('sst');
    });

    it('should reject if S-NSSAI has invalid SD format', () => {
      const requestedNssai: Snssai[] = [{ sst: 1, sd: 'INVALID' }];
      const result = validateRequestedNssai(requestedNssai);
      expect(result.isValid).to.be.false;
      expect(result.invalidParams).to.have.lengthOf(1);
      expect(result.invalidParams[0].param).to.include('sd');
    });

    it('should reject if requested NSSAI contains duplicates', () => {
      const requestedNssai: Snssai[] = [
        { sst: 1, sd: 'ABCDEF' },
        { sst: 2, sd: '123456' },
        { sst: 1, sd: 'ABCDEF' }
      ];
      const result = validateRequestedNssai(requestedNssai);
      expect(result.isValid).to.be.false;
      expect(result.invalidParams).to.have.lengthOf(1);
      expect(result.invalidParams[0].reason).to.include('duplicate');
    });

    it('should use custom param prefix in error messages', () => {
      const requestedNssai: Snssai[] = [{ sst: 256 }];
      const result = validateRequestedNssai(requestedNssai, 'customParam');
      expect(result.isValid).to.be.false;
      expect(result.invalidParams[0].param).to.include('customParam');
    });
  });

  describe('processRequestedNssai', () => {
    it('should return empty array for undefined input', () => {
      const result = processRequestedNssai(undefined);
      expect(result).to.be.empty;
    });

    it('should return empty array for non-array input', () => {
      const result = processRequestedNssai('invalid' as any);
      expect(result).to.be.empty;
    });

    it('should remove duplicates from requested NSSAI', () => {
      const requestedNssai: Snssai[] = [
        { sst: 1, sd: 'ABCDEF' },
        { sst: 2, sd: '123456' },
        { sst: 1, sd: 'ABCDEF' }
      ];
      const result = processRequestedNssai(requestedNssai);
      expect(result).to.have.lengthOf(2);
      expect(result).to.deep.include({ sst: 1, sd: 'ABCDEF' });
      expect(result).to.deep.include({ sst: 2, sd: '123456' });
    });

    it('should limit to maximum 8 S-NSSAI entries', () => {
      const requestedNssai: Snssai[] = [
        { sst: 1 },
        { sst: 2 },
        { sst: 3 },
        { sst: 4 },
        { sst: 5 },
        { sst: 6 },
        { sst: 7 },
        { sst: 8 },
        { sst: 9 }
      ];
      const result = processRequestedNssai(requestedNssai);
      expect(result).to.have.lengthOf(8);
    });
  });

  describe('filterRequestedNssaiBySubscription', () => {
    it('should filter requested NSSAI by subscription', () => {
      const requestedNssai: Snssai[] = [
        { sst: 1, sd: 'ABCDEF' },
        { sst: 2, sd: '123456' },
        { sst: 3 }
      ];
      const subscribedNssai: Snssai[] = [
        { sst: 1, sd: 'ABCDEF' },
        { sst: 3 }
      ];

      const result = filterRequestedNssaiBySubscription(requestedNssai, subscribedNssai);
      expect(result.subscribed).to.have.lengthOf(2);
      expect(result.notSubscribed).to.have.lengthOf(1);
      expect(result.subscribed).to.deep.include({ sst: 1, sd: 'ABCDEF' });
      expect(result.subscribed).to.deep.include({ sst: 3 });
      expect(result.notSubscribed).to.deep.include({ sst: 2, sd: '123456' });
    });

    it('should return empty subscribed list when none match', () => {
      const requestedNssai: Snssai[] = [{ sst: 1 }];
      const subscribedNssai: Snssai[] = [{ sst: 2 }];

      const result = filterRequestedNssaiBySubscription(requestedNssai, subscribedNssai);
      expect(result.subscribed).to.be.empty;
      expect(result.notSubscribed).to.have.lengthOf(1);
    });

    it('should return empty notSubscribed list when all match', () => {
      const requestedNssai: Snssai[] = [{ sst: 1 }, { sst: 2 }];
      const subscribedNssai: Snssai[] = [{ sst: 1 }, { sst: 2 }];

      const result = filterRequestedNssaiBySubscription(requestedNssai, subscribedNssai);
      expect(result.subscribed).to.have.lengthOf(2);
      expect(result.notSubscribed).to.be.empty;
    });
  });

  describe('prioritizeRequestedNssai', () => {
    it('should prioritize default S-NSSAI entries first', () => {
      const requestedNssai: Snssai[] = [
        { sst: 1 },
        { sst: 2 },
        { sst: 3 }
      ];
      const subscribedNssai = [
        { subscribedSnssai: { sst: 1 }, defaultIndication: false },
        { subscribedSnssai: { sst: 2 }, defaultIndication: true },
        { subscribedSnssai: { sst: 3 }, defaultIndication: false }
      ];

      const result = prioritizeRequestedNssai(requestedNssai, subscribedNssai);
      expect(result).to.have.lengthOf(3);
      expect(result[0]).to.deep.equal({ sst: 2 });
      expect(result[1]).to.deep.equal({ sst: 1 });
      expect(result[2]).to.deep.equal({ sst: 3 });
    });

    it('should handle multiple default S-NSSAI entries', () => {
      const requestedNssai: Snssai[] = [
        { sst: 1 },
        { sst: 2 },
        { sst: 3 }
      ];
      const subscribedNssai = [
        { subscribedSnssai: { sst: 1 }, defaultIndication: true },
        { subscribedSnssai: { sst: 2 }, defaultIndication: true },
        { subscribedSnssai: { sst: 3 }, defaultIndication: false }
      ];

      const result = prioritizeRequestedNssai(requestedNssai, subscribedNssai);
      expect(result[0]).to.deep.equal({ sst: 1 });
      expect(result[1]).to.deep.equal({ sst: 2 });
      expect(result[2]).to.deep.equal({ sst: 3 });
    });

    it('should return requested NSSAI as-is when none are default', () => {
      const requestedNssai: Snssai[] = [
        { sst: 1 },
        { sst: 2 }
      ];
      const subscribedNssai = [
        { subscribedSnssai: { sst: 1 } },
        { subscribedSnssai: { sst: 2 } }
      ];

      const result = prioritizeRequestedNssai(requestedNssai, subscribedNssai);
      expect(result).to.deep.equal(requestedNssai);
    });
  });
});
