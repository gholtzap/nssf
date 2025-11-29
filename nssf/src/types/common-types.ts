export type Snssai = {
  sst: number;
  sd?: string;
};

export type PlmnId = {
  mcc: string;
  mnc: string;
};

export type Tai = {
  plmnId: PlmnId;
  tac: string;
};

export type NfInstanceId = string;

export type Uri = string;

export type SupportedFeatures = string;

export type NfServiceSetId = string;

export type NsSrg = string;

export type NsagId = number;

export enum AccessType {
  THREE_GPP_ACCESS = '3GPP_ACCESS',
  NON_3GPP_ACCESS = 'NON_3GPP_ACCESS'
}

export enum NFType {
  NRF = 'NRF',
  UDM = 'UDM',
  AMF = 'AMF',
  SMF = 'SMF',
  AUSF = 'AUSF',
  NEF = 'NEF',
  PCF = 'PCF',
  SMSF = 'SMSF',
  NSSF = 'NSSF',
  UDR = 'UDR',
  LMF = 'LMF',
  GMLC = 'GMLC',
  FIVE_G_EIR = '5G_EIR',
  SEPP = 'SEPP',
  UPF = 'UPF',
  N3IWF = 'N3IWF',
  AF = 'AF',
  UDSF = 'UDSF',
  BSF = 'BSF',
  CHF = 'CHF',
  NWDAF = 'NWDAF',
  PCSCF = 'PCSCF',
  CBCF = 'CBCF',
  HSS = 'HSS',
  UCMF = 'UCMF',
  SOR_AF = 'SOR_AF',
  SPAF = 'SPAF',
  MME = 'MME',
  SCSAS = 'SCSAS',
  SCEF = 'SCEF',
  SCP = 'SCP',
  NSSAAF = 'NSSAAF',
  ICSCF = 'ICSCF',
  SCSCF = 'SCSCF',
  DRA = 'DRA',
  IMS_AS = 'IMS_AS',
  AANF = 'AANF',
  FIVE_G_DDNMF = '5G_DDNMF',
  NSACF = 'NSACF',
  MFAF = 'MFAF',
  EASDF = 'EASDF',
  DCCF = 'DCCF',
  MB_SMF = 'MB_SMF',
  TSCTSF = 'TSCTSF',
  ADRF = 'ADRF',
  GBA_BSF = 'GBA_BSF',
  CEF = 'CEF',
  MB_UPF = 'MB_UPF',
  NSWOF = 'NSWOF',
  PKMF = 'PKMF',
  MNPF = 'MNPF',
  SMS_GMSC = 'SMS_GMSC',
  SMS_IWMSC = 'SMS_IWMSC',
  MBSF = 'MBSF',
  MBSTF = 'MBSTF',
  PANF = 'PANF'
}
