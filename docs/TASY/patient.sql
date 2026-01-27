CREATE OR REPLACE VIEW "TASY"."PATIENT" AS
SELECT PF.NR_PRONTUARIO AS "patient_code",
       PF.CD_PESSOA_FISICA AS "prontuary_patient",
       PF.NM_PESSOA_FISICA AS "patient_name",
       PF.NM_SOCIAL AS "patient_social_name",
       PF.IE_TIPO_SANGUE AS "patient_blood_type",
       TO_CHAR(PF.DT_NASCIMENTO, 'YYYY-MM-DD') AS "patient_birth_date",
       DECODE(PF.IE_SEXO, 'M', 'Masculino', 'F', 'Feminino', 'I', 'Indeterminado') AS "patient_sex",
       G.DS_GENERO AS "patient_gender",
       MAE.NM_CONTATO AS "patient_mother_name",
       PAI.NM_CONTATO AS "patient_father_name",
       PF.NR_CPF AS "patient_primary_document",
       PF.NR_IDENTIDADE AS "patient_secondary_document",
       PF.NR_CNH AS "patient_third_document",
       PF.NR_CARTAO_NAC_SUS AS "patient_fourth_document",
       NULL AS "patient_fifth_document",
       PF.NR_REG_GERAL_ESTRANG AS "patient_sixth_document",
       pf.nr_pASsaporte AS "patient_seventh_document",
       PF.NR_CERT_NASC AS "patient_eighth_document",
       PF.CD_DECLARACAO_NASC_VIVO AS "patient_ninth_document",
       CASE WHEN MAE.NR_CPF IS NOT NULL THEN MAE.NR_CPF
            WHEN MAE.NR_CPF IS NULL THEN RESP.NR_CPF END AS "patient_tenth_document",
       CASE WHEN MAE.NR_CPF IS NOT NULL THEN NULL
            WHEN MAE.NR_CPF IS NULL THEN RESP.DS_PARENTESCO END AS "patient_relationship",
       CP.DS_COR_PELE AS "patient_race",
       SE.DS_ETNIA AS "patient_ethnicity",
       R.DS_RELIGIAO AS "patient_religion",
       C.DS_CARGO AS "patient_profession",
       VD1.DS_VALOR_DOMINIO AS "patient_education",
       N.DS_NACIONALIDADE AS "patient_nationality",
       DS_TIPO_LOGRADOURO AS "patient_accommodation",
       VD2.DS_VALOR_DOMINIO AS "patient_marital_status",
       CPF.DS_EMAIL AS "patient_email",
       PF.NR_DDD_CELULAR AS "patient_ddd_phone",
       PF.NR_TELEFONE_CELULAR AS "patient_phone",
       PF.NR_DDD_CELULAR AS "patient_ddd_mobile",
       PF.NR_TELEFONE_CELULAR AS "patient_mobile",
       RESP1.NR_TELEFONE AS "patient_phone_contact",
       RESP1.NR_DDD_TELEFONE AS "patient_ddd_phone_contact",
       CPF.DS_ENDERECO AS "patient_address",
       CPF.NR_ENDERECO AS "patient_number",
       CPF.DS_COMPLEMENTO AS "patient_complement",
       CPF.DS_MUNICIPIO AS "patient_city",
       CPF.DS_BAIRRO AS "patient_neighborhood",
       CPF.SG_ESTADO AS "patient_state",
       N.DS_NACIONALIDADE AS "patient_country",
       CPF.CD_CEP AS "patient_zipcode",
       PFC.CD_BANCO AS "patient_bank_number",
       PFC.CD_AGENCIA_BANCARIA||'-'||IE_DIGITO_AGENCIA AS "patient_agency",
       PFC.NR_CONTA||NR_DIGITO_CONTA AS "patient_account",
       IE_TIPO_CONTA AS "patient_account_type",
       RESP.RESP_PACIENTE AS "patient_related_person",
       TO_CHAR(PF.DT_ATUALIZACAO_NREC, 'YYYY-MM-DD') AS "patient_created_date",
       TO_CHAR(PF.DT_ATUALIZACAO, 'YYYY-MM-DD') AS "patient_updated_date",
       CASE WHEN DT_OBITO IS NOT NULL THEN 'I'
       ELSE 'A' END AS "patient_status",
       'N' AS "patient_blocked"
  FROM TASY.PESSOA_FISICA PF
       LEFT JOIN TASY.COMPL_PESSOA_FISICA CPF ON PF.CD_PESSOA_FISICA = CPF.CD_PESSOA_FISICA
       LEFT JOIN TASY.GENERO G ON PF.NR_GENERO = G.NR_SEQUENCIA
       LEFT JOIN TASY.COR_PELE CP ON PF.NR_SEQ_COR_PELE = CP.NR_SEQUENCIA
       LEFT JOIN TASY.SUS_ETNIA SE ON PF.NR_SEQ_ETNIA = SE.NR_SEQUENCIA
       LEFT JOIN TASY.RELIGIAO R ON R.CD_RELIGIAO = PF.CD_RELIGIAO
       LEFT JOIN TASY.CARGO C ON C.CD_CARGO = PF.CD_CARGO
       LEFT JOIN TASY.NACIONALIDADE N ON PF.CD_NACIONALIDADE = N.CD_NACIONALIDADE
       LEFT JOIN TASY.SUS_TIPO_LOGRADOURO STL ON STL.CD_TIPO_LOGRADOURO = CPF.CD_TIPO_LOGRADOURO
       LEFT JOIN TASY.PESSOA_FISICA_CONTA PFC ON PF.CD_PESSOA_FISICA = PFC.CD_PESSOA_FISICA
       LEFT JOIN TASY.VALOR_DOMINIO VD1 ON IE_GRAU_INSTRUCAO = VD1.VL_DOMINIO AND VD1.CD_DOMINIO = 10
       LEFT JOIN TASY.VALOR_DOMINIO VD2 ON IE_ESTADO_CIVIL = VD1.VL_DOMINIO AND VD1.CD_DOMINIO = 5
       LEFT JOIN (SELECT CD_PESSOA_FISICA,
                         NM_CONTATO,
                         NR_CPF
                    FROM TASY.COMPL_PESSOA_FISICA CPF1
                   WHERE NR_SEQ_PARENTESCO = 6) MAE ON MAE.CD_PESSOA_FISICA = PF.CD_PESSOA_FISICA
       LEFT JOIN (SELECT CPF2.CD_PESSOA_FISICA,
                         CPF2.NM_CONTATO
                    FROM TASY.COMPL_PESSOA_FISICA CPF2
                   WHERE CPF2.NR_SEQ_PARENTESCO = 7) PAI ON PAI.CD_PESSOA_FISICA = PF.CD_PESSOA_FISICA
       LEFT JOIN (SELECT CPF2.CD_PESSOA_FISICA,
                         CPF2.NM_CONTATO AS RESP_PACIENTE,
                         DS_PARENTESCO,
                         CPF2.NR_CPF
                    FROM TASY.COMPL_PESSOA_FISICA CPF2
                         LEFT JOIN TASY.GRAU_PARENTESCO GP ON CPF2.NR_SEQ_PARENTESCO = GP.NR_SEQUENCIA
                   WHERE CPF2.IE_TIPO_COMPLEMENTO = 3) RESP ON RESP.CD_PESSOA_FISICA = PF.CD_PESSOA_FISICA
       LEFT JOIN (SELECT CPF3.CD_PESSOA_FISICA,
                         CPF3.NR_TELEFONE,
                         CPF3.NR_DDD_TELEFONE
                    FROM TASY.COMPL_PESSOA_FISICA CPF3
                   WHERE CPF3.NR_SEQ_TIPO_COMPL_ADIC = 1) RESP1 ON RESP.CD_PESSOA_FISICA = PF.CD_PESSOA_FISICA;