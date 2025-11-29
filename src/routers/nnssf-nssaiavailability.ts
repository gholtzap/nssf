import { Router, Request, Response } from 'express';
import {
  NssaiAvailabilitySubscriptionCreateRequest,
  NssaiAvailabilitySubscriptionUpdateRequest
} from '../types/nnssf-nssaiavailability-types';
import {
  createNssaiAvailabilitySubscription,
  getNssaiAvailabilitySubscription,
  updateNssaiAvailabilitySubscription,
  deleteNssaiAvailabilitySubscription,
  getAuthorizedNssaiAvailabilityData
} from '../services/nssai-availability';

const router = Router();

router.post('/subscriptions', async (req: Request, res: Response) => {
  try {
    const request: NssaiAvailabilitySubscriptionCreateRequest = req.body;

    if (!request.nfInstanceId || !request.subscriptionData || !request.notificationUri) {
      return res.status(400).json({
        error: 'Bad Request',
        message: 'nfInstanceId, subscriptionData, and notificationUri are required'
      });
    }

    if (!request.subscriptionData.tai) {
      return res.status(400).json({
        error: 'Bad Request',
        message: 'subscriptionData.tai is required'
      });
    }

    const subscription = await createNssaiAvailabilitySubscription(request);

    const authorizedData = await getAuthorizedNssaiAvailabilityData(
      request.subscriptionData.tai,
      request.subscriptionData.supportedSnssaiList
    );

    res.status(201)
      .header('Location', `/nnssf-nssaiavailability/v1/subscriptions/${subscription.subscriptionId}`)
      .json({
        subscriptionId: subscription.subscriptionId,
        authorizedNssaiAvailabilityData: [authorizedData],
        supportedFeatures: subscription.supportedFeatures
      });
  } catch (error) {
    console.error('Error creating NSSAI availability subscription:', error);
    res.status(500).json({
      error: 'Internal Server Error',
      message: 'An error occurred while creating the subscription'
    });
  }
});

router.get('/subscriptions/:subscriptionId', async (req: Request, res: Response) => {
  try {
    const { subscriptionId } = req.params;

    const subscription = await getNssaiAvailabilitySubscription(subscriptionId);

    if (!subscription) {
      return res.status(404).json({
        error: 'Not Found',
        message: 'Subscription not found'
      });
    }

    const authorizedData = await getAuthorizedNssaiAvailabilityData(
      subscription.subscriptionData.tai,
      subscription.subscriptionData.supportedSnssaiList
    );

    res.status(200).json({
      subscriptionId: subscription.subscriptionId,
      nfInstanceId: subscription.nfInstanceId,
      subscriptionData: subscription.subscriptionData,
      notificationUri: subscription.notificationUri,
      authorizedNssaiAvailabilityData: [authorizedData],
      supportedFeatures: subscription.supportedFeatures,
      expiryTime: subscription.expiryTime
    });
  } catch (error) {
    console.error('Error retrieving NSSAI availability subscription:', error);
    res.status(500).json({
      error: 'Internal Server Error',
      message: 'An error occurred while retrieving the subscription'
    });
  }
});

router.patch('/subscriptions/:subscriptionId', async (req: Request, res: Response) => {
  try {
    const { subscriptionId } = req.params;
    const update: NssaiAvailabilitySubscriptionUpdateRequest = req.body;

    if (!update.subscriptionData) {
      return res.status(400).json({
        error: 'Bad Request',
        message: 'subscriptionData is required'
      });
    }

    if (!update.subscriptionData.tai) {
      return res.status(400).json({
        error: 'Bad Request',
        message: 'subscriptionData.tai is required'
      });
    }

    const updatedSubscription = await updateNssaiAvailabilitySubscription(subscriptionId, update);

    if (!updatedSubscription) {
      return res.status(404).json({
        error: 'Not Found',
        message: 'Subscription not found'
      });
    }

    const authorizedData = await getAuthorizedNssaiAvailabilityData(
      update.subscriptionData.tai,
      update.subscriptionData.supportedSnssaiList
    );

    res.status(200).json({
      subscriptionId: updatedSubscription.subscriptionId,
      authorizedNssaiAvailabilityData: [authorizedData],
      supportedFeatures: updatedSubscription.supportedFeatures
    });
  } catch (error) {
    console.error('Error updating NSSAI availability subscription:', error);
    res.status(500).json({
      error: 'Internal Server Error',
      message: 'An error occurred while updating the subscription'
    });
  }
});

router.delete('/subscriptions/:subscriptionId', async (req: Request, res: Response) => {
  try {
    const { subscriptionId } = req.params;

    const deleted = await deleteNssaiAvailabilitySubscription(subscriptionId);

    if (!deleted) {
      return res.status(404).json({
        error: 'Not Found',
        message: 'Subscription not found'
      });
    }

    res.status(204).send();
  } catch (error) {
    console.error('Error deleting NSSAI availability subscription:', error);
    res.status(500).json({
      error: 'Internal Server Error',
      message: 'An error occurred while deleting the subscription'
    });
  }
});

export default router;
