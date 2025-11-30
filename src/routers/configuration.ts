import { Router, Request, Response } from 'express';
import {
  getAllSliceConfigurations,
  getSliceConfiguration,
  createSliceConfiguration,
  updateSliceConfiguration,
  deleteSliceConfiguration
} from '../services/slice-configuration';
import {
  getAllNsiConfigurations,
  getNsiConfiguration,
  getNsiConfigurationsBySnssai,
  createNsiConfiguration,
  updateNsiConfiguration,
  deleteNsiConfiguration
} from '../services/nsi-configuration';
import {
  getAllAmfSets,
  getAmfSet,
  createAmfSet,
  updateAmfSet,
  deleteAmfSet,
  getAllAmfServiceSets,
  getAmfServiceSet,
  createAmfServiceSet,
  updateAmfServiceSet,
  deleteAmfServiceSet,
  getAllAmfInstances,
  getAmfInstance,
  createAmfInstance,
  updateAmfInstance,
  deleteAmfInstance
} from '../services/amf-configuration';
import {
  getSubscriptionBySupi,
  createSubscription,
  updateSubscription,
  deleteSubscription
} from '../services/subscription';
import {
  getAllPolicies,
  getPoliciesBySnssai,
  getPolicyById,
  createPolicy,
  updatePolicy,
  deletePolicy
} from '../services/policy-configuration';
import {
  getAllSnssaiMappings,
  getSnssaiMappingById,
  createSnssaiMapping,
  updateSnssaiMapping,
  deleteSnssaiMapping
} from '../services/snssai-mapping';
import {
  getAllNsagConfigurations,
  getNsagConfiguration,
  createNsagConfiguration,
  updateNsagConfiguration,
  deleteNsagConfiguration
} from '../services/nsag-configuration';
import { handleError } from '../utils/error-handler';
import { createProblemDetails } from '../types/problem-details-types';

const router = Router();

router.get('/slices', async (req: Request, res: Response) => {
  try {
    const slices = await getAllSliceConfigurations();
    res.json(slices);
  } catch (error) {
    handleError(error, res, 'GET /slices');
  }
});

router.get('/slices/:sst/:sd?', async (req: Request, res: Response) => {
  try {
    const sst = parseInt(req.params.sst);
    const sd = req.params.sd;
    const plmnIdRaw = req.query.plmnId as string;

    if (!plmnIdRaw) {
      return res.status(400).json(createProblemDetails(
        400,
        'Bad Request',
        'plmnId query parameter is required'
      ));
    }

    const plmnId = JSON.parse(plmnIdRaw);
    const snssai = { sst, sd };

    const slice = await getSliceConfiguration(snssai, plmnId);
    if (!slice) {
      return res.status(404).json(createProblemDetails(
        404,
        'Not Found',
        'Slice configuration not found'
      ));
    }

    res.json(slice);
  } catch (error) {
    handleError(error, res, 'GET /slices/:sst/:sd');
  }
});

router.post('/slices', async (req: Request, res: Response) => {
  try {
    await createSliceConfiguration(req.body);
    res.status(201).json({ message: 'Slice configuration created successfully' });
  } catch (error: any) {
    if (error.message?.includes('already exists')) {
      return res.status(409).json(createProblemDetails(
        409,
        'Conflict',
        'Slice configuration already exists',
        error.message
      ));
    }
    handleError(error, res, 'POST /slices');
  }
});

