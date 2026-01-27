CREATE OR REPLACE VIEW "DBAMV"."ORGANIZATION"
(
    "organization_code",
    "organization_name",
    "organization_alias",
    "organization_primary_doc",
    "organization_secondary_doc",
    "organization_third_document",
    "organization_type",
    "organization_location_code",
    "organization_address",
    "organization_number",
    "organization_complement",
    "organization_city",
    "organization_neighborhood",
    "organization_state",
    "organization_zipcode",
    "organization_person_resp",
    "organization_ddd_phone",
    "organization_phone",
    "organization_website",
    "organization_email",
    "organization_management_type",
    "organization_created_date",
    "organization_updated_date",
    "organization_reason",
    "organization_responsible"
) AS
SELECT ME.CD_MULTI_EMPRESA AS "organization_code",
       ME.DS_RAZAO_SOCIAL AS "organization_name",
       ME.DS_MULTI_EMPRESA AS "organization_alias",
       TO_CHAR(ME.CD_CGC) AS "organization_primary_doc",
       TO_CHAR(ME.NR_CNES) AS "organization_secondary_doc",
       NVL(TO_CHAR(MES.NR_ANS), ME.CD_EMPRESA_ANS) AS "organization_third_document",
       ME.CD_NAT_JURIDICA_SPED_REINF AS "organization_type", -- codigo_reinf
       C.CD_IBGE AS "organization_location_code",
       TL.DS_TIPO_LOGRADOURO || ' ' || ME.DS_ENDERECO AS "organization_address",
       ME.NR_ENDERECO AS "organization_number",
       NULL AS "organization_complement",
       C.NM_CIDADE AS "organization_city",
       ME.NM_BAIRRO AS "organization_neighborhood",
       ME.CD_UF AS "organization_state",
       ME.NR_CEP AS "organization_zipcode",
       ME.NR_CNPJCPF_REPRES_LEGAL AS "organization_person_resp",
       ME.NR_DDD_EMPRESA AS "organization_ddd_phone",
       ME.NR_TELEFONE_EMPRESA AS "organization_phone",
       NULL AS "organization_website",
       NULL AS "organization_email",
       NULL AS "organization_management_type",
       NULL AS "organization_created_date",
       NULL AS "organization_updated_date",
       NULL AS "organization_reason",
       NULL AS "organization_responsible"
  FROM DBAMV.MULTI_EMPRESAS ME
  LEFT JOIN DBAMV.MULTI_EMPRESAS_MV_SAUDE MES ON ME.CD_MULTI_EMPRESA = MES.CD_MULTI_EMPRESA
  LEFT JOIN DBAMV.CIDADE C ON ME.CD_CIDADE = C.CD_CIDADE
  LEFT JOIN DBAMV.TIPO_LOGRADOURO TL ON ME.CD_TIPO_LOGRADOURO = TL.CD_TIPO_LOGRADOURO
  -- Missing indexes on both sides of the JOIN.
  -- JOIN performed using fields that aren't part of primary and foreign keys.
  -- JOIN performed using fields with the same datatype, but different lengths.
  LEFT JOIN DBAMV.ESTADO E ON ME.CD_UF = E.DS_SIGLA;

