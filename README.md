# NSSF

**This repo is not production-ready yet. I am still developing the core features.**

The NSSF (Network Slice Selection Function) is a core component of the 5G architecture. It is responsible for selecting the Network Slice instances to serve a UE, or a PDU Session.

`nssf-typescript` is a TypeScript implementation of 3GPP's 5G NSSF specification. The specification can be found [here](https://portal.3gpp.org/desktopmodules/Specifications/SpecificationDetails.aspx?specificationId=3407).

## Pre-requisites

1. Set up MongoDB

In `.env`:
```
MONGODB_URI=mongodb+srv://...
MONGODB_DB_NAME=nssf
MONGODB_COLLECTION_NAME=slices
PORT=8080
```

## Start NSSF
1. `npm install`
2. `npm run dev`

### Run tests
1. `npm test`

My testing framework of choice is Mocha.

## API Endpoints

### Nnssf_NSSelection Service

- `GET /nnssf-nsselection/v2/network-slice-information` - Retrieve the Network Slice Selection Information

### Nnssf_NSSAIAvailability Service

- `POST /nnssf-nssaiavailability/v1/subscriptions` - Subscribe to NSSAI availability notifications
- `GET /nnssf-nssaiavailability/v1/subscriptions/:subscriptionId` - Get NSSAI availability subscription
- `PATCH /nnssf-nssaiavailability/v1/subscriptions/:subscriptionId` - Update NSSAI availability subscription
- `DELETE /nnssf-nssaiavailability/v1/subscriptions/:subscriptionId` - Unsubscribe from NSSAI availability notifications

### Configuration Management API

#### Slice Configuration
- `GET /nssf-config/v1/slices` - Get all slice configurations
- `GET /nssf-config/v1/slices/:sst/:sd?` - Get specific slice configuration
- `POST /nssf-config/v1/slices` - Create slice configuration
- `PUT /nssf-config/v1/slices/:sst/:sd?` - Update slice configuration
- `DELETE /nssf-config/v1/slices/:sst/:sd?` - Delete slice configuration

#### NSI Configuration
- `GET /nssf-config/v1/nsi` - Get all NSI configurations
- `GET /nssf-config/v1/nsi/:nsiId` - Get specific NSI configuration
- `POST /nssf-config/v1/nsi` - Create NSI configuration
- `PUT /nssf-config/v1/nsi/:nsiId` - Update NSI configuration
- `DELETE /nssf-config/v1/nsi/:nsiId` - Delete NSI configuration

#### AMF Set Configuration
- `GET /nssf-config/v1/amf-sets` - Get all AMF Set configurations
- `GET /nssf-config/v1/amf-sets/:amfSetId` - Get specific AMF Set configuration
- `POST /nssf-config/v1/amf-sets` - Create AMF Set configuration
- `PUT /nssf-config/v1/amf-sets/:amfSetId` - Update AMF Set configuration
- `DELETE /nssf-config/v1/amf-sets/:amfSetId` - Delete AMF Set configuration

#### AMF Service Set Configuration
- `GET /nssf-config/v1/amf-service-sets` - Get all AMF Service Set configurations
- `GET /nssf-config/v1/amf-service-sets/:amfServiceSetId` - Get specific AMF Service Set configuration
- `POST /nssf-config/v1/amf-service-sets` - Create AMF Service Set configuration
- `PUT /nssf-config/v1/amf-service-sets/:amfServiceSetId` - Update AMF Service Set configuration
- `DELETE /nssf-config/v1/amf-service-sets/:amfServiceSetId` - Delete AMF Service Set configuration

#### AMF Instance Configuration
- `GET /nssf-config/v1/amf-instances` - Get all AMF Instance configurations
- `GET /nssf-config/v1/amf-instances/:nfInstanceId` - Get specific AMF Instance configuration
- `POST /nssf-config/v1/amf-instances` - Create AMF Instance configuration
- `PUT /nssf-config/v1/amf-instances/:nfInstanceId` - Update AMF Instance configuration
- `DELETE /nssf-config/v1/amf-instances/:nfInstanceId` - Delete AMF Instance configuration

#### Subscription Management
- `GET /nssf-config/v1/subscriptions/:supi` - Get subscription by SUPI
- `POST /nssf-config/v1/subscriptions` - Create subscription
- `PUT /nssf-config/v1/subscriptions/:supi` - Update subscription
- `DELETE /nssf-config/v1/subscriptions/:supi` - Delete subscription

#### Policy Management
- `GET /nssf-config/v1/policies` - Get all slice policies
- `GET /nssf-config/v1/policies/:policyId` - Get specific policy configuration
- `GET /nssf-config/v1/policies/snssai/:sst/:sd?` - Get policies for specific S-NSSAI
- `POST /nssf-config/v1/policies` - Create policy configuration
- `PUT /nssf-config/v1/policies/:policyId` - Update policy configuration
- `DELETE /nssf-config/v1/policies/:policyId` - Delete policy configuration

