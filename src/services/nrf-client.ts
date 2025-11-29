import axios from 'axios';
import {
  AccessTokenRequest,
  AccessTokenResponse,
  AccessTokenCacheEntry,
  NFProfile,
  SearchResult,
  NfType
} from '../types/nrf-types';
import { PlmnId, Snssai, Tai, NfInstanceId, Uri } from '../types/common-types';
import { NrfError, handleNrfError } from '../utils/errors';

const tokenCache = new Map<string, AccessTokenCacheEntry>();

type NrfClientConfig = {
  nrfId: Uri;
  nrfNfMgtUri?: Uri;
  nrfAccessTokenUri?: Uri;
  nfInstanceId: NfInstanceId;
};

const getCacheKey = (nrfId: Uri, scope: string): string => {
  return `${nrfId}:${scope}`;
};

const isTokenValid = (cacheEntry: AccessTokenCacheEntry): boolean => {
  return Date.now() < cacheEntry.expiresAt - 30000;
};

export const acquireAccessToken = async (
  config: NrfClientConfig,
  scope: string,
  targetNfType?: NfType,
  targetNfInstanceId?: NfInstanceId
): Promise<string | null> => {
  if (!config.nrfAccessTokenUri) {
    return null;
  }

  const cacheKey = getCacheKey(config.nrfId, scope);
  const cachedToken = tokenCache.get(cacheKey);

  if (cachedToken && isTokenValid(cachedToken)) {
    return cachedToken.token;
  }

  try {
    const tokenRequest: AccessTokenRequest = {
      grant_type: 'client_credentials',
      nfInstanceId: config.nfInstanceId,
      scope,
      targetNfType,
      targetNfInstanceId
    };

    const response = await axios.post<AccessTokenResponse>(
      config.nrfAccessTokenUri,
      new URLSearchParams(tokenRequest as any),
      {
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        },
        timeout: 5000
      }
    );

    const { access_token, expires_in } = response.data;
    const expiresAt = Date.now() + expires_in * 1000;

    tokenCache.set(cacheKey, {
      token: access_token,
      expiresAt,
      scope
    });

    return access_token;
  } catch (error) {
    console.error('Failed to acquire access token from NRF:', error);
    const nrfError = handleNrfError(error, config.nrfAccessTokenUri);
    throw nrfError;
  }
};

type DiscoverAmfParams = {
  targetPlmnList?: PlmnId[];
  targetNsiList?: string[];
  targetSnssaiList?: Snssai[];
  amfRegionId?: string;
  amfSetId?: string;
  taiList?: Tai[];
  limit?: number;
};

export const discoverAmfInstances = async (
  config: NrfClientConfig,
  params: DiscoverAmfParams,
  oauth2Required?: boolean
): Promise<NFProfile[]> => {
  if (!config.nrfNfMgtUri) {
    console.warn('NRF NF Management URI not configured');
    return [];
  }

  try {
    let headers: Record<string, string> = {
      'Accept': 'application/json'
    };

    if (oauth2Required) {
      try {
        const token = await acquireAccessToken(
          config,
          'nnrf-disc',
          NfType.AMF
        );

        if (token) {
          headers['Authorization'] = `Bearer ${token}`;
        }
      } catch (tokenError) {
        console.error('Failed to acquire OAuth2 token for NRF discovery:', tokenError);
        throw tokenError;
      }
    }

    const queryParams = new URLSearchParams();
    queryParams.append('target-nf-type', NfType.AMF);

    if (params.targetPlmnList && params.targetPlmnList.length > 0) {
      params.targetPlmnList.forEach(plmn => {
        queryParams.append('target-plmn-list', JSON.stringify(plmn));
      });
    }

    if (params.targetNsiList && params.targetNsiList.length > 0) {
      params.targetNsiList.forEach(nsi => {
        queryParams.append('target-nsi-list', nsi);
      });
    }

    if (params.targetSnssaiList && params.targetSnssaiList.length > 0) {
      params.targetSnssaiList.forEach(snssai => {
        queryParams.append('snssais', JSON.stringify(snssai));
      });
    }

    if (params.amfRegionId) {
      queryParams.append('amf-region-id', params.amfRegionId);
    }

    if (params.amfSetId) {
      queryParams.append('amf-set-id', params.amfSetId);
    }

    if (params.taiList && params.taiList.length > 0) {
      params.taiList.forEach(tai => {
        queryParams.append('tai', JSON.stringify(tai));
      });
    }

    if (params.limit) {
      queryParams.append('limit', params.limit.toString());
    }

    const discoveryUrl = `${config.nrfNfMgtUri}/nf-instances?${queryParams.toString()}`;

    const response = await axios.get<SearchResult>(discoveryUrl, {
      headers,
      timeout: 5000
    });

    return response.data.nfInstances || [];
  } catch (error) {
    if (error instanceof NrfError) {
      throw error;
    }

    console.error('Failed to discover AMF instances from NRF:', error);
    const nrfError = handleNrfError(error, config.nrfNfMgtUri);
    throw nrfError;
  }
};

export const getNfProfile = async (
  config: NrfClientConfig,
  nfInstanceId: NfInstanceId,
  oauth2Required?: boolean
): Promise<NFProfile | null> => {
  if (!config.nrfNfMgtUri) {
    console.warn('NRF NF Management URI not configured');
    return null;
  }

  try {
    let headers: Record<string, string> = {
      'Accept': 'application/json'
    };

    if (oauth2Required) {
      try {
        const token = await acquireAccessToken(
          config,
          'nnrf-nfm',
          undefined,
          nfInstanceId
        );

        if (token) {
          headers['Authorization'] = `Bearer ${token}`;
        }
      } catch (tokenError) {
        console.error('Failed to acquire OAuth2 token for NF profile retrieval:', tokenError);
        throw tokenError;
      }
    }

    const profileUrl = `${config.nrfNfMgtUri}/nf-instances/${nfInstanceId}`;

    const response = await axios.get<NFProfile>(profileUrl, {
      headers,
      timeout: 5000
    });

    return response.data;
  } catch (error) {
    if (error instanceof NrfError) {
      throw error;
    }

    console.error(`Failed to get NF profile for ${nfInstanceId}:`, error);
    const nrfError = handleNrfError(error, config.nrfNfMgtUri);
    throw nrfError;
  }
};

export const clearTokenCache = (): void => {
  tokenCache.clear();
};
