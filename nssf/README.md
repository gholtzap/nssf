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

## IMPLEMENTED FEATURES

- Basic server setup with Express
- MongoDB connection
- TypeScript types from 3GPP OpenAPI specification
- Health check endpoint
- Network Slice Selection endpoint skeleton
- Network Slice Selection for UE Registration

## NOT IMPLEMENTED FEATURES

### Core Network Slice Selection
- Network Slice Selection for PDU Session Establishment
- Network Slice Selection for UE Configuration Update
- Allowed NSSAI determination based on subscription and policy
- Configured NSSAI determination
- Rejected NSSAI handling (in PLMN and in TA)
- Default S-NSSAI indication handling
- Requested NSSAI validation and processing

### S-NSSAI Mapping & Translation
- S-NSSAI mapping between serving and home network
- Mapped home S-NSSAI handling
- S-NSSAI mapping request processing
- NSSAI mapping configuration storage

### Network Slice Instance (NSI) Management
- NSI selection logic
- NSI information provisioning (NRF URIs, NSI IDs)
- Per-NSI NRF endpoint configuration
- NSI-specific OAuth2 requirements

### AMF Selection & Redirection
- Target AMF Set selection
- Target AMF Service Set selection
- Candidate AMF list generation
- AMF Set NRF discovery endpoint selection
- Redirection responses (307/308) with NRF information

### NRF Integration
- NRF discovery for AMF instances
- NRF OAuth2 token endpoint integration
- NRF NFManagement service integration
- Per-service OAuth2 requirement handling

### Geographical & Topology Support
- Tracking Area (TAI) based slice availability
- TA-specific rejected NSSAI handling
- PLMN-specific slice configuration
- Home PLMN identification and handling

### Roaming Support
- Roaming indication processing (non-roaming, local breakout, home-routed)
- Home-routed roaming slice selection
- Local breakout roaming slice selection
- VPLMN/HPLMN S-NSSAI mapping

### Subscription & Policy Management
- Subscribed S-NSSAI storage and retrieval
- Subscriber profile management in MongoDB
- Slice subscription validation
- Default configured S-NSSAI handling
- Network Slice-Specific Registration Group (NSSRG) support
- UE NSSRG support indication handling
- NSSRG suppression indication handling

### Network Slice Admission Control
- Network Slice Admission Group (NSAG) support
- NSAG to S-NSSAI association
- NSAG information provisioning with TAI ranges
- NSAG-based admission decisions

### Feature Negotiation & Capabilities
- Supported features parameter handling
- Feature negotiation between NF consumer and NSSF
- Required features for target slice instances
- NF capability-based slice selection

### Nnssf_NSSAIAvailability Service
- NSSAI availability subscription (POST)
- NSSAI availability unsubscribe (DELETE)
- NSSAI availability notification
- NSSAI availability update (PATCH)
- TA-based NSSAI availability management

### Security & Authorization
- OAuth2 client credentials flow
- NRF-based OAuth2 token acquisition
- Per-NRF OAuth2 requirement configuration
- Secure slice information access control

### Configuration & Management
- Slice configuration storage in MongoDB
- Network slice policy configuration
- TA range configuration for slices
- PLMN-wide slice configuration
- Slice priority and QoS policies

### Error Handling & Edge Cases
- Invalid NSSAI request handling
- Slice unavailability scenarios
- NRF discovery failure handling
- Database connection error handling
- Malformed request validation