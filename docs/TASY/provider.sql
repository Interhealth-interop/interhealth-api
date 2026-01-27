CREATE OR REPLACE VIEW "TASY"."PROVIDER" AS 
SELECT DISTINCT M.CD_PESSOA_FISICA AS "provider_code",
       M.NM_GUERRA AS "provider_name",
       PF.NR_CPF AS "provider_primary_document",
       PF.NR_IDENTIDADE AS "provider_secondary_document",
       PF.NR_CARTAO_NAC_SUS AS "provider_third_document",
       PF.NR_CNH AS "provider_fourth_document",
       NULL AS "provider_fifth_document",
       PF.NR_REG_GERAL_ESTRANG AS "provider_sixth_document",
       DECODE(PF.IE_SEXO, 'M', 'Masculino', 'F', 'Feminino', 'I', 'Indeterminado') AS "provider_sex",
       G.DS_GENERO AS "provider_gender",
       DS_CONSELHO AS "provider_register_name",
       DS_CODIGO_PROF AS "provider_register",
       UF_CRM AS "provider_register_uf",
       DS_ESPECIALIDADE AS "provider_specialty",
       NULL AS "provider_subspecialty",
       DS_CARGO AS "provider_occupation",
       NR_DDD_TELEFONE AS "provider_ddd_phone",
       NR_TELEFONE AS "provider_phone",
       PF.NR_DDD_CELULAR AS "provider_ddd_mobile",
       PF.NR_TELEFONE_CELULAR AS "provider_mobile",
       NR_DDD_FONE_ADIC AS "provider_ddd_phone_contact",
       DS_FONE_ADIC AS "provider_phone_contact",
       CPF.DS_EMAIL AS "provider_email",
       TO_CHAR(PF.DT_NASCIMENTO,'YYYY-MM-DD') AS "provider_birth_date",
       DECODE(M.NR_CRM, NULL, 'N', 'S') AS "provider_ishealthprofessional",
       TO_CHAR(M.DT_ATUALIZACAO_NREC, 'YYYY-MM-DD') AS "provider_create_date",
       TO_CHAR(M.DT_ATUALIZACAO_NREC, 'HH24:MI:SS') AS "provider_create_time",
       TO_CHAR(M.DT_ATUALIZACAO, 'YYYY-MM-DD') AS "provider_update_date",
       TO_CHAR(M.DT_ATUALIZACAO, 'HH24:MI:SS') AS "provider_update_time",
       NULL AS "provider_reason",
       M.NM_USUARIO AS "provider_responsible"
  FROM TASY.MEDICO M
       JOIN TASY.PESSOA_FISICA PF ON M.CD_PESSOA_FISICA = PF.CD_PESSOA_FISICA
       JOIN TASY.COMPL_PESSOA_FISICA CPF ON M.CD_PESSOA_FISICA = CPF.CD_PESSOA_FISICA AND CPF.IE_TIPO_COMPLEMENTO = 1
       LEFT JOIN TASY.MEDICO_ESPECIALIDADE ME ON ME.CD_PESSOA_FISICA = ME.CD_PESSOA_FISICA
       LEFT JOIN TASY.ESPECIALIDADE_MEDICA EM ON EM.CD_ESPECIALIDADE = ME.CD_ESPECIALIDADE
       LEFT JOIN TASY.GENERO G ON PF.NR_GENERO = G.NR_SEQUENCIA
       LEFT JOIN TASY.CONSELHO_PROFISSIONAL CP ON PF.NR_SEQ_CONSELHO = CP.NR_SEQUENCIA
       LEFT JOIN TASY.CARGO C ON PF.CD_CARGO = C.CD_CARGO;