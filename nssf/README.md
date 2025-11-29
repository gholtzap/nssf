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

## NOT IMPLEMENTED FEATURES

- Network Slice Selection logic
- Network Slice instance selection
- S-NSSAI mapping
- NRF integration
- AMF set selection
- Subscription management
- Policy-based slice selection
- Multi-PLMN support
- Roaming scenarios