#### S-NSSAI Mapping Management
- `GET /nssf-config/v1/snssai-mappings` - Get all S-NSSAI mappings
- `GET /nssf-config/v1/snssai-mappings/:mappingId` - Get specific S-NSSAI mapping
- `POST /nssf-config/v1/snssai-mappings` - Create S-NSSAI mapping
- `PUT /nssf-config/v1/snssai-mappings/:mappingId` - Update S-NSSAI mapping
- `DELETE /nssf-config/v1/snssai-mappings/:mappingId` - Delete S-NSSAI mapping

#### NSAG Configuration
- `GET /nssf-config/v1/nsags` - Get all NSAG configurations
- `GET /nssf-config/v1/nsags/:nsagId` - Get specific NSAG configuration
- `POST /nssf-config/v1/nsags` - Create NSAG configuration
- `PUT /nssf-config/v1/nsags/:nsagId` - Update NSAG configuration
- `DELETE /nssf-config/v1/nsags/:nsagId` - Delete NSAG configuration

## IMPLEMENTED FEATURES

- Basic server setup with Express
- MongoDB connection
- TypeScript types from 3GPP OpenAPI specification
- Health check endpoint
- Network Slice Selection endpoint skeleton
- Network Slice Selection for UE Registration
- Network Slice Selection for PDU Session Establishment
- Network Slice Selection for UE Configuration Update
- Subscribed S-NSSAI storage and retrieval
- Subscriber profile management in MongoDB
- Slice subscription validation
- NSI selection logic
- NSI information provisioning (NRF URIs, NSI IDs)
- Per-NSI NRF endpoint configuration
- NSI-specific OAuth2 requirements
- Target AMF Set selection
- Target AMF Service Set selection
- Candidate AMF list generation
- AMF Set NRF discovery endpoint selection
- Redirection responses (307/308) with NRF information
- Configuration Management REST API
- Slice configuration CRUD operations
- NSI configuration CRUD operations
- AMF Set configuration CRUD operations
- AMF Service Set configuration CRUD operations
- AMF Instance configuration CRUD operations
- Subscription management REST API endpoints
- NRF discovery for AMF instances
- NRF OAuth2 token endpoint integration
- NRF NFManagement service integration
- Per-service OAuth2 requirement handling
- NSSAI availability subscription (POST)
- NSSAI availability unsubscribe (DELETE)
- NSSAI availability notification
- NSSAI availability update (PATCH)
- TA-based NSSAI availability management
- Default S-NSSAI indication handling
- 3GPP Problem Details error responses
- Request validation and input sanitization
- Database connection error handling
- Database operation error handling with retries
- NRF discovery failure handling
- NRF OAuth2 token acquisition error handling
- Malformed request validation
- Standardized error responses across all endpoints
- Allowed NSSAI determination based on subscription and policy
- Policy-based slice access control
- Time-based slice access policies
- Location-based (TAI) slice access policies
- Load-based slice access policies
- Policy configuration management
- Policy evaluation engine
- S-NSSAI mapping between serving and home network
- Mapped home S-NSSAI handling
- S-NSSAI mapping request processing
- S-NSSAI mapping configuration storage
- S-NSSAI mapping configuration management API
- Configured NSSAI determination with mapping support
- Requested NSSAI validation and processing
- Requested NSSAI format validation
- Requested NSSAI duplicate detection
- Requested NSSAI maximum limit enforcement (8 S-NSSAI)
- Requested NSSAI subscription filtering
- Requested NSSAI prioritization based on default indication
- Roaming indication processing (non-roaming, local breakout, home-routed)
- Home-routed roaming slice selection
- Local breakout roaming slice selection
- VPLMN/HPLMN S-NSSAI mapping for roaming
- Home PLMN identification and handling
- Default configured S-NSSAI handling
- Supported features parameter handling
- Feature negotiation between NF consumer and NSSF
- Required features for target slice instances
- NF capability-based slice selection
- Network Slice Admission Group (NSAG) support
- NSAG to S-NSSAI association
- NSAG information provisioning with TAI ranges
- NSAG-based admission decisions
- NSAG configuration management
- NSAG capacity control and UE count tracking

## NOT IMPLEMENTED FEATURES

### Geographical & Topology Support

### Subscription & Policy Management
- Network Slice-Specific Registration Group (NSSRG) support
- UE NSSRG support indication handling
- NSSRG suppression indication handling

### Security & Authorization
- OAuth2 client credentials flow
- NRF-based OAuth2 token acquisition
- Per-NRF OAuth2 requirement configuration
- Secure slice information access control

