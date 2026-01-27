CREATE OR REPLACE VIEW "DBAMV"."ENCOUNTER"
(
    "encounter_code",
    "encounter_accounter_code",
    "encounter_patient_code",
    "encounter_schedule_code",
    "encounter_organization_code",
    "encounter_unit_code",
    "encounter_provider_code",
    "encounter_type",
    "encounter_modality",
    "encounter_health_program",
    "encounter_status",
    "encounter_entry_date",
    "encounter_entry_time",
    "encounter_date_start",
    "encounter_time_start",
    "encounter_filter_date",
    "encounter_filter_time",
    "encounter_medical_date_start",
    "encounter_medical_time_start",
    "encounter_medical_date_end",
    "encounter_medical_time_end",
    "encounter_date_end",
    "encounter_time_end",
    "encounter_clinical_outcome",
    "encounter_accident_type",
    "encounter_request_reason",
    "created_date",
    "created_time",
    "updated_date",
    "updated_time",
    "encounter_reason",
    "encounter_responsible"
) AS
SELECT A.CD_ATENDIMENTO AS "encounter_code",
       NULL AS "encounter_accounter_code",
       A.CD_PACIENTE AS "encounter_patient_code",
       IAC.CD_IT_AGENDA_CENTRAL AS "encounter_schedule_code",
       A.CD_MULTI_EMPRESA AS "encounter_organization_code",
       OA.CD_SETOR AS "encounter_unit_code",
       A.CD_PRESTADOR AS "encounter_provider_code",
       CASE A.TP_ATENDIMENTO
            WHEN 'A' THEN 'AMBULATORIAL'
            WHEN 'B' THEN 'BUSCA ATIVA'
            WHEN 'E' THEN 'EXTERNO'
            WHEN 'H' THEN 'HOME CARE'
            WHEN 'I' THEN 'INTERNACAO'
            WHEN 'S' THEN 'SUS - AIH'
            WHEN 'U' THEN 'URGENCIA'
            ELSE TO_CHAR(A.TP_ATENDIMENTO) END AS "encounter_type",
       CASE A.TP_ATENDIMENTO_TISS
            WHEN 1 THEN 'REMOCAO'
            WHEN 10 THEN 'TRS TERAPIA RENAL SUBSTITUTA'
            WHEN 2 THEN 'PEQUENA CIRURGIA'
            WHEN 3 THEN 'TERAPIAS'
            WHEN 4 THEN 'CONSULTA'
            WHEN 5 THEN 'EXAMES'
            WHEN 6 THEN 'ATENDIMENTO HOSPITALAR'
            WHEN 7 THEN 'SADT INTERNACAO'
            WHEN 8 THEN 'QUIMIOTERAPIA'
            WHEN 9 THEN 'RADIOTERAPIA'
            ELSE  TO_CHAR(A.TP_ATENDIMENTO_TISS) END AS "encounter_modality",
       NULL "encounter_health_program",
       TS.DS_TIP_SITUACAO AS "encounter_status",
       TO_CHAR(TA.DH_PRE_ATENDIMENTO,'YYYY-MM-DD') AS "encounter_entry_date",
       TO_CHAR(TA.DH_PRE_ATENDIMENTO, 'HH24:MI:SS') AS "encounter_entry_time",
       TO_CHAR(A.DT_ATENDIMENTO,'YYYY-MM-DD') AS "encounter_date_start",
       TO_CHAR(A.HR_ATENDIMENTO, 'HH24:MI:SS') AS "encounter_time_start",
       TO_CHAR(S1.DH_PROCESSO,'YYYY-MM-DD') AS "encounter_filter_date",
       TO_CHAR(S1.DH_PROCESSO, 'HH24:MI:SS') AS "encounter_filter_time",
       TO_CHAR(S2.DH_PROCESSO,'YYYY-MM-DD')AS "encounter_medical_date_start",
       TO_CHAR(S2.DH_PROCESSO, 'HH24:MI:SS')AS "encounter_medical_time_start",
       TO_CHAR(A.DT_ALTA_MEDICA,'YYYY-MM-DD') AS "encounter_medical_date_end",
       TO_CHAR(A.HR_ALTA_MEDICA, 'HH24:MI:SS') AS "encounter_medical_time_end",
       TO_CHAR(A.DT_ALTA,'YYYY-MM-DD') AS "encounter_date_end",
       TO_CHAR(A.HR_ALTA, 'HH24:MI:SS') AS "encounter_time_end",
       CASE WHEN (TRE.DS_TIP_RES IS NOT NULL AND MA.DS_MOT_ALT IS NOT NULL)
            THEN TRE.DS_TIP_RES || ' - ' || MA.DS_MOT_ALT
            ELSE TRE.DS_TIP_RES || MA.DS_MOT_ALT END AS "encounter_clinical_outcome",
       A.TP_ACIDENTE_TISS AS "encounter_accident_type",
       TI.DS_TIPO_INTERNACAO AS "encounter_request_reason",
       TO_CHAR(A.DT_ATENDIMENTO,'YYYY-MM-DD') AS "created_date",
       TO_CHAR(A.HR_ATENDIMENTO, 'HH24:MI:SS') AS "created_time", 
       NULL "updated_date",
       NULL "updated_time",
       NULL "encounter_reason",
       NULL "encounter_responsible"
  FROM DBAMV.ATENDIME A
  LEFT JOIN DBAMV.TIP_SITUACAO TS ON A.CD_TIP_SITUACAO = TS.CD_TIP_SITUACAO
  left join DBAMV.TRIAGEM_ATENDIMENTO TA on A.CD_ATENDIMENTO = TA.CD_ATENDIMENTO
  left join DBAMV.SACR_TEMPO_PROCESSO S1 ON S1.CD_ATENDIMENTO = A.CD_ATENDIMENTO AND S1.CD_TIPO_TEMPO_PROCESSO = 11 -- TRIAGEM
  left join DBAMV.SACR_TEMPO_PROCESSO S2 ON S2.CD_ATENDIMENTO = A.CD_ATENDIMENTO AND S2.CD_TIPO_TEMPO_PROCESSO = 31 -- INICIO ATENDIMENTO MEDICO
  LEFT JOIN DBAMV.IT_AGENDA_CENTRAL IAC ON A.CD_ATENDIMENTO = IAC.CD_ATENDIMENTO
  LEFT JOIN DBAMV.TIPO_INTERNACAO TI ON A.CD_TIPO_INTERNACAO = TI.CD_TIPO_INTERNACAO
  LEFT JOIN DBAMV.TIP_RES TRE ON A.CD_TIP_RES = TRE.CD_TIP_RES
  LEFT JOIN DBAMV.MOT_ALT MA ON A.CD_MOT_ALT = MA.CD_MOT_ALT
  LEFT JOIN DBAMV.ORI_ATE OA ON OA.CD_ORI_ATE = A.CD_ORI_ATE;