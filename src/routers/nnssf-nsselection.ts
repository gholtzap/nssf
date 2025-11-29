import { Router, Request, Response } from 'express';
import { AuthorizedNetworkSliceInfo, SliceInfoForRegistration, SliceInfoForPDUSession, SliceInfoForUEConfigurationUpdate } from '../types/nnssf-nsselection-types';
import { NFType, PlmnId, Tai, SupportedFeatures } from '../types/common-types';
import { selectNetworkSlicesForRegistration, selectNetworkSlicesForPDUSession, selectNetworkSlicesForUEConfigurationUpdate } from '../services/network-slice-selection';
import { createProblemDetails, InvalidParam } from '../types/problem-details-types';
import { validateRequiredParam, validatePlmnId, validateTai, validateSupi, parseJsonParam, validateSnssai, extractHomePlmnFromSupi, validateHomePlmnConsistency } from '../utils/validation';
import { DatabaseError } from '../db/mongodb';
import { NrfError } from '../utils/errors';
import { validateRequestedNssai } from '../services/requested-nssai-validation';

const router = Router();

router.get('/network-slice-information', async (req: Request, res: Response) => {
  try {
    const invalidParams: InvalidParam[] = [];

    const nfType = req.query['nf-type'] as NFType;
    const nfId = req.query['nf-id'] as string;
    const supi = req.query['supi'] as string | undefined;
    const sliceInfoRequestForRegistrationRaw = req.query['slice-info-request-for-registration'] as string | undefined;
    const sliceInfoRequestForPduSessionRaw = req.query['slice-info-request-for-pdu-session'] as string | undefined;
    const sliceInfoRequestForUeCuRaw = req.query['slice-info-request-for-ue-cu'] as string | undefined;
    const homePlmnIdRaw = req.query['home-plmn-id'] as string | undefined;
    const taiRaw = req.query['tai'] as string | undefined;
    const supportedFeatures = req.query['supported-features'] as SupportedFeatures | undefined;

    const nfTypeError = validateRequiredParam(nfType, 'nf-type');
    if (nfTypeError) invalidParams.push(nfTypeError);

    const nfIdError = validateRequiredParam(nfId, 'nf-id');
    if (nfIdError) invalidParams.push(nfIdError);

    const supiError = validateSupi(supi);
    if (supiError) invalidParams.push(supiError);

    let homePlmnId: PlmnId | null = null;

    if (homePlmnIdRaw) {
      const { value: parsedHomePlmnId, error: homePlmnIdParseError } = parseJsonParam<PlmnId>(homePlmnIdRaw, 'home-plmn-id');
      if (homePlmnIdParseError) {
        invalidParams.push(homePlmnIdParseError);
      } else if (parsedHomePlmnId) {
        const plmnValidationError = validatePlmnId(parsedHomePlmnId, 'home-plmn-id');
        if (plmnValidationError) {
          invalidParams.push(plmnValidationError);
        } else {
          homePlmnId = parsedHomePlmnId;
        }
      }
    }

    if (supi && !homePlmnId) {
      homePlmnId = extractHomePlmnFromSupi(supi);
    }

    if (supi && homePlmnIdRaw && homePlmnId) {
      const consistencyError = validateHomePlmnConsistency(supi, homePlmnId);
      if (consistencyError) {
        invalidParams.push(consistencyError);
      }
    }

    if (!homePlmnId) {
      invalidParams.push({
        param: 'home-plmn-id',
        reason: 'is required and could not be extracted from SUPI'
      });
    }

    let tai: Tai | undefined = undefined;
    if (taiRaw) {
      const { value: taiValue, error: taiParseError } = parseJsonParam<Tai>(taiRaw, 'tai');
      if (taiParseError) {
        invalidParams.push(taiParseError);
      } else if (taiValue) {
        const taiValidationError = validateTai(taiValue, 'tai');
        if (taiValidationError) {
          invalidParams.push(taiValidationError);
        } else {
          tai = taiValue;
        }
      }
    }

    if (!sliceInfoRequestForRegistrationRaw && !sliceInfoRequestForPduSessionRaw && !sliceInfoRequestForUeCuRaw) {
      invalidParams.push({
        param: 'slice-info-request',
        reason: 'one of slice-info-request-for-registration, slice-info-request-for-pdu-session, or slice-info-request-for-ue-cu is required'
      });
    }

    if (invalidParams.length > 0) {
      return res.status(400).json(createProblemDetails(
        400,
        'Invalid request parameters',
        'The request contains invalid or missing parameters',
        undefined,
        invalidParams
      ));
    }

    let authorizedNetworkSliceInfo: AuthorizedNetworkSliceInfo;

    if (sliceInfoRequestForRegistrationRaw) {
      const { value: sliceInfoRequestForRegistration, error: parseError } =
        parseJsonParam<SliceInfoForRegistration>(sliceInfoRequestForRegistrationRaw, 'slice-info-request-for-registration');

      if (parseError || !sliceInfoRequestForRegistration) {
        return res.status(400).json(createProblemDetails(
          400,
          'Invalid slice-info-request-for-registration',
          'The slice-info-request-for-registration parameter must be valid JSON',
          undefined,
          parseError ? [parseError] : undefined
        ));
      }

      const requestedNssaiValidation = validateRequestedNssai(
        sliceInfoRequestForRegistration.requestedNssai,
        'slice-info-request-for-registration.requestedNssai'
      );

      if (!requestedNssaiValidation.isValid) {
        return res.status(400).json(createProblemDetails(
          400,
          'Invalid requested NSSAI',
          'The requested NSSAI is invalid',
          undefined,
          requestedNssaiValidation.invalidParams
        ));
      }

      authorizedNetworkSliceInfo = await selectNetworkSlicesForRegistration({
        sliceInfoForRegistration: sliceInfoRequestForRegistration,
        homePlmnId: homePlmnId!,
        supi: supi!,
        tai
      });
    } else if (sliceInfoRequestForPduSessionRaw) {
      const { value: sliceInfoRequestForPduSession, error: parseError } =
        parseJsonParam<SliceInfoForPDUSession>(sliceInfoRequestForPduSessionRaw, 'slice-info-request-for-pdu-session');

      if (parseError || !sliceInfoRequestForPduSession) {
        return res.status(400).json(createProblemDetails(
          400,
          'Invalid slice-info-request-for-pdu-session',
          'The slice-info-request-for-pdu-session parameter must be valid JSON',
          undefined,
          parseError ? [parseError] : undefined
        ));
      }

      const snssaiError = validateSnssai(sliceInfoRequestForPduSession.sNssai, 'slice-info-request-for-pdu-session.sNssai');
      if (snssaiError) {
        return res.status(400).json(createProblemDetails(
          400,
          'Invalid S-NSSAI',
          'The requested S-NSSAI is invalid',
          undefined,
          [snssaiError]
        ));
      }

      if (sliceInfoRequestForPduSession.homeSnssai) {
        const homeSnssaiError = validateSnssai(sliceInfoRequestForPduSession.homeSnssai, 'slice-info-request-for-pdu-session.homeSnssai');
        if (homeSnssaiError) {
          return res.status(400).json(createProblemDetails(
            400,
            'Invalid home S-NSSAI',
            'The home S-NSSAI is invalid',
            undefined,
            [homeSnssaiError]
          ));
        }
      }

      authorizedNetworkSliceInfo = await selectNetworkSlicesForPDUSession({
        sliceInfoForPDUSession: sliceInfoRequestForPduSession,
        homePlmnId: homePlmnId!,
        supi: supi!,
        tai
      });
    } else {
      const { value: sliceInfoRequestForUeCu, error: parseError } =
        parseJsonParam<SliceInfoForUEConfigurationUpdate>(sliceInfoRequestForUeCuRaw!, 'slice-info-request-for-ue-cu');

      if (parseError || !sliceInfoRequestForUeCu) {
        return res.status(400).json(createProblemDetails(
          400,
          'Invalid slice-info-request-for-ue-cu',
          'The slice-info-request-for-ue-cu parameter must be valid JSON',
          undefined,
          parseError ? [parseError] : undefined
        ));
      }

      const requestedNssaiValidation = validateRequestedNssai(
        sliceInfoRequestForUeCu.requestedNssai,
        'slice-info-request-for-ue-cu.requestedNssai'
      );

      if (!requestedNssaiValidation.isValid) {
        return res.status(400).json(createProblemDetails(
          400,
          'Invalid requested NSSAI',
          'The requested NSSAI is invalid',
          undefined,
          requestedNssaiValidation.invalidParams
        ));
      }

      authorizedNetworkSliceInfo = await selectNetworkSlicesForUEConfigurationUpdate({
        sliceInfoForUEConfigurationUpdate: sliceInfoRequestForUeCu,
        homePlmnId: homePlmnId!,
        supi: supi!,
        tai
      });
    }

    if (authorizedNetworkSliceInfo.targetAmfSet && authorizedNetworkSliceInfo.nrfAmfSet) {
      const location = authorizedNetworkSliceInfo.nrfAmfSetNfMgtUri || authorizedNetworkSliceInfo.nrfAmfSet;

      res.status(307).header('Location', location).json(authorizedNetworkSliceInfo);
    } else {
      res.status(200).json(authorizedNetworkSliceInfo);
    }
  } catch (error) {
    console.error('Error in network-slice-information endpoint:', error);

    if (error instanceof DatabaseError) {
      const status = error.isConnectionError ? 503 : 500;
      const problemDetails = createProblemDetails(
        status,
        error.isConnectionError ? 'Service Unavailable' : 'Internal Server Error',
        error.isConnectionError
          ? 'Unable to connect to database. Please try again later.'
          : 'A database error occurred while processing the request',
        error.message
      );
      return res.status(status).json(problemDetails);
    }

    if (error instanceof NrfError) {
      const status = error.isTimeout ? 504 : 503;
      const problemDetails = createProblemDetails(
        status,
        error.isTimeout ? 'Gateway Timeout' : 'Service Unavailable',
        error.isTimeout
          ? 'NRF discovery service timed out'
          : 'Unable to communicate with NRF discovery service',
        error.message
      );
      return res.status(status).json(problemDetails);
    }

    const problemDetails = createProblemDetails(
      500,
      'Internal Server Error',
      'An unexpected error occurred while processing the network slice selection request',
      error instanceof Error ? error.message : 'Unknown error'
    );
    res.status(500).json(problemDetails);
  }
});

export default router;