router.put('/slices/:sst/:sd?', async (req: Request, res: Response) => {
  try {
    const sst = parseInt(req.params.sst);
    const sd = req.params.sd;
    const plmnIdRaw = req.query.plmnId as string;

    if (!plmnIdRaw) {
      return res.status(400).json({ error: 'plmnId query parameter is required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    const snssai = { sst, sd };

    await updateSliceConfiguration(snssai, plmnId, req.body);
    res.json({ message: 'Slice configuration updated successfully' });
  } catch (error: any) {
    console.error('Error updating slice configuration:', error);
    if (error.message?.includes('not found')) {
      return res.status(404).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.delete('/slices/:sst/:sd?', async (req: Request, res: Response) => {
  try {
    const sst = parseInt(req.params.sst);
    const sd = req.params.sd;
    const plmnIdRaw = req.query.plmnId as string;

    if (!plmnIdRaw) {
      return res.status(400).json({ error: 'plmnId query parameter is required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    const snssai = { sst, sd };

    await deleteSliceConfiguration(snssai, plmnId);
    res.json({ message: 'Slice configuration deleted successfully' });
  } catch (error: any) {
    console.error('Error deleting slice configuration:', error);
    if (error.message?.includes('not found')) {
      return res.status(404).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.get('/nsi', async (req: Request, res: Response) => {
  try {
    const snssaiRaw = req.query.snssai as string | undefined;
    const plmnIdRaw = req.query.plmnId as string | undefined;

    if (snssaiRaw && plmnIdRaw) {
      const snssai = JSON.parse(snssaiRaw);
      const plmnId = JSON.parse(plmnIdRaw);
      const nsis = await getNsiConfigurationsBySnssai(snssai, plmnId);
      return res.json(nsis);
    }

    const nsis = await getAllNsiConfigurations();
    res.json(nsis);
  } catch (error) {
    console.error('Error getting NSI configurations:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.get('/nsi/:nsiId', async (req: Request, res: Response) => {
  try {
    const nsiId = req.params.nsiId;
    const nsi = await getNsiConfiguration(nsiId);

    if (!nsi) {
      return res.status(404).json({ error: 'NSI configuration not found' });
    }

    res.json(nsi);
  } catch (error) {
    console.error('Error getting NSI configuration:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.post('/nsi', async (req: Request, res: Response) => {
  try {
    await createNsiConfiguration(req.body);
    res.status(201).json({ message: 'NSI configuration created successfully' });
  } catch (error: any) {
    console.error('Error creating NSI configuration:', error);
    if (error.message?.includes('already exists')) {
      return res.status(409).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.put('/nsi/:nsiId', async (req: Request, res: Response) => {
  try {
    const nsiId = req.params.nsiId;
    await updateNsiConfiguration(nsiId, req.body);
    res.json({ message: 'NSI configuration updated successfully' });
  } catch (error: any) {
    console.error('Error updating NSI configuration:', error);
    if (error.message?.includes('not found')) {
      return res.status(404).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.delete('/nsi/:nsiId', async (req: Request, res: Response) => {
  try {
    const nsiId = req.params.nsiId;
    await deleteNsiConfiguration(nsiId);
    res.json({ message: 'NSI configuration deleted successfully' });
  } catch (error: any) {
    console.error('Error deleting NSI configuration:', error);
    if (error.message?.includes('not found')) {
      return res.status(404).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.get('/amf-sets', async (req: Request, res: Response) => {
  try {
    const amfSets = await getAllAmfSets();
    res.json(amfSets);
  } catch (error) {
    console.error('Error getting AMF Sets:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.get('/amf-sets/:amfSetId', async (req: Request, res: Response) => {
  try {
    const amfSetId = req.params.amfSetId;
    const plmnIdRaw = req.query.plmnId as string;

    if (!plmnIdRaw) {
      return res.status(400).json({ error: 'plmnId query parameter is required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    const amfSet = await getAmfSet(amfSetId, plmnId);

    if (!amfSet) {
      return res.status(404).json({ error: 'AMF Set not found' });
    }

    res.json(amfSet);
  } catch (error) {
    console.error('Error getting AMF Set:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.post('/amf-sets', async (req: Request, res: Response) => {
  try {
    await createAmfSet(req.body);
    res.status(201).json({ message: 'AMF Set created successfully' });
  } catch (error: any) {
    console.error('Error creating AMF Set:', error);
    if (error.message?.includes('already exists')) {
      return res.status(409).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.put('/amf-sets/:amfSetId', async (req: Request, res: Response) => {
  try {
    const amfSetId = req.params.amfSetId;
    const plmnIdRaw = req.query.plmnId as string;

    if (!plmnIdRaw) {
      return res.status(400).json({ error: 'plmnId query parameter is required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    await updateAmfSet(amfSetId, plmnId, req.body);
    res.json({ message: 'AMF Set updated successfully' });
  } catch (error: any) {
    console.error('Error updating AMF Set:', error);
    if (error.message?.includes('not found')) {
      return res.status(404).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.delete('/amf-sets/:amfSetId', async (req: Request, res: Response) => {
  try {
    const amfSetId = req.params.amfSetId;
    const plmnIdRaw = req.query.plmnId as string;

    if (!plmnIdRaw) {
      return res.status(400).json({ error: 'plmnId query parameter is required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    await deleteAmfSet(amfSetId, plmnId);
    res.json({ message: 'AMF Set deleted successfully' });
  } catch (error: any) {
    console.error('Error deleting AMF Set:', error);
    if (error.message?.includes('not found')) {
      return res.status(404).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.get('/amf-service-sets', async (req: Request, res: Response) => {
  try {
    const serviceSets = await getAllAmfServiceSets();
    res.json(serviceSets);
  } catch (error) {
    console.error('Error getting AMF Service Sets:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.get('/amf-service-sets/:amfServiceSetId', async (req: Request, res: Response) => {
  try {
    const amfServiceSetId = req.params.amfServiceSetId;
    const amfSetId = req.query.amfSetId as string;
    const plmnIdRaw = req.query.plmnId as string;

    if (!amfSetId || !plmnIdRaw) {
      return res.status(400).json({ error: 'amfSetId and plmnId query parameters are required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    const serviceSet = await getAmfServiceSet(amfServiceSetId, amfSetId, plmnId);

    if (!serviceSet) {
      return res.status(404).json({ error: 'AMF Service Set not found' });
    }

    res.json(serviceSet);
  } catch (error) {
    console.error('Error getting AMF Service Set:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.post('/amf-service-sets', async (req: Request, res: Response) => {
  try {
    await createAmfServiceSet(req.body);
    res.status(201).json({ message: 'AMF Service Set created successfully' });
  } catch (error: any) {
    console.error('Error creating AMF Service Set:', error);
    if (error.message?.includes('already exists')) {
      return res.status(409).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.put('/amf-service-sets/:amfServiceSetId', async (req: Request, res: Response) => {
  try {
    const amfServiceSetId = req.params.amfServiceSetId;
    const amfSetId = req.query.amfSetId as string;
    const plmnIdRaw = req.query.plmnId as string;

    if (!amfSetId || !plmnIdRaw) {
      return res.status(400).json({ error: 'amfSetId and plmnId query parameters are required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    await updateAmfServiceSet(amfServiceSetId, amfSetId, plmnId, req.body);
    res.json({ message: 'AMF Service Set updated successfully' });
  } catch (error: any) {
    console.error('Error updating AMF Service Set:', error);
    if (error.message?.includes('not found')) {
      return res.status(404).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.delete('/amf-service-sets/:amfServiceSetId', async (req: Request, res: Response) => {
  try {
    const amfServiceSetId = req.params.amfServiceSetId;
    const amfSetId = req.query.amfSetId as string;
    const plmnIdRaw = req.query.plmnId as string;

    if (!amfSetId || !plmnIdRaw) {
      return res.status(400).json({ error: 'amfSetId and plmnId query parameters are required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    await deleteAmfServiceSet(amfServiceSetId, amfSetId, plmnId);
    res.json({ message: 'AMF Service Set deleted successfully' });
  } catch (error: any) {
    console.error('Error deleting AMF Service Set:', error);
    if (error.message?.includes('not found')) {
      return res.status(404).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.get('/amf-instances', async (req: Request, res: Response) => {
  try {
    const instances = await getAllAmfInstances();
    res.json(instances);
  } catch (error) {
    console.error('Error getting AMF Instances:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.get('/amf-instances/:nfInstanceId', async (req: Request, res: Response) => {
  try {
    const nfInstanceId = req.params.nfInstanceId;
    const instance = await getAmfInstance(nfInstanceId);

    if (!instance) {
      return res.status(404).json({ error: 'AMF Instance not found' });
    }

    res.json(instance);
  } catch (error) {
    console.error('Error getting AMF Instance:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.post('/amf-instances', async (req: Request, res: Response) => {
  try {
    await createAmfInstance(req.body);
    res.status(201).json({ message: 'AMF Instance created successfully' });
  } catch (error: any) {
    console.error('Error creating AMF Instance:', error);
    if (error.message?.includes('already exists')) {
      return res.status(409).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.put('/amf-instances/:nfInstanceId', async (req: Request, res: Response) => {
  try {
    const nfInstanceId = req.params.nfInstanceId;
    await updateAmfInstance(nfInstanceId, req.body);
    res.json({ message: 'AMF Instance updated successfully' });
  } catch (error: any) {
    console.error('Error updating AMF Instance:', error);
    if (error.message?.includes('not found')) {
      return res.status(404).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.delete('/amf-instances/:nfInstanceId', async (req: Request, res: Response) => {
  try {
    const nfInstanceId = req.params.nfInstanceId;
    await deleteAmfInstance(nfInstanceId);
    res.json({ message: 'AMF Instance deleted successfully' });
  } catch (error: any) {
    console.error('Error deleting AMF Instance:', error);
    if (error.message?.includes('not found')) {
      return res.status(404).json({ error: error.message });
    }
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.get('/subscriptions/:supi', async (req: Request, res: Response) => {
  try {
    const supi = req.params.supi;
    const plmnIdRaw = req.query.plmnId as string;

    if (!plmnIdRaw) {
      return res.status(400).json({ error: 'plmnId query parameter is required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    const subscription = await getSubscriptionBySupi(supi, plmnId);

    if (!subscription) {
      return res.status(404).json({ error: 'Subscription not found' });
    }

    res.json(subscription);
  } catch (error) {
    console.error('Error getting subscription:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.post('/subscriptions', async (req: Request, res: Response) => {
  try {
    await createSubscription(req.body);
    res.status(201).json({ message: 'Subscription created successfully' });
  } catch (error: any) {
    console.error('Error creating subscription:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.put('/subscriptions/:supi', async (req: Request, res: Response) => {
  try {
    const supi = req.params.supi;
    const plmnIdRaw = req.query.plmnId as string;

    if (!plmnIdRaw) {
      return res.status(400).json({ error: 'plmnId query parameter is required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    await updateSubscription(supi, plmnId, req.body);
    res.json({ message: 'Subscription updated successfully' });
  } catch (error: any) {
    console.error('Error updating subscription:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.delete('/subscriptions/:supi', async (req: Request, res: Response) => {
  try {
    const supi = req.params.supi;
    const plmnIdRaw = req.query.plmnId as string;

    if (!plmnIdRaw) {
      return res.status(400).json({ error: 'plmnId query parameter is required' });
    }

    const plmnId = JSON.parse(plmnIdRaw);
    await deleteSubscription(supi, plmnId);
    res.json({ message: 'Subscription deleted successfully' });
  } catch (error: any) {
    console.error('Error deleting subscription:', error);
    res.status(500).json({ error: 'Internal Server Error' });
  }
});

router.get('/policies', async (req: Request, res: Response) => {
  try {
    const policies = await getAllPolicies();
    res.json(policies);
  } catch (error) {
    handleError(error, res, 'GET /policies');
  }
});

router.get('/policies/:policyId', async (req: Request, res: Response) => {
  try {
    const policyId = req.params.policyId;
    const policy = await getPolicyById(policyId);

    if (!policy) {
      return res.status(404).json(createProblemDetails(
        404,
        'Not Found',
        'Policy not found'
      ));
    }

    res.json(policy);
  } catch (error) {
    handleError(error, res, 'GET /policies/:policyId');
  }
});

router.get('/policies/snssai/:sst/:sd?', async (req: Request, res: Response) => {
  try {
    const sst = parseInt(req.params.sst);
    const sd = req.params.sd;
    const plmnIdRaw = req.query.plmnId as string;

    if (!plmnIdRaw) {
      return res.status(400).json(createProblemDetails(
        400,
        'Bad Request',
        'plmnId query parameter is required'
      ));
    }

    const plmnId = JSON.parse(plmnIdRaw);
    const snssai = { sst, sd };

    const policies = await getPoliciesBySnssai(snssai, plmnId);
    res.json(policies);
  } catch (error) {
    handleError(error, res, 'GET /policies/snssai/:sst/:sd');
  }
});

router.post('/policies', async (req: Request, res: Response) => {
  try {
    await createPolicy(req.body);
    res.status(201).json({ message: 'Policy created successfully' });
  } catch (error: any) {
    if (error.message?.includes('already exists')) {
      return res.status(409).json(createProblemDetails(
        409,
        'Conflict',
        'Policy already exists',
        error.message
      ));
    }
    handleError(error, res, 'POST /policies');
  }
});

router.put('/policies/:policyId', async (req: Request, res: Response) => {
  try {
    const policyId = req.params.policyId;
    await updatePolicy(policyId, req.body);
    res.json({ message: 'Policy updated successfully' });
  } catch (error: any) {
    if (error.message?.includes('not found')) {
      return res.status(404).json(createProblemDetails(
        404,
        'Not Found',
        'Policy not found',
        error.message
      ));
    }
    handleError(error, res, 'PUT /policies/:policyId');
  }
});

router.delete('/policies/:policyId', async (req: Request, res: Response) => {
  try {
    const policyId = req.params.policyId;
    await deletePolicy(policyId);
    res.json({ message: 'Policy deleted successfully' });
  } catch (error: any) {
    if (error.message?.includes('not found')) {
      return res.status(404).json(createProblemDetails(
        404,
        'Not Found',
        'Policy not found',
        error.message
      ));
    }
    handleError(error, res, 'DELETE /policies/:policyId');
  }
});

router.get('/snssai-mappings', async (req: Request, res: Response) => {
  try {
    const mappings = await getAllSnssaiMappings();
    res.json(mappings);
  } catch (error) {
    handleError(error, res, 'GET /snssai-mappings');
  }
});

router.get('/snssai-mappings/:mappingId', async (req: Request, res: Response) => {
  try {
    const mappingId = req.params.mappingId;
    const mapping = await getSnssaiMappingById(mappingId);

    if (!mapping) {
      return res.status(404).json(createProblemDetails(
        404,
        'Not Found',
        'S-NSSAI mapping not found'
      ));
    }

    res.json(mapping);
  } catch (error) {
    handleError(error, res, 'GET /snssai-mappings/:mappingId');
  }
});

router.post('/snssai-mappings', async (req: Request, res: Response) => {
  try {
    const mapping = await createSnssaiMapping(req.body);
    res.status(201).json(mapping);
  } catch (error: any) {
    if (error.message?.includes('already exists')) {
      return res.status(409).json(createProblemDetails(
        409,
        'Conflict',
        'S-NSSAI mapping already exists',
        error.message
      ));
    }
    handleError(error, res, 'POST /snssai-mappings');
  }
});

router.put('/snssai-mappings/:mappingId', async (req: Request, res: Response) => {
  try {
    const mappingId = req.params.mappingId;
    const mapping = await updateSnssaiMapping(mappingId, req.body);

    if (!mapping) {
      return res.status(404).json(createProblemDetails(
        404,
        'Not Found',
        'S-NSSAI mapping not found'
      ));
    }

    res.json(mapping);
  } catch (error) {
    handleError(error, res, 'PUT /snssai-mappings/:mappingId');
  }
});

router.delete('/snssai-mappings/:mappingId', async (req: Request, res: Response) => {
  try {
    const mappingId = req.params.mappingId;
    const deleted = await deleteSnssaiMapping(mappingId);

    if (!deleted) {
      return res.status(404).json(createProblemDetails(
        404,
        'Not Found',
        'S-NSSAI mapping not found'
      ));
    }

    res.json({ message: 'S-NSSAI mapping deleted successfully' });
  } catch (error) {
    handleError(error, res, 'DELETE /snssai-mappings/:mappingId');
  }
});

router.get('/nsags', async (req: Request, res: Response) => {
  try {
    const nsags = await getAllNsagConfigurations();
    res.json(nsags);
  } catch (error) {
    handleError(error, res, 'GET /nsags');
  }
});

router.get('/nsags/:nsagId', async (req: Request, res: Response) => {
  try {
    const nsagId = parseInt(req.params.nsagId);
    const nsag = await getNsagConfiguration(nsagId);

    if (!nsag) {
      return res.status(404).json(createProblemDetails(
        404,
        'Not Found',
        'NSAG configuration not found'
      ));
    }

    res.json(nsag);
  } catch (error) {
    handleError(error, res, 'GET /nsags/:nsagId');
  }
});

router.post('/nsags', async (req: Request, res: Response) => {
  try {
    await createNsagConfiguration(req.body);
    res.status(201).json({ message: 'NSAG configuration created successfully' });
  } catch (error: any) {
    if (error.message?.includes('already exists')) {
      return res.status(409).json(createProblemDetails(
        409,
        'Conflict',
        'NSAG configuration already exists',
        error.message
      ));
    }
    handleError(error, res, 'POST /nsags');
  }
});

router.put('/nsags/:nsagId', async (req: Request, res: Response) => {
  try {
    const nsagId = parseInt(req.params.nsagId);
    await updateNsagConfiguration(nsagId, req.body);
    res.json({ message: 'NSAG configuration updated successfully' });
  } catch (error: any) {
    if (error.message?.includes('not found')) {
      return res.status(404).json(createProblemDetails(
        404,
        'Not Found',
        'NSAG configuration not found',
        error.message
      ));
    }
    handleError(error, res, 'PUT /nsags/:nsagId');
  }
});

router.delete('/nsags/:nsagId', async (req: Request, res: Response) => {
  try {
    const nsagId = parseInt(req.params.nsagId);
    await deleteNsagConfiguration(nsagId);
    res.json({ message: 'NSAG configuration deleted successfully' });
  } catch (error: any) {
    if (error.message?.includes('not found')) {
      return res.status(404).json(createProblemDetails(
        404,
        'Not Found',
        'NSAG configuration not found',
        error.message
      ));
    }
    handleError(error, res, 'DELETE /nsags/:nsagId');
  }
});

export default router;
