import { Router, Request, Response } from 'express';
import { AuthorizedNetworkSliceInfo, SliceInfoForRegistration, SliceInfoForPDUSession, SliceInfoForUEConfigurationUpdate } from '../types/nnssf-nsselection-types';
import { NFType, PlmnId, Tai, SupportedFeatures } from '../types/common-types';
import { selectNetworkSlicesForRegistration, selectNetworkSlicesForPDUSession, selectNetworkSlicesForUEConfigurationUpdate } from '../services/network-slice-selection';

const router = Router();

router.get('/network-slice-information', async (req: Request, res: Response) => {
  try {
    const nfType = req.query['nf-type'] as NFType;
    const nfId = req.query['nf-id'] as string;
    const supi = req.query['supi'] as string | undefined;
    const sliceInfoRequestForRegistrationRaw = req.query['slice-info-request-for-registration'] as string | undefined;
    const sliceInfoRequestForPduSessionRaw = req.query['slice-info-request-for-pdu-session'] as string | undefined;
    const sliceInfoRequestForUeCuRaw = req.query['slice-info-request-for-ue-cu'] as string | undefined;
    const homePlmnIdRaw = req.query['home-plmn-id'] as string | undefined;
    const taiRaw = req.query['tai'] as string | undefined;
    const supportedFeatures = req.query['supported-features'] as SupportedFeatures | undefined;

    if (!nfType || !nfId) {
      return res.status(400).json({
        error: 'Bad Request',
        message: 'nf-type and nf-id are required parameters'
      });
    }

    if (!supi) {
      return res.status(400).json({
        error: 'Bad Request',
        message: 'supi is required'
      });
    }

    if (!homePlmnIdRaw) {
      return res.status(400).json({
        error: 'Bad Request',
        message: 'home-plmn-id is required'
      });
    }

    let authorizedNetworkSliceInfo: AuthorizedNetworkSliceInfo;

    const homePlmnId: PlmnId = JSON.parse(homePlmnIdRaw);
    const tai: Tai | undefined = taiRaw ? JSON.parse(taiRaw) : undefined;

    if (sliceInfoRequestForRegistrationRaw) {
      const sliceInfoRequestForRegistration: SliceInfoForRegistration = JSON.parse(sliceInfoRequestForRegistrationRaw);

      authorizedNetworkSliceInfo = await selectNetworkSlicesForRegistration({
        sliceInfoForRegistration: sliceInfoRequestForRegistration,
        homePlmnId,
        supi,
        tai
      });
    } else if (sliceInfoRequestForPduSessionRaw) {
      const sliceInfoRequestForPduSession: SliceInfoForPDUSession = JSON.parse(sliceInfoRequestForPduSessionRaw);

      authorizedNetworkSliceInfo = await selectNetworkSlicesForPDUSession({
        sliceInfoForPDUSession: sliceInfoRequestForPduSession,
        homePlmnId,
        supi,
        tai
      });
    } else if (sliceInfoRequestForUeCuRaw) {
      const sliceInfoRequestForUeCu: SliceInfoForUEConfigurationUpdate = JSON.parse(sliceInfoRequestForUeCuRaw);

      authorizedNetworkSliceInfo = await selectNetworkSlicesForUEConfigurationUpdate({
        sliceInfoForUEConfigurationUpdate: sliceInfoRequestForUeCu,
        homePlmnId,
        supi,
        tai
      });
    } else {
      return res.status(400).json({
        error: 'Bad Request',
        message: 'One of slice-info-request-for-registration, slice-info-request-for-pdu-session, or slice-info-request-for-ue-cu is required'
      });
    }

    res.status(200).json(authorizedNetworkSliceInfo);
  } catch (error) {
    console.error('Error in network-slice-information endpoint:', error);
    res.status(500).json({
      error: 'Internal Server Error',
      message: 'An error occurred while processing the request'
    });
  }
});

export default router;
