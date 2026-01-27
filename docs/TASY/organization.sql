CREATE OR REPLACE VIEW "TASY"."ORGANIZATION" AS
SELECT E.CD_ESTABELECIMENTO AS "organization_code",
       PJ.DS_RAZAO_SOCIAL AS "organization_name",
       PJ.NM_FANTASIA AS "organization_alias",
       E.CD_CGC AS "organization_primary_doc",
       E.CD_CNS AS "organization_secondary_doc",
       E.CD_ANS AS "organization_third_doc",
       TPJ.DS_TIPO_PESSOA AS "organization_type",
       PJ.CD_MUNICIPIO_IBGE AS "organization_location_code",
       PJ.DS_ENDERECO AS "organization_address",
       NR_ENDERECO AS "organization_number",
       DS_COMPLEMENTO AS "organization_complement",
       DS_MUNICIPIO AS "organization_city",
       DS_BAIRRO AS "organization_neighborhood",
       PJ.SG_ESTADO AS "organization_state",
       CD_CEP AS "organization_zipcode",
       PJ.CD_PF_RESP_TECNICO AS "organization_person_resp",
       PJ.NR_TELEFONE AS "organization_ddd_phone",
       PJ.NR_TELEFONE AS "organization_phone",
       ds_site_internet "organization_website",
       DS_EMAIL_NFE AS "organization_email",
       NULL AS "organization_management_type",
       TO_CHAR(PJ.DT_ATUALIZACAO_NREC, 'YYYY-MM-DD') AS "organization_created_date",
       TO_CHAR(PJ.DT_ATUALIZACAO, 'YYYY-MM-DD') AS "organization_updated_date",
       NULL AS "organization_reason",
       PJ.NM_USUARIO AS "organization_responsible"
  FROM TASY.ESTABELECIMENTO E
       JOIN TASY.PESSOA_JURIDICA PJ ON PJ.CD_CGC = E.CD_CGC
       LEFT JOIN TASY.TIPO_PESSOA_JURIDICA TPJ ON TPJ.CD_TIPO_PESSOA = PJ.CD_TIPO_PESSOA
       JOIN TASY.EMPRESA EMP ON EMP.CD_EMPRESA = E.CD_EMPRESA;