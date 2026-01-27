CREATE OR REPLACE VIEW "DBAMV"."SCHEDULE"
(
    "schedule_encounter_code",
    "schedule_code",
    "schedule_patient_code",
    "schedule_location_code",
    "schedule_provider_code",
    "schedule_speciality",
    "schedule_status", 
    "schedule_event",
    "schedule_event_description",
    "schedule_type",
    "schedule_date",
    "schedule_time",
    "schedule_created_date",
    "schedule_created_time",
    "schedule_updated_date",
    "schedule_updated_time",
    "schedule_reason",
    "schedule_responsible",
    "schedule_description",
    "schedule_queues.id",
    "schedule_shouldWait"
) AS
SELECT 
       IAC.CD_ATENDIMENTO as "schedule_encounter_code",
       IAC.CD_IT_AGENDA_CENTRAL AS "schedule_code",
       IAC.CD_PACIENTE AS "schedule_patient_code",
       AC.CD_SETOR AS "schedule_location_code",
       AC.CD_PRESTADOR AS "schedule_provider_code",
       SD.DS_SER_DIS AS "schedule_speciality",
       CASE TP_SITUACAO
            WHEN 'M' THEN 'Marcado'
            WHEN 'A' THEN 'Aguardando'
            WHEN 'E' THEN 'Atendido'
            WHEN 'C' THEN 'Cancelado'
            ELSE TO_CHAR(TP_SITUACAO)
            END AS "schedule_status", 
       NULL AS "schedule_event",
       IA.DS_ITEM_AGENDAMENTO AS "schedule_event_description",
       TM.DS_TIP_MAR AS "schedule_type",
       TO_CHAR(IAC.HR_AGENDA, 'YYYY-MM-DD') AS "schedule_date",
       TO_CHAR(IAC.HR_AGENDA, 'HH24:MI:SS') AS "schedule_time",
       TO_CHAR(IAC.DT_GRAVACAO, 'YYYY-MM-DD') AS  "schedule_created_date",  
       TO_CHAR(IAC.DT_GRAVACAO, 'HH24:MI:SS') AS "schedule_created_time",
       NULL AS "schedule_updated_date",
       NULL AS "schedule_updated_time",
       NULL AS "schedule_reason",
       NULL AS "schedule_responsible",
       IAC.DS_OBSERVACAO_GERAL as "schedule_description",
       NULL AS "schedule_queues.id",
       NULL AS "schedule_shouldWait"
  FROM DBAMV.AGENDA_CENTRAL AC
  INNER JOIN DBAMV.IT_AGENDA_CENTRAL IAC ON AC.CD_AGENDA_CENTRAL = IAC.CD_AGENDA_CENTRAL
  LEFT JOIN DBAMV.ITEM_AGENDAMENTO IA ON IAC.CD_ITEM_AGENDAMENTO = IA.CD_ITEM_AGENDAMENTO
  LEFT JOIN DBAMV.TIP_MAR TM ON IAC.CD_TIP_MAR = TM.CD_TIP_MAR
  LEFT JOIN DBAMV.SER_DIS SD ON IAC.CD_SER_DIS = SD.CD_SER_DIS;

