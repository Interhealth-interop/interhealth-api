CREATE OR REPLACE VIEW "TASY"."ENCOUNTER" AS 
SELECT DISTINCT AP.NR_ATENDIMENTO AS "encounter_code",
     CP.NR_INTERNO_CONTA AS "encounter_accounter_code",
     PF.NR_PRONTUARIO AS "encounter_patient_code",
     AP.NR_ATENDIMENTO AS "encounter_schedule_code",
     AP.CD_ESTABELECIMENTO AS "encounter_organization_code",
     APU.CD_SETOR_ATENDIMENTO AS "encounter_unit_code",
     AP.CD_MEDICO_RESP AS "encounter_provider_code",
     VD1.DS_VALOR_DOMINIO AS "encounter_type",
     VD2.DS_VALOR_DOMINIO AS "encounter_modality",
     NULL AS "encounter_health_program",
     DECODE(AP.IE_STATUS_ATENDIMENTO, 'E', 'finished', 'in-progress') AS "encounter_status",
     TO_CHAR(DT_RECEBIMENTO_SENHA, 'YYYY-MM-DD') AS "encounter_entry_date",
     TO_CHAR(DT_RECEBIMENTO_SENHA, 'HH24:MI:SS') AS "encounter_entry_time",
     TO_CHAR(AP.DT_ENTRADA, 'YYYY-MM-DD') AS "encounter_date_start",
     TO_CHAR(AP.DT_ENTRADA, 'HH24:MI:SS') AS "encounter_time_start",
     TO_CHAR(AP.DT_INICIO_ATENDIMENTO, 'YYYY-MM-DD') AS "encounter_filter_date",
     TO_CHAR(AP.DT_INICIO_ATENDIMENTO, 'HH24:MI:SS') AS "encounter_filter_time",
     TO_CHAR(AP.DT_ATEND_MEDICO,'YYYY-MM-DD') AS "encounter_medical_date_start",
     TO_CHAR(AP.DT_ATEND_MEDICO,'HH24:MI:SS') AS "encounter_medical_time_start",
     TO_CHAR(AP.DT_ALTA_MEDICO, 'YYYY-MM-DD') AS "encounter_medical_date_end",
     TO_CHAR(AP.DT_ALTA_MEDICO, 'HH24:MI:SS') AS "encounter_medical_time_end",
     TO_CHAR(AP.DT_ALTA, 'YYYY-MM-DD') AS "encounter_date_end",
     TO_CHAR(AP.DT_ALTA, 'HH24:MI:SS') AS "encounter_time_end",
     MA.DS_MOTIVO_ALTA AS "encounter_clinical_outcome",
     NULL AS "encounter_accident_type",
     SCI.DS_CARATER_INTERNACAO AS "encounter_request_reason",
     TO_CHAR(AP.DT_ATUALIZACAO_NREC, 'YYYY-MM-DD') AS "encounter_created_date",
     TO_CHAR(AP.DT_ATUALIZACAO_NREC, 'HH24:MI:SS') AS "encounter_created_time",
     TO_CHAR(AP.DT_ATUALIZACAO, 'YYYY-MM-DD') AS "encounter_updated_date",
     TO_CHAR(AP.DT_ATUALIZACAO, 'HH24:MI:SS') AS "encounter_updated_time",
     NULL AS "encounter_reason",
     AP.NM_USUARIO AS "encounter_responsible"
FROM TASY.ATENDIMENTO_PACIENTE AP
     JOIN TASY.CONTA_PACIENTE CP ON AP.NR_ATENDIMENTO = CP.NR_ATENDIMENTO
     JOIN TASY.PESSOA_FISICA PF ON AP.CD_PESSOA_FISICA = PF.CD_PESSOA_FISICA
     JOIN TASY.ESTABELECIMENTO E ON AP.CD_ESTABELECIMENTO = E.CD_ESTABELECIMENTO
     JOIN TASY.ATEND_PACIENTE_UNIDADE APU ON AP.NR_ATENDIMENTO = APU.NR_ATENDIMENTO
     LEFT JOIN TASY.CLASSIFICACAO_ATENDIMENTO CA ON CA.NR_SEQUENCIA = AP.NR_SEQ_CLASSIFICACAO
     LEFT JOIN TASY.MOTIVO_ALTA MA ON MA.CD_MOTIVO_ALTA = AP.CD_MOTIVO_ALTA
     LEFT JOIN TASY.SUS_CARATER_INTERNACAO SCI ON AP.IE_CARATER_INTER_SUS = SCI.CD_CARATER_INTERNACAO
     LEFT JOIN TASY.VALOR_DOMINIO VD1 ON AP.IE_TIPO_ATENDIMENTO = VD1.VL_DOMINIO AND VD1.CD_DOMINIO = 12
     LEFT JOIN TASY.VALOR_DOMINIO VD2 ON AP.IE_TIPO_ATEND_TISS = VD2.VL_DOMINIO AND VD2.CD_DOMINIO = 1761;